use std::env;
use std::process;

use crate::args::DownloadCommand;
use crate::types::Post;
use anyhow::{anyhow, Result};
use hyper::client::HttpConnector;
use hyper::Body;
use hyper::Client;
use hyper::Request;
use hyper_tls::HttpsConnector;
use rayon::prelude::*;
use scraper::Html;
use scraper::Selector;
use urlencoding::encode;

pub async fn handle_download(args: DownloadCommand) {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let total_pages = if let Ok(x) = get_total_pages(&args.tags, &client).await {
        x
    } else {
        eprintln!("No results found for tags: {:?}", args.tags);
        process::exit(1);
    };
    println!("Total pages: {total_pages}");

    let posts = fetch_posts(&args.tags, total_pages, &client).await;
}

async fn fetch_posts(
    tags: &[String],
    pages_amount: i32,
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Vec<Post> {

    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let posts: Vec<Post> = (1..=pages_amount)
        .into_par_iter()
        .flat_map(|page| {
            match get_posts_from_page(&encoded_tags, page, &client).await {
                Ok(x) => x,
                Err(e) => vec![]
            }
        })
        .collect();



    posts
}

async fn get_posts_from_page(
    encoded_tags: &str,
    page: i32,
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Result<Vec<Post>> {

    let posts: Vec<Post> = vec![];
    let mut query =
        format!("https://danbooru.donmai.us/posts.json?page={page}&tags={encoded_tags}")
            + "&limit=200&only=rating,file_url,id,score,file_ext,large_file_url";

    if env::var("LOGIN_NAME").is_ok() && env::var("API_KEY").is_ok() {
        query += &format!(
            "&login={}&api_key={}",
            env::var("LOGIN_NAME").unwrap_or_default(),
            env::var("API_KEY").unwrap_or_default()
        );
    }

    let request = Request::builder()
        .method("GET")
        .uri(&query)
        .header(
            "User-Agent",
            format!("danbooru-rs/{}", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "application/json")
        .body(Body::empty())?;

    let response = client.request(request).await?;
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let json_body = String::from_utf8(body.to_vec())?;
    let posts: Vec<Post> = serde_json::from_str(&json_body)?;

    Ok(posts)

}

async fn get_total_pages(
    tags: &[String],
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Result<i32> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let mut query = format!("https://danbooru.donmai.us/posts?tags={encoded_tags}&limit=200");

    if env::var("LOGIN_NAME").is_ok() && env::var("API_KEY").is_ok() {
        query += &format!(
            "&login={}&api_key={}",
            env::var("LOGIN_NAME")?,
            env::var("API_KEY")?
        );
    }

    let request = Request::builder()
        .method("GET")
        .uri(&query)
        .header(
            "User-Agent",
            format!("danbooru-rs/{}", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "text/html")
        .body(Body::empty())?;

    let response = client.request(request).await?;
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let html_body = String::from_utf8(body.to_vec())?;
    let document = Html::parse_document(&html_body);

    let no_posts_selector = match Selector::parse("#posts > div > p") {
        Ok(x) => x,
        Err(_) => return Err(anyhow!("Failed to parse selector")),
    };

    if document.select(&no_posts_selector).count() != 0 {
        return Err(anyhow!("No results found for tags: {:?}", tags));
    }

    let pagination_selector = match Selector::parse(".paginator-page.desktop-only") {
        Ok(x) => x,
        Err(_) => return Err(anyhow!("Failed to parse selector")),
    };

    let amount: i32 = match document.select(&pagination_selector).last() {
        Some(x) => x.inner_html().parse()?,
        None => 1,
    };

    Ok(amount)
}
