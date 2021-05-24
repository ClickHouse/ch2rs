use anyhow::Result;

mod codegen;
mod miner;
mod options;
mod schema;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = options::Options {
        url: "http://localhost:8123".into(),
        user: None,
        password: None,
        database: "default".into(),
        table: "balance_log".into(),
        owned: false,
        types: vec![],
        overrides: vec![],
    };

    let table = miner::mine(&options).await?;

    codegen::generate(&table, &options)?;

    Ok(())
}
