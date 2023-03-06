use std::env;
use std::process;

use crate::args::DownloadCommand;
use crate::types::Post;
use anyhow::{anyhow, Result};

use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::Html;
use scraper::Selector;
use urlencoding::encode;

pub fn handle_download(args: &DownloadCommand) {
    let client = Client::new();
    let total_pages = get_total_pages(&args.tags, &client).map_or_else(
        |_| {
            eprintln!("No results found for tags: {:?}", args.tags);
            process::exit(1);
        },
        |x| x,
    );

    let posts = fetch_posts(&args.tags, total_pages, &client);

    let progress_bar = ProgressBar::new(posts.len() as u64)
        .with_style(
            ProgressStyle::with_template(
                "{msg} {percent}%  |{wide_bar:0.cyan/blue}| ({pos}/{len}) [{elapsed_precise} elapsed]",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#= "),
        )
        .with_message("Downloading posts");

    posts
        .into_par_iter()
        .progress_with(progress_bar.clone())
        .for_each(|post| {
            match post.download(&client, args) {
                Err(_) | Ok(_) => (),
            };
        });

    progress_bar.finish();
}

fn fetch_posts(tags: &[String], pages_amount: u64, client: &Client) -> Vec<Post> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let progress_bar = ProgressBar::new(pages_amount)
        .with_style(
            ProgressStyle::with_template("{msg} {percent}% |{wide_bar:0.cyan/blue}| ({pos}/{len} pages)")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#= "),
        )
        .with_message("Fetching posts");

    #[allow(clippy::option_if_let_else)]
    let posts: Vec<Post> = (1..=pages_amount)
        .into_par_iter()
        .progress_with(progress_bar.clone())
        .flat_map(
            |page| match get_posts_from_page(&encoded_tags, page, client) {
                Ok(x) => x,
                Err(_) => vec![],
            },
        )
        .collect();

    progress_bar.finish();

    posts
}

fn get_posts_from_page(encoded_tags: &str, page: u64, client: &Client) -> Result<Vec<Post>> {
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

    let response = client
        .get(&query)
        .headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "User-Agent",
                format!("danbooru-rs/{}", env!("CARGO_PKG_VERSION")).parse()?,
            );
            headers.insert("Accept", "application/json".parse()?);
            headers
        })
        .send()?;

    let json_body = String::from_utf8(response.bytes()?.to_vec())?;

    let posts: Vec<Post> = serde_json::from_str(&json_body)?;

    Ok(posts)
}

fn get_total_pages(tags: &[String], client: &Client) -> Result<u64> {
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

    let response = client
        .get(&query)
        .headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "User-Agent",
                format!("danbooru-rs/{}", env!("CARGO_PKG_VERSION")).parse()?,
            );
            headers.insert("Accept", "text/html".parse()?);
            headers
        })
        .send()?;

    let html_body = String::from_utf8(response.bytes()?.to_vec())?;
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

    let amount: u64 = match document.select(&pagination_selector).last() {
        Some(x) => x.inner_html().parse()?,
        None => 1,
    };

    Ok(amount)
}
