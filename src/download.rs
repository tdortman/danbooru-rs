use std::env;
use std::process;
use std::str::FromStr;

use crate::args::DownloadCommand;
use anyhow::Result;
use hyper::{client::HttpConnector, Client, Uri};
use urlencoding::encode;

pub async fn handle_download(args: DownloadCommand) {
    let client = Client::new();
    let _total_pages = match get_total_pages(&args.tags, client).await {
        Ok(x) => x,
        Err(e) => {
            eprintln!("No results found for tags: {:?}", args.tags);
            eprintln!("{e:#?}");
            process::exit(1);
        }
    };
}

async fn get_total_pages(tags: &[String], client: Client<HttpConnector>) -> Result<i32> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let mut query = format!("http://danbooru.donmai.us/posts?tags={encoded_tags}&limit=200");

    if env::var("LOGIN_NAME").is_ok() && env::var("API_KEY").is_ok() {
        query += &format!(
            "&login={}&api_key={}",
            env::var("LOGIN_NAME")?,
            env::var("API_KEY")?
        );
    }


    let uri = Uri::from_str(&query)?;
    let response = client.get(uri).await?;
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let html_body = String::from_utf8(body.to_vec())?;
    println!("{:?}", html_body);
    Ok(1)
}
