use crate::args::SearchCommand;

#[allow(clippy::unused_async)]
pub fn handle_search(args: &SearchCommand) {
    println!("{args:#?}");
}