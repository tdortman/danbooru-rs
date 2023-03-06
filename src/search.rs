use crate::args::SearchCommand;

#[allow(clippy::unused_async)]
pub async fn handle_search(args: SearchCommand) {
    println!("{args:#?}");
}