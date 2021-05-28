use anyhow::Result;
use structopt::StructOpt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = ch2rs::Options::from_args();
    let code = ch2rs::generate(options).await?;
    println!("{}", code);
    Ok(())
}
