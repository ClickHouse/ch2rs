use anyhow::Result;

mod generator;
mod miner;
mod options;
mod schema;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = options::Options {
        url: "http://localhost:8123".into(),
        user: None,
        password: None,
    };

    miner::mine(&options).await?;
    Ok(())
}
