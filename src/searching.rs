use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};

use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use skia_safe::Image;

pub type SearchResults = Vec<SearchResult>;

pub struct SearchResult {
    pub icon: Image,
    pub name: String,
    pub location: String,
    pub execute_command: String,
}

struct AppDatabase {}

impl AppDatabase {
    fn new() -> Self {
        let locales = get_languages_from_env();
        let apps = desktop_entries(&locales);
        AppDatabase {}
    }
}

pub fn searching_thread(search_reults: Sender<SearchResults>, query_recv: Receiver<String>) {
    let apps = AppDatabase::new();
    let image_cache: HashMap<String, Image> = HashMap::new();

    // Start main loop which reads search requests and updates the request output
    // TODO: implement
    for query in query_recv.iter() {}
}
