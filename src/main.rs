mod args;

use args::Args;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{args:#?}");
}
