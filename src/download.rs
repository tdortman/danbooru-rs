use std::env;
use std::process;

use crate::args::DownloadCommand;
use crate::post::Post;
use anyhow::{anyhow, Result};

use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::Html;
use scraper::Selector;
use urlencoding::encode;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn handle_download(args: &DownloadCommand) {
    let client = Client::builder()
        .user_agent(format!("{NAME}/{VERSION}"))
        .build()
        .unwrap_or_else(|_| {
            eprintln!("Failed to build request client");
            process::exit(1);
        });
    let total_pages = get_total_pages(&args.tags, &client).unwrap_or_else(|_| {
        eprintln!("No results found that contain all the tags {:?}", args.tags);
        process::exit(1);
    });

    let posts = fetch_posts(&args.tags, total_pages, &client);

    let progress_bar = ProgressBar::new(posts.len() as u64)
        .with_style(
            ProgressStyle::with_template(
                "{msg} {percent}% {wide_bar:0.cyan/blue} ({pos}/{len}) [{elapsed_precise}]",
            )
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("██░"),
        )
        .with_message("Downloading posts");

    posts
        .into_par_iter()
        .progress_with(progress_bar.clone())
        .for_each(|post| post.download(&client, args).unwrap_or_default());

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
            ProgressStyle::with_template(
                "{msg} {percent}% {wide_bar:0.cyan/blue} ({pos}/{len} pages)",
            )
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("██░"),
        )
        .with_message("Fetching posts");

    let posts: Vec<Post> = (1..=pages_amount)
        .into_par_iter()
        .progress_with(progress_bar)
        .flat_map(|page| get_posts_from_page(&encoded_tags, page, client).unwrap_or_default())
        .collect();

    posts
}

fn get_posts_from_page(encoded_tags: &str, page: u64, client: &Client) -> Result<Vec<Post>> {
    let mut query =
        format!("https://danbooru.donmai.us/posts.json?page={page}&tags={encoded_tags}")
            + "&limit=200&only=rating,file_url,id,score,file_ext,large_file_url";

    if env::var("DANBOORU_LOGIN").is_ok() && env::var("DANBOORU_API_KEY").is_ok() {
        query += &format!(
            "&login={}&api_key={}",
            env::var("DANBOORU_LOGIN")?,
            env::var("DANBOORU_API_KEY")?
        );
    }

    let response = client
        .get(&query)
        .header("Accept", "application/json")
        .send()?;

    let posts: Vec<Post> = serde_json::from_reader(response)?;

    Ok(posts)
}

fn get_total_pages(tags: &[String], client: &Client) -> Result<u64> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let mut query = format!("https://danbooru.donmai.us/posts?tags={encoded_tags}&limit=200");

    if env::var("DANBOORU_LOGIN").is_ok() && env::var("DANBOORU_API_KEY").is_ok() {
        query += &format!(
            "&login={}&api_key={}",
            env::var("DANBOORU_LOGIN")?,
            env::var("DANBOORU_API_KEY")?
        );
    }

    let response = client.get(&query).header("Accept", "text/html").send()?;

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
