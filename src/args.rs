use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(
    author = "Tilted Toast",
    version = "0.1.0",
    about = "A command line interface for Danbooru"
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Download images via tags", alias = "dl")]
    Download(DownloadCommand),
    #[clap(about = "Search for tags", alias = "s")]
    Search(SearchCommand),
}

#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct DownloadCommand {
    #[clap(
        short = 't',
        long = "tag",
        help = "Tag to search for (can be used multiple times)"
    )]
    pub tags: Vec<String>,
    #[clap(
        short = 'o',
        long = "output",
        default_value = "output",
        help = "Output directory"
    )]
    pub save_location: PathBuf,
    #[clap(
        short = 'g',
        long = "general",
        help = "Exclude tags with the 'general' rating"
    )]
    pub exclude_general: bool,
    #[clap(
        short = 's',
        long = "sensitive",
        help = "Exclude tags with the 'sensitive' rating"
    )]
    pub exclude_sensitive: bool,
    #[clap(
        short = 'q',
        long = "questionable",
        help = "Exclude tags with the 'questionable' rating"
    )]
    pub exclude_questionable: bool,
    #[clap(
        short = 'e',
        long = "explicit",
        help = "Exclude tags with the 'explicit' rating"
    )]
    pub exclude_explicit: bool,
}

#[derive(clap::Args, Debug)]
pub struct SearchCommand {
    #[clap(short = 't', long = "term", help = "The term to search for")]
    pub search_term: String,
}
