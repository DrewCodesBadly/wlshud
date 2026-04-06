use std::{collections::HashMap, fs, path::PathBuf};

use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use gtk4::{
    Box, Button, Image, Label, ListBox, ListBoxRow, Widget,
    glib::{object::IsA, variant::ToVariant},
    prelude::{BoxExt, ListBoxRowExt, WidgetExt},
};
use rust_fuzzy_search::fuzzy_search_best_n;

use crate::icon_from_name;

// TODO: User-customizable?
const MAX_SEARCH_RESULTS: usize = 20;

pub type SearchResults = Vec<SearchResult>;

#[derive(Clone)]
pub struct SearchResult {
    pub icon_path: Option<String>,
    pub name: String,
    pub location: PathBuf,
    pub execute_command: Vec<String>,
}

#[derive(Clone)]
pub struct SearchDatabase {
    // Hash map of app names to the full search result w/info
    apps: HashMap<String, SearchResult>,
}

impl SearchDatabase {
    pub fn new() -> Self {
        let locales = get_languages_from_env();
        let entries = desktop_entries(&locales);
        let apps_list = entries.iter().filter_map(|e| {
            if let Ok(exec) = e.parse_exec() {
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
                        execute_command: exec,
                    },
                ))
            } else {
                None
            }
        });
        let apps = HashMap::from_iter(apps_list);
        SearchDatabase { apps }
    }

    pub fn search(&self, query: &str) -> SearchResults {
        let mut search_results = SearchResults::new();
        if query.starts_with('/') || query.starts_with('~') {
            let mut maybe_entries = get_file_search_entries(query);
            search_results.append(&mut maybe_entries);
        } else if let Some(q) = query.strip_prefix('>') {
            search_results.push(SearchResult {
                icon_path: Some("terminal-symbolic".to_owned()),
                name: "Run this command from the current working directory".to_owned(),
                location: std::env::current_dir().expect("cannot get working directory"),
                execute_command: q.split(' ').map(|s| s.to_owned()).collect(),
            });
        } else {
            let app_names = self.apps.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
            let lower_search = query.to_ascii_lowercase();
            let results =
                fuzzy_search_best_n(&lower_search, app_names.as_slice(), MAX_SEARCH_RESULTS);
            for result in results {
                // should be a guaranteed success
                if let Some(app) = self.apps.get(result.0) {
                    search_results.push(app.clone());
                }
            }
        }

        search_results
    }
}

pub fn build_search_results(results: SearchResults) -> impl IsA<Widget> {
    let list_box = ListBox::builder()
        .activate_on_single_click(true)
        .selection_mode(gtk4::SelectionMode::Single)
        .show_separators(true)
        .build();

    for result in results {
        let row = ListBoxRow::builder()
            .selectable(true)
            .activatable(true)
            .action_name("wlshud.exec")
            .action_target(&result.execute_command.to_variant())
            .build();
        const ROW_SPACING_MARGIN: i32 = 8;
        let row_contents = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_bottom(ROW_SPACING_MARGIN)
            .margin_top(ROW_SPACING_MARGIN)
            .margin_end(ROW_SPACING_MARGIN)
            .margin_start(ROW_SPACING_MARGIN)
            .vexpand(true)
            .spacing(16)
            .build();
        let labels_box = Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        let name_label = Label::new(Some(&result.name));
        name_label.set_halign(gtk4::Align::Start);
        let file_path = result
            .location
            .to_str()
            .expect("Invalid path from searches");
        let location_label = Label::new(Some(file_path));
        location_label.set_css_classes(&["subtitle"]);
        location_label.set_halign(gtk4::Align::Start);
        labels_box.append(&name_label);
        labels_box.append(&location_label);

        let create_shortcut_button = Button::builder()
            .icon_name("plus-symbolic")
            .action_name("wlshud.create-shortcut")
            .action_target(&file_path.to_variant())
            .halign(gtk4::Align::End)
            .hexpand(true)
            .build();

        let icon = if let Some(path) = result.icon_path {
            icon_from_name(&path)
        } else {
            Image::from_icon_name("external-link-symbolic")
        };
        icon.set_icon_size(gtk4::IconSize::Large);

        row_contents.append(&icon);
        row_contents.append(&labels_box);
        row_contents.append(&create_shortcut_button);

        row.set_child(Some(&row_contents));
        list_box.append(&row);
    }

    list_box
}

pub fn get_file_search_entries(query: &str) -> Vec<SearchResult> {
    let mut maybe_entries = Vec::new();
    let last_slash = query.rfind('/').unwrap_or(0);
    let (mut path_str, mut file_portion) = if last_slash > 0 {
        query.split_at(query.rfind('/').unwrap_or(0))
    } else {
        query.split_at(1)
    };
    if file_portion.starts_with('/') {
        file_portion = &file_portion[1..];
    }
    let new_str = path_str.replacen(
        '~',
        std::env::home_dir()
            .expect("user does not have a home directory")
            .to_str()
            .expect("bad path to home dir"),
        1,
    );
    path_str = &new_str;
    let files = fs::read_dir(path_str);
    if let Ok(iter) = files {
        for entry in iter.flatten() {
            maybe_entries.push(SearchResult {
                icon_path: entry.file_type().ok().and_then(|t| {
                    if t.is_dir() {
                        Some("folder".to_owned())
                    } else {
                        None
                    }
                }),
                name: entry
                    .file_name()
                    .into_string()
                    .unwrap_or("Corrupt File".to_owned()),
                location: entry.path(),
                execute_command: vec![
                    "xdg-open".to_owned(),
                    format!("{}/{}", path_str, entry.file_name().display()),
                ],
            });
        }
        maybe_entries = maybe_entries
            .iter()
            .filter(|e| e.name.starts_with(file_portion))
            .cloned()
            .collect();
        maybe_entries.sort_by(|a, b| a.name.cmp(&b.name));
    }
    maybe_entries
}
