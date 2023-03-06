use crate::args::DownloadCommand;

pub async fn handle_download(args: DownloadCommand) {
    println!("{args:#?}");
}