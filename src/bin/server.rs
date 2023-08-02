use std::collections::HashMap;
use std::error::Error;
use std::format;
use std::sync::Arc;

use async_std::channel::{self, Sender};
use async_std::io::BufReader;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::{Mutex, RwLock};

use futures::{select, StreamExt, TryStreamExt};

#[derive(Debug)]
struct Message {
    sender: String,
    contents: String,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let login_map = Arc::new(RwLock::new(HashMap::new()));

    listener
        .incoming()
        .map(|stream_res| stream_res.map(|stream| (stream, Arc::clone(&login_map))))
        .try_for_each_concurrent(100_000, |(stream, login_map)| async move {
            handle_connection(stream, login_map).await?;
            Ok::<(), std::io::Error>(())
        })
        .await?;

    Ok(())
}

async fn handle_connection(
    mut stream: TcpStream,
    login_map: Arc<RwLock<HashMap<String, Mutex<Sender<Message>>>>>,
) -> std::io::Result<()> {
    // first future must be login request from client
    let mut input = BufReader::new(stream.clone()).lines().fuse();
    let login = input.next().await.unwrap()?;
    let (tx, rx) = channel::unbounded::<Message>();
    let mut rx = rx.fuse();

    {
        let mut login_map = login_map.write().await;
        // TODO ensure entry doesn't already exist
        login_map.insert(login.clone(), Mutex::new(tx));
    }

    // once client has logged in, we can select b/w send and recv futures
    loop {
        select! {
            msg_for_client = rx.next() => {
                handle_msg_for_client(&mut stream, msg_for_client).await
            },
            msg_from_client = input.next() => {
                handle_msg_from_client(&login, msg_from_client, &login_map).await
            }
        }
    }
}

async fn handle_msg_for_client(stream: &mut TcpStream, message: Option<Message>) {
    if let Some(message) = message {
        let _ = stream
            .write(format!("from {}: {}\n", message.sender, message.contents).as_bytes())
            .await;
    }
}

async fn handle_msg_from_client(
    sender: &str,
    message: Option<std::io::Result<String>>,
    login_map: &Arc<RwLock<HashMap<String, Mutex<Sender<Message>>>>>,
) {
    if let Some(Ok(message)) = message {
        let msg_parts = message.split(':').collect::<Vec<_>>();
        let recipients = msg_parts[0].split(',').map(str::trim).collect::<Vec<_>>();
        let contents = msg_parts[1].trim();

        {
            let login_map = login_map.read().await;
            for r in recipients {
                if let Some(tx) = login_map.get(r) {
                    let _ = tx
                        .lock()
                        .await
                        .send(Message {
                            sender: sender.to_owned(),
                            contents: contents.to_owned(),
                        })
                        .await;
                }
            }
        }
    }
}
