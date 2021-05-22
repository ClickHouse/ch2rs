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
        include: vec!["%".into()],
        exclude: vec![],
    };

    let schema = miner::mine(&options).await?;
    println!("{:#?}", schema);

    Ok(())
}
