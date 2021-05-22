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
        // TODO: remove support for many tables.
        include: vec!["%.balance_log".into()],
        exclude: vec![],
    };

    let mut schema = miner::mine(&options).await?;

    codegen::generate(&schema.tables[0], &options)?;

    Ok(())
}
