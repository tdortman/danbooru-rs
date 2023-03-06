use crate::args::SearchCommand;

pub async fn handle_search(args: SearchCommand) {
    println!("{args:#?}");
}