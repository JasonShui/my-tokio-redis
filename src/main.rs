use mini_redis::{client, Result};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("key1", "world".into()).await?;
    let result = client.get("key1").await?;
    println!("{:?}", result);

    Ok(())
}
