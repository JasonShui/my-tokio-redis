use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let tx_clone = tx.clone();

    let recv_mgr = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key } => {
                    let val = client.get(&key).await.unwrap();
                    println!("val of key {:?}: {:?}", key, val);
                }
                Command::Set { key, val } => {
                    client.set(&key, val.clone()).await.unwrap();
                    println!("set (key, val) ({:?} {:?})", key, val);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        tx.send(Command::Get {
            key: "key1".to_string(),
        })
        .await
        .unwrap();
    });

    let t2 = tokio::spawn(async move {
        tx_clone
            .send(Command::Set {
                key: "key1".to_string(),
                val: "world".into(),
            })
            .await
            .unwrap();
    });

    t1.await.unwrap();
    t2.await.unwrap();
    recv_mgr.await.unwrap();
}
