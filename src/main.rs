#![allow(clippy::module_name_repetitions)]

mod args;
mod download;
mod search;

use args::{
    Args,
    Commands::{Download, Search},
};
use clap::Parser;
use download::handle_download;
use search::handle_search;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Download(x) => handle_download(x).await,
        Search(x) => handle_search(x).await,
    }
}
