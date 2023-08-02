use std::{error::Error, println};

use async_std::{
    io::{self, prelude::*, BufReader},
    net::TcpStream,
};
use futures::{select, StreamExt};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    let mut user_input = BufReader::new(io::stdin()).lines().fuse();
    let mut server_msg = BufReader::new(stream.clone()).lines().fuse();

    loop {
        select! {
            input = user_input.next() => {
                if let Some(Ok(input)) = input {
                    let input = format!("{}\n", input);
                    let _ = stream.write(input.as_bytes()).await;
                }
            },
            message = server_msg.next() => {
                if let Some(Ok(message)) = message {
                    println!("{message}");
                }
            }
        }
    }
}
