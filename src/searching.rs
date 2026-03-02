use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    mem::take,
    path::PathBuf,
    result,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};

use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use rust_fuzzy_search::fuzzy_search_best_n;
use skia_safe::{Image, graphics::resource_cache_total_bytes_limit};

const MAX_SEARCH_RESULTS: usize = 10;
const MIN_RESULT_THRESHOLD: f32 = 0.1;

pub type SearchResults = Vec<SearchResult>;

#[derive(Clone)]
pub struct SearchResult {
    pub icon_path: Option<String>,
    pub name: String,
    pub location: PathBuf,
    pub execute_command: String,
}

struct AppDatabase {
    // Hash map of app names to the full search result w/info
    apps: HashMap<String, SearchResult>,
}

impl AppDatabase {
    fn new() -> Self {
        let locales = get_languages_from_env();
        let entries = desktop_entries(&locales);
        let apps_list = entries.iter().filter_map(|e| {
            if let Some(exec) = e.exec() {
                let name = e.name(&locales).map(|c| c.to_string()).unwrap_or(
                    e.generic_name(&locales)
                        .map(|c| c.to_string())
                        .unwrap_or(e.appid.to_string()),
                );
                Some((
                    name.to_lowercase().clone(),
                    SearchResult {
                        name,
                        icon_path: e.icon().map(|s| s.to_string()),
                        location: e.path.clone(),
                        execute_command: exec.to_string(),
                    },
                ))
            } else {
                None
            }
        });
        let apps = HashMap::from_iter(apps_list);
        AppDatabase { apps }
    }
}

pub fn searching_thread(results_send: Sender<SearchResults>, query_recv: Receiver<String>) {
    let apps = AppDatabase::new();
    let app_names = apps.apps.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
    let image_cache: HashMap<String, Image> = HashMap::new();

    // Start main loop which reads search requests and updates the request output
    for query in query_recv.iter() {
        // Special case for files
        let mut search_results = SearchResults::new();
        if query.starts_with('/') || query.starts_with('~') {
        } else {
            let results = fuzzy_search_best_n(&query, app_names.as_slice(), MAX_SEARCH_RESULTS);
            for result in results {
                // should be a guaranteed success
                if let Some(app) = apps.apps.get(result.0) {
                    search_results.push(app.clone());
                }
            }
        }

        let _ = results_send.send(search_results);
    }
}
