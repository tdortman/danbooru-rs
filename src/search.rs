use crate::args::SearchCommand;

pub fn handle_search(args: &SearchCommand) {
    println!("{args:#?}");
}

// https://danbooru.donmai.us/tags.json?search[name_ilike]=*annie*&search[order]=count&search[post_count]>0&limit=10
