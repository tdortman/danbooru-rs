use std::process;

use crate::args::DownloadCommand;
use urlencoding::encode;
use anyhow::Result;

pub async fn handle_download(args: DownloadCommand) {
    let total_pages = match get_total_pages(&args.tags) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("No results found for tags: {:?}", args.tags);
            process::exit(1);
        }
    };
}

fn get_total_pages(tags: &[String]) -> Result<i32> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let query = format!("https://danbooru.donmai.us/posts?tags={encoded_tags}&limit=200");
    println!("{query}");
    Ok(1)
}
