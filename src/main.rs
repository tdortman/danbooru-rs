mod args;
mod download;
mod post;
mod search;

use args::{
    Args,
    Commands::{Download, Search},
};
use clap::Parser;
use dotenvy::dotenv;
use download::handle_download;
use search::handle_search;

#[rustfmt::skip]
fn main() {
    let args = Args::parse();
    dotenv().ok();

    match args.command {
        Download(mut x) => match handle_download(&mut x) {
            Ok(())  => (),
            Err(e)  => eprintln!("{e}"),
        },
        Search(x)       => handle_search(&x),
    }
}
