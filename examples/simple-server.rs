use bytes::Bytes;
use mini_redis::Command::{self, Get, Set};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type MyRedisDB = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: MyRedisDB) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.lock()
                    .unwrap()
                    .insert(cmd.key().to_string(), cmd.value().clone());
                println!("recv set cmd {:?}", cmd);
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                println!("recv get cmd {:?}", cmd);
                if let Some(value) = db.lock().unwrap().get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => {
                println!("not implemented cmd: {:?}", cmd);
                Frame::Null
            }
        };

        println!("curr redis data: {:?}", db);
        connection.write_frame(&response).await.unwrap();
    }
}
