use anyhow::Result;

pub use options::Options;

mod codegen;
mod miner;
mod options;
mod schema;

pub async fn generate(options: Options) -> Result<String> {
    let table = miner::mine(&options).await?;
    let code = codegen::generate(&table, &options)?;
    Ok(code)
}
