use std::{
    default, fs,
    io::{self, Write},
    process::Command,
};

use directories::ProjectDirs;
use gtk4::{
    ApplicationWindow, Box, CssProvider, EventControllerKey, Frame, IconTheme, Image, Label,
    ListBox, ListBoxRow, Overlay, ScrolledWindow, SearchEntry, Widget,
    gdk::{
        Display, Key,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::{
        ActionEntry, SimpleActionGroup,
        prelude::{ActionGroupExt, ActionMapExtManual, ApplicationExt, ApplicationExtManual},
        resources_register_include,
    },
    glib::{
        VariantTy, clone,
        object::{CastNone, IsA},
        user_config_dir,
        variant::ToVariant,
    },
    prelude::{
        BoxExt, FrameExt, GtkApplicationExt, GtkWindowExt, ListBoxRowExt, NativeExt, RootExt,
        WidgetExt,
    },
    subclass::dialog,
};
use gtk4::{glib, prelude::EditableExt};
use gtk4_layer_shell::LayerShell;
use libadwaita::{
    Application, CallbackAnimationTarget, Easing, TimedAnimation, prelude::AnimationExt,
};

use crate::{
    actions::build_actions,
    config::{ConfigData, css_file_path},
    searching::get_file_search_entries,
    shortcuts::ShortcutsDisplay,
};
use crate::{
    main_widgets::build_main_widgets,
    searching::{SearchDatabase, SearchResults, build_search_results},
};

mod actions;
mod config;
mod main_widgets;
mod searching;
mod shortcuts;

const APP_MARGIN: i32 = 32;
const APP_ID: &str = "com.DrewCodesBadly.wlshud";
const DEFAULT_CSS_STRING: &str = include_str!("nonrust/default_style.css");

fn main() -> glib::ExitCode {
    let _ = resources_register_include!("wlshud.gresource");
    let app = libadwaita::Application::builder()
        .application_id(APP_ID)
        .build();
    // startup tasks
    app.connect_startup(|_| {
        let provider = CssProvider::new();

        // Handle loading CSS
        let path = css_file_path();
        if let Ok(css) = fs::read_to_string(&path) {
            provider.load_from_data(&css);
        } else {
            let _ = fs::write(path, DEFAULT_CSS_STRING);
            provider.load_from_data(DEFAULT_CSS_STRING);
        }

        // load icons
        let display = Display::default().expect("No display connected.");
        let theme = IconTheme::for_display(&display);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        theme.add_resource_path("/wlshud/icons");
    });
    app.connect_activate(activate);

    // Set binds
    app.set_accels_for_action("wlshud.close", &["Escape"]);

    app.run();

    glib::ExitCode::SUCCESS
}

fn activate(app: &Application) {
    let search_database = SearchDatabase::new();
    let config = ConfigData::default();
    let shortcuts_display = ShortcutsDisplay::new(config.shortcuts_list());

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
        .placeholder_text("Search for applications...")
        .build();

    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_bottom(APP_MARGIN)
        .margin_top(APP_MARGIN)
        .margin_end(APP_MARGIN)
        .margin_start(APP_MARGIN)
        .build();
    outer_box.append(&entry);
    let dialog_overlay = Overlay::builder().child(&outer_box).build();
    let search_results_window = ScrolledWindow::builder().vexpand(true).build();

    let default_box = build_main_widgets(&shortcuts_display);
    outer_box.append(&default_box);

    // TODO: check this, figure out how it works
    // Send key presses to the shortcuts display to trigger shortcuts.
    let key_controller = EventControllerKey::builder().build();
    key_controller.connect_key_pressed(clone!(
        #[strong]
        entry,
        move |_, key, _, _| {
            // Do not handle events if the search entry currently has focus.
            let t = entry.text();
            if key == Key::Tab && (t.starts_with('/') || t.starts_with('~')) {
                let entries = get_file_search_entries(&t);
                if let Some(e) = entries.first() {
                    let mut new_t = e
                        .location
                        .to_str()
                        .expect("cannot convert path to string")
                        .to_owned();
                    if !e.location.is_file() {
                        new_t.push('/');
                    }
                    entry.set_text(&new_t);
                    entry.set_position(new_t.len() as i32);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            } else if entry.has_focus() {
                glib::Propagation::Proceed
            } else if let Some(char) = key.to_unicode() {
                if shortcuts_display.handle_key_pressed(char) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            } else {
                glib::Propagation::Proceed
            }
        }
    ));
    window.add_controller(key_controller);

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
        .duration(250)
        .build();

    // Actions
    let actions = SimpleActionGroup::new();
    actions.add_action_entries(build_actions(&window, &start_fade, &dialog_overlay));
    window.insert_action_group("wlshud", Some(&actions));

    // Connect search bar to input handling
    entry.connect_text_notify(clone!(
        #[strong]
        search_database,
        #[strong]
        search_results_window,
        #[strong]
        default_box,
        #[weak]
        outer_box,
        move |entry| {
            if entry.text().is_empty() {
                // the has focus check here is to make sure this doesn't rebuild and
                // break the focus when space is pressed to trigger shortcuts
                if entry.focus_child().is_some() {
                    // should always be true
                    if let Some(last_child) = outer_box.last_child() {
                        outer_box.remove(&last_child);
                    }
                    outer_box.append(&default_box);
                }
            } else if entry.text().starts_with(' ') {
                entry.set_text("");
                default_box.grab_focus();
            } else {
                let results = search_database.search(&entry.text());
                let results_display = build_search_results(results);
                // should always be true
                if let Some(last_child) = outer_box.last_child() {
                    outer_box.remove(&last_child);
                }
                search_results_window.set_child(Some(&results_display));
                outer_box.append(&search_results_window);
            }
        }
    ));

    // `activate` means when the user presses enter
    entry.connect_activate(clone!(
        #[weak]
        search_results_window,
        move |entry| {
            let t = entry.text();
            // hacky workaround to open directories when you have the ending /
            // since it makes tab autocomplete work as expected and kinda annoyed me
            if t.ends_with('/') && (t.starts_with('/') || t.starts_with('~')) {
                let _ = Command::new("xdg-open").arg(t).spawn();
                let _ = entry.activate_action("wlshud.close", None);
            } else if let Some(c) = search_results_window.child() {
                // there's a GtkViewport in between these for some reason
                c.first_child()
                    .and_downcast::<ListBox>()
                    .and_then(|b| b.row_at_index(0))
                    .inspect(|r| {
                        r.activate();
                    });
            }
        }
    ));

    // close when unfocused
    window.connect_is_active_notify(|window| {
        if !window.is_active() {
            let _ = <ApplicationWindow as WidgetExt>::activate_action(window, "wlshud.close", None);
        }
    });
    window.set_child(Some(&dialog_overlay));
    window.show();

    // play starting animation
    start_fade.play();
}

pub fn icon_from_name(icon_name: &str) -> Image {
    // TODO: less stupid way of doing this? I think it only needs to be / but just to be safe.
    // would also be nice if this worked on other platforms as a future-proof thing
    if icon_name.starts_with('/') || icon_name.starts_with('~') {
        Image::from_file(icon_name)
    } else {
        Image::from_icon_name(&icon_name)
    }
}
