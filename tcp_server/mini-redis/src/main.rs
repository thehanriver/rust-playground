use bytes::Bytes;
use mini_redis::{
    self,
    Command::{self, Get, Set},
    Connection, Frame,
};
use std::{
    collections::HashMap,
    io::Error,
    sync::{Arc, Mutex},
};
use tokio::{
    io::Result,
    net::{TcpListener, TcpStream},
};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    println!("Hello World");
    if let Err(e) = run().await {
        eprintln!("server error: {e}");
    }
}

async fn run() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) -> Result<()> {
    let mut connection = Connection::new(socket);
    let mut db = HashMap::new();

    loop {
        match connection.read_frame().await {
            Ok(Some(frame)) => {
                let response = match Command::from_frame(frame) {
                    Ok(cmd) => match cmd {
                        Set(cmd) => {
                            db.insert(cmd.key().to_string(), cmd.value().to_vec());
                            Frame::Simple("OK".into())
                        }
                        Get(cmd) => db
                            .get(cmd.key())
                            .map(|value| Frame::Bulk(value.clone().into()))
                            .unwrap_or(Frame::Null),
                        _ => Frame::Null,
                    },
                    Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
                };
                connection.write_frame(&response).await?;
            }
            Ok(None) => break, // client closed cleanly
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)), // or log before returning
        }
    }

    // Example of looking at Redis Frame turns TcpStream into a Redis Frame
    // if let Some(frame) = connection.read_frame().await.expect("frame does not exist") {
    //     println!("GOT: {:?}", frame);

    //     let response = Frame::Error("unimplemented".to_string());
    //     connection.write_frame(&response).await?;
    // }

    Ok(())
}
