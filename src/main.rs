use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type ResultResponse<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: ResultResponse<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: ResultResponse<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let tx_clone = tx.clone();

    let recv_mgr = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    resp.send(res).unwrap();
                }
                Command::Set { key, val , resp} => {
                    let res = client.set(&key, val.clone()).await;
                    resp.send(res).unwrap();
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();
        tx.send(Command::Get {
            key: "key1".to_string(),
            resp: res_tx,
        })
        .await
        .unwrap();

        let res = res_rx.await;
        println!("get res {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();
        tx_clone
            .send(Command::Set {
                key: "key1".to_string(),
                val: "world".into(),
                resp: res_tx,
            })
            .await
            .unwrap();

            
        let res = res_rx.await;
        println!("set res {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    recv_mgr.await.unwrap();
}
