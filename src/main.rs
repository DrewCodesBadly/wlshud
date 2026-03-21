use std::{
    sync::mpsc,
    thread::{self},
    time::Instant,
};

use gtk4::{
    ApplicationWindow, Box, Frame, SearchEntry, Widget,
    gio::{
        ActionEntry, SimpleActionGroup,
        prelude::{ActionMapExtManual, ApplicationExt, ApplicationExtManual},
        spawn_blocking,
    },
    glib::{clone, object::IsA},
    prelude::{BoxExt, GtkApplicationExt, GtkWindowExt, WidgetExt},
};
use gtk4::{glib, prelude::EditableExt};
use gtk4_layer_shell::LayerShell;
use libadwaita::{
    Application, CallbackAnimationTarget, Easing, TimedAnimation, prelude::AnimationExt,
};

use crate::searching::{SearchDatabase, SearchResults};

mod config;
mod searching;

const APP_MARGIN: i32 = 32;

fn main() {
    let app = libadwaita::Application::builder()
        .application_id("com.github.DrewCodesBadly.wlshud")
        .build();
    app.connect_activate(activate);

    // Set binds
    app.set_accels_for_action("wlshud.close", &["Escape"]);

    app.run();
}

fn activate(app: &Application) {
    let search_database = SearchDatabase::new();

    let window = gtk4::ApplicationWindow::new(app);

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    // anchors to all 4 sides (take up whole screen)
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);

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
    let default_box = build_default_box();
    outer_box.append(&default_box);

    // Startup animation
    let opacity_target = CallbackAnimationTarget::new(clone!(
        #[weak]
        window,
        move |v| {
            window.set_opacity(v);
        }
    ));
    let start_fade = TimedAnimation::builder()
        .value_from(0.0)
        .value_to(1.0)
        .widget(&window)
        .target(&opacity_target)
        .easing(Easing::EaseOutCirc)
        .duration(300)
        .build();

    // Actions
    let close_action = ActionEntry::builder("close")
        .activate(clone!(
            #[weak]
            window,
            #[strong]
            start_fade,
            move |_, _, _| {
                start_fade.set_reverse(true);
                start_fade.set_easing(Easing::EaseInCirc);
                start_fade.play();
                start_fade.connect_done(move |_| {
                    window.close();
                });
            }
        ))
        .build();
    let actions = SimpleActionGroup::new();
    actions.add_action_entries([close_action]);
    window.insert_action_group("wlshud", Some(&actions));

    // Connect search bar to input handling
    entry.connect_changed(clone!(
        #[strong]
        search_database,
        #[weak]
        outer_box,
        move |entry| {
            if entry.text().starts_with(' ') {
                entry.set_text("");
                // todo: activate shortcuts
            } else {
                let results = search_database.search(&entry.text());
                let results_display = build_search_results(results);
                // should always be true
                if let Some(last_child) = outer_box.last_child() {
                    outer_box.remove(&last_child);
                    outer_box.append(&results_display);
                }
            }
        }
    ));

    // close when unfocused
    window.connect_is_active_notify(|window| {
        if !window.is_active() {
            let _ = <ApplicationWindow as WidgetExt>::activate_action(window, "wlshud.close", None);
        }
    });
    window.set_child(Some(&outer_box));
    window.show();

    // play starting animation
    start_fade.play();
}

fn build_default_box() -> impl IsA<Widget> {
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
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

    outer_box
}

fn build_search_results(results: SearchResults) -> impl IsA<Widget> {
    Box::new(gtk4::Orientation::Horizontal, 10)
}
