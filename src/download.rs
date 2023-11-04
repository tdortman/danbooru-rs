use std::{env, fs::create_dir_all, io::BufReader, path::PathBuf, process};

use anyhow::{anyhow, bail, Result};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use urlencoding::encode;

use crate::{args::DownloadCommand, post::Post};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const POSTS_PER_PAGE: u64 = 200;

pub fn handle_download(args: &mut DownloadCommand) -> Result<()> {
    let client = Client::builder()
        .user_agent(format!("{PKG_NAME}/{VERSION}"))
        .build()
        .map_err(|_| anyhow!("Failed to build request client"))?;

    let total_pages = get_total_pages(&args.tags, &client)?;

    if create_dir_all(&args.save_location).is_err() {
        println!(
            "Failed to create folder {:?}",
            env::current_dir()
                .unwrap_or(PathBuf::from("."))
                .join(&args.save_location)
        );
        println!("Creating folder named \"output\" in current directory instead");
        args.save_location = PathBuf::from("output");
    }

    let posts = fetch_posts(&args.tags, total_pages, &client, args);

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

    Ok(())
}

/// Fetches all posts that contain all the given tags
///
/// # Arguments
/// * `tags` - The tags to search for
/// * `pages_amount` - The amount of pages to fetch, from [`get_total_pages`]
/// * `client` - The reqwest client to use
/// * `args` - The CLI arguments used to run the program
fn fetch_posts(
    tags: &[String],
    pages_amount: u64,
    client: &Client,
    args: &DownloadCommand,
) -> Vec<Post> {
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
        .flat_map(|page| get_posts_from_page(&encoded_tags, page, client, args).unwrap_or_default())
        .collect();

    if posts.is_empty() {
        eprintln!("No posts found!");
        process::exit(1);
    }

    posts
}

/// Gets all posts from specified page that contain all the given tags
/// # Arguments
/// * `encoded_tags` - The tags to search for (url encoded)
/// * `page` - The page number to fetch posts from
/// * `client` - The reqwest client to use
/// * `args` - The CLI arguments used to run the program
fn get_posts_from_page(
    encoded_tags: &str,
    page: u64,
    client: &Client,
    args: &DownloadCommand,
) -> Result<Vec<Post>> {
    let mut query =
        format!("https://danbooru.donmai.us/posts.json?page={page}&tags={encoded_tags}&limit={POSTS_PER_PAGE}")
              + "&only=rating,file_url,id,score,file_ext,large_file_url";

    if let (Ok(login), Ok(api_key)) = (env::var("DANBOORU_LOGIN"), env::var("DANBOORU_API_KEY")) {
        query.push_str(&format!("&login={login}&api_key={api_key}"));
    }

    let response = client
        .get(&query)
        .header("Accept", "application/json")
        .send()?;

    let posts: Vec<Post> = serde_json::from_reader(BufReader::new(response))?;

    #[rustfmt::skip]
    let posts = posts
        .into_iter()
        .filter(|post| {
            !(
                post.rating == 's' && args.exclude_sensitive
             || post.rating == 'q' && args.exclude_questionable
             || post.rating == 'e' && args.exclude_explicit
             || post.rating == 'g' && args.exclude_general
            )
        })
        .collect();

    Ok(posts)
}

/// Makes a request to the danbooru api to get the total amount of pages
/// of that contain all the given tags
///
/// # Arguments
/// * `tags` - The tags to search for
/// * `client` - The reqwest client to use
fn get_total_pages(tags: &[String], client: &Client) -> Result<u64> {
    let encoded_tags = tags
        .iter()
        .map(|x| encode(x).into_owned())
        .collect::<Vec<String>>()
        .join("+");

    let mut query =
        format!("https://danbooru.donmai.us/posts?tags={encoded_tags}&limit={POSTS_PER_PAGE}");

    if let (Ok(login), Ok(api_key)) = (env::var("DANBOORU_LOGIN"), env::var("DANBOORU_API_KEY")) {
        query.push_str(&format!("&login={login}&api_key={api_key}"));
    }

    let Ok(response) = client.get(&query).header("Accept", "text/html").send() else {
        bail!("Failed to make request to danbooru");
    };

    let Ok(text) = response.text() else {
        bail!("Failed to get response text");
    };

    let document = Html::parse_document(&text);

    let Ok(no_posts_selector) = Selector::parse("#posts > div > p") else {
        bail!("Failed to parse post selector");
    };

    if document.select(&no_posts_selector).count() != 0 {
        bail!("No results found for tags: {tags:?}");
    }

    let Ok(pagination_selector) = Selector::parse(".paginator-page.desktop-only") else {
        bail!("Failed to parse pagination selector");
    };

    let amount: u64 = document
        .select(&pagination_selector)
        .last()
        .and_then(|x| x.inner_html().parse().ok())
        .unwrap_or(1);

    Ok(amount)
}
