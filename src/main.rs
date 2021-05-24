use anyhow::Result;
use structopt::StructOpt;

mod codegen;
mod miner;
mod options;
mod schema;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = options::Options::from_args();
    let table = miner::mine(&options).await?;

    codegen::generate(&table, &options)?;

    Ok(())
}
