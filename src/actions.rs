use std::process::Command;

use freedesktop_desktop_entry::{DesktopEntry, get_languages_from_env};
use gtk4::{
    ApplicationWindow, Button, Entry, InputHints, Label, Overlay, Separator, Widget,
    gio::{ActionEntry, SimpleActionGroup, prelude::ListModelExtManual},
    glib::{self, VariantTy, clone, object::IsA},
    prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt, WidgetExt},
};
use libadwaita::{Easing, TimedAnimation, ffi::AdwAnimation, prelude::AnimationExt};

use crate::config::{
    ConfigData, ShortcutNode, insert_shortcut_node, load_shortcuts_from_config, save_shortcuts_json,
};

pub fn build_actions(
    window: &gtk4::ApplicationWindow,
    start_fade: &TimedAnimation,
    overlay: &Overlay,
) -> Vec<ActionEntry<SimpleActionGroup>> {
    vec![
        ActionEntry::builder("close")
            .activate(clone!(
                #[weak]
                window,
                #[strong]
                start_fade,
                move |_, _, _| {
                    // stop input
                    window.set_sensitive(false);

                    // start animation
                    start_fade.set_reverse(true);
                    start_fade.set_easing(Easing::EaseInCirc);
                    start_fade.play();

                    // close when done
                    start_fade.connect_done(move |_| {
                        window.close();
                    });
                }
            ))
            .build(),
        ActionEntry::builder("exec")
            .parameter_type(Some(VariantTy::STRING_ARRAY))
            .activate(clone!(
                #[weak]
                window,
                move |_, _, parameter| {
                    if let Some(p) = parameter {
                        if let Some(exec_list) = p.get::<Vec<String>>() {
                            if exec_list.len() > 0 {
                                let mut exec = exec_list.iter();
                                let mut cmd = Command::new(&exec.next().unwrap());
                                for arg in exec {
                                    cmd.arg(arg);
                                }
                                let _ = cmd.spawn();
                                let _ = <ApplicationWindow as WidgetExt>::activate_action(
                                    &window,
                                    "wlshud.close",
                                    None,
                                );
                            }
                        }
                    }
                }
            ))
            .build(),
        ActionEntry::builder("create-shortcut")
            .parameter_type(Some(VariantTy::STRING))
            .activate(clone!(
                #[weak]
                overlay,
                move |_, _, parameter| {
                    if let Some(p) = parameter {
                        if let Some(file_path) = p.get::<String>() {
                            // Builds the overlay to show
                            overlay
                                .add_overlay(&build_create_shortcut_overlay(file_path, &overlay));
                        }
                    }
                }
            ))
            .build(),
    ]
}

fn build_overlay_base() -> gtk4::CenterBox {
    let outer_box = gtk4::CenterBox::builder()
        .hexpand(true)
        .vexpand(true)
        .css_classes(["dialog-background"])
        .build();
    outer_box
}

fn build_create_shortcut_overlay(file_path: String, overlay: &Overlay) -> impl IsA<Widget> {
    let base = build_overlay_base();
    let center_box = gtk4::Box::builder()
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .orientation(gtk4::Orientation::Vertical)
        .css_classes(["dialog-box"])
        .spacing(8)
        .opacity(1.0)
        .build();
    base.set_center_widget(Some(&center_box));

    let title = Label::builder()
        .label(format!("Adding shortcut for {}", &file_path))
        .css_classes(["title"])
        .build();
    let path_entry = Entry::builder()
        .placeholder_text("Type out the letters that will activate this shortcut...")
        .build();
    let finish_button = Button::builder()
        .label("Add Shortcut")
        .css_classes(["suggested-action"])
        .build();

    finish_button.connect_clicked(clone!(
        #[weak]
        path_entry,
        #[weak]
        base,
        #[weak]
        overlay,
        move |_| {
            let mut character_path = path_entry.text().trim().to_owned();
            if character_path.len() == 0 {
                return;
            }
            let last_char = character_path.pop().unwrap();
            let locales = get_languages_from_env();
            let parsed_desktop_entry = DesktopEntry::from_path(&file_path, Some(&locales)).ok();
            let to_insert = ShortcutNode {
                character: last_char,
                exec: Some(
                    parsed_desktop_entry
                        .as_ref()
                        .and_then(|e| e.parse_exec().ok())
                        // Defaults to opening a file if the file is not a valid .desktop file
                        .unwrap_or(vec!["xdg-open".to_owned(), file_path.clone()]),
                ),
                children: Vec::new(),
                icon: Some(
                    parsed_desktop_entry
                        .and_then(|e| e.icon().map(|s| s.to_owned()))
                        .unwrap_or("external-link-symbolic".to_owned()),
                ),
            };

            let mut shortcuts: Vec<ShortcutNode> = load_shortcuts_from_config();

            insert_shortcut_node(&mut character_path.chars(), to_insert, &mut shortcuts);

            // save new shortcuts
            save_shortcuts_json(&shortcuts);

            // close overlay
            overlay.remove_overlay(&base);
        }
    ));

    center_box.append(&title);
    center_box.append(&Separator::new(gtk4::Orientation::Horizontal));
    center_box.append(&path_entry);
    center_box.append(&finish_button);

    base
}
