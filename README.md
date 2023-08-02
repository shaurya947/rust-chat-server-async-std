## Spec

The chat uses a simple text protocol over TCP. The protocol consists of utf-8 messages, separated by `\n`.

The client connects to the server and sends `login` as a first line. After that, the client can send messages to other clients using the following syntax:

```
login1, login2, ... loginN: message
```

Each of the specified clients then receives a `from login: message` message.

A possible session might look like this

```
On Alice's computer:   |   On Bob's computer:

> alice                |   > bob
> bob: hello               < from alice: hello
                       |   > alice, bob: hi!
                           < from bob: hi!
< from bob: hi!        |
```