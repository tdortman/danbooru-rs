#![allow(clippy::module_name_repetitions)]

mod args;
mod download;
mod search;
mod types;

use args::{
    Args,
    Commands::{Download, Search},
};
use clap::Parser;
use dotenvy::dotenv;
use download::handle_download;
use search::handle_search;

fn main() {
    let args = Args::parse();
    dotenv().ok();

    match args.command {
        Download(x) => handle_download(&x),
        Search(x) => handle_search(&x),
    }
}
