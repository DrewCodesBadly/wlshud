use std::{collections::HashMap, path::PathBuf};

use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use gtk4::{
    Box, Button, Image, Label, ListBox, ListBoxRow, MenuButton, Popover, Widget,
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
        } else {
            let app_names = self.apps.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
            let results = fuzzy_search_best_n(&query, app_names.as_slice(), MAX_SEARCH_RESULTS);
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
        name_label.set_css_classes(&["title"]);
        name_label.set_halign(gtk4::Align::Start);
        let location_label = Label::new(result.location.to_str());
        location_label.set_css_classes(&["subtitle"]);
        location_label.set_halign(gtk4::Align::Start);
        labels_box.append(&name_label);
        labels_box.append(&location_label);

        let icon = if let Some(path) = result.icon_path {
            icon_from_name(&path)
        } else {
            Image::from_icon_name("folder")
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
