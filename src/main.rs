use std::{
    sync::mpsc,
    thread::{self},
};

use gtk4::{
    Application, ApplicationWindow, Box, CenterBox, Frame, Label, MenuButton, SearchBar,
    SearchEntry, Window,
    builders::SearchBarBuilder,
    gio::{
        ActionEntry, SimpleActionGroup,
        prelude::{ActionGroupExt, ActionMapExtManual, ApplicationExt, ApplicationExtManual},
    },
    glib::{self, clone},
    prelude::{BoxExt, GtkApplicationExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;

use crate::searching::searching_thread;

mod config;
mod searching;

const APP_MARGIN: i32 = 32;

fn main() {
    // Spawn searcher thread
    // let search_results = SearchResults::default();
    let (results_send, results_recv) = mpsc::channel();
    let (query_send, query_recv) = mpsc::channel();

    // Start searcher thread
    let _searching_thread = thread::spawn(move || searching_thread(results_send, query_recv));

    let app = gtk4::Application::builder()
        .application_id("com.github.DrewCodesBadly.wlshud")
        .build();
    app.connect_activate(activate);

    // Set binds
    app.set_accels_for_action("wlshud.close", &["Escape"]);

    app.run();
}

fn activate(app: &Application) {
    let window = gtk4::ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    // anchors to all 4 sides (take up whole screen)
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);

    // Actions
    let close_action = ActionEntry::builder("close")
        .activate(clone!(
            #[weak]
            window,
            move |_, _, _| {
                window.close();
            }
        ))
        .build();
    let actions = SimpleActionGroup::new();
    actions.add_action_entries([close_action]);
    window.insert_action_group("wlshud", Some(&actions));

    let entry = SearchEntry::builder()
        .hexpand(true)
        .valign(gtk4::Align::Start)
        .build();
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_bottom(APP_MARGIN)
        .margin_top(APP_MARGIN)
        .margin_end(APP_MARGIN)
        .margin_start(APP_MARGIN)
        .build();
    outer_box.append(&entry);
    // 2 rows inside
    let top_row = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .vexpand(true)
        .hexpand(true)
        .build();
    let bottom_row = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .build();
    outer_box.append(&top_row);
    outer_box.append(&bottom_row);

    let left_bar = Frame::builder().width_request(64).build();
    let bottom_bar = Frame::builder()
        // .orientation(gtk4::Orientation::Horizontal)
        .height_request(64)
        .hexpand(true)
        .build();
    let shortcuts_area = Frame::builder().hexpand(true).build();

    let media_box = Frame::builder()
        .width_request(300)
        .height_request(200)
        .build();
    let notes_box = Frame::builder().width_request(400).build();

    // populate two inner rows
    top_row.append(&left_bar);
    top_row.append(&shortcuts_area);
    top_row.append(&notes_box);

    bottom_row.append(&media_box);
    bottom_row.append(&bottom_bar);

    // close when unfocused
    window.connect_is_active_notify(|window| {
        if !window.is_active() {
            let _ = <ApplicationWindow as WidgetExt>::activate_action(window, "wlshud.close", None);
        }
    });
    window.set_child(Some(&outer_box));
    window.show();
}
