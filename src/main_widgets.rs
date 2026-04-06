use std::{collections::HashMap, fs, hash::Hash, process::Command, thread, time::Duration};

use gtk4::{
    Box, Button, Frame, Image, Label, Overlay, Picture, ScrolledWindow, TextIter, TextView, Widget,
    builders::ScrolledWindowBuilder,
    gio::{self, spawn_blocking},
    glib::{self, clone, ffi::g_main_context_wait, object::IsA, spawn_future_local},
    prelude::{BoxExt, ButtonExt, TextBufferExt, TextViewExt, WidgetExt},
};

use crate::{config::notes_file_path, icon_from_name, shortcuts::ShortcutsDisplay};

// currently set to 250 as that is the length of the close animation
const NOTES_BOX_SAVE_DELAY: Duration = Duration::from_millis(250);
// this system SUCKS what the hell gtk4. i find it hard to believe there isn't a better way to do this
const MAX_LABEL_SIZE: i32 = 24;

pub fn build_main_widgets(shortcuts_display: &ShortcutsDisplay) -> impl IsA<Widget> {
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        // .spacing(16)
        .margin_top(16)
        .focusable(true)
        .build();
    let left_side_box = Box::builder()
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
    left_side_box.append(&top_row);
    left_side_box.append(&bottom_row);
    outer_box.append(&left_side_box);

    let left_bar_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_end(16)
        .margin_start(16)
        .build();
    let left_bar = Frame::builder()
        .child(&left_bar_box)
        .vexpand(false)
        .valign(gtk4::Align::Start)
        .build();
    let new_shortcut_button = Button::builder()
        .icon_name("editor-symbolic")
        .width_request(48)
        .height_request(48)
        .action_name("wlshud.new-command-shortcut")
        .build();
    let delete_shortcut_button = Button::builder()
        .icon_name("user-trash-symbolic")
        .width_request(48)
        .height_request(48)
        .action_name("wlshud.remove-shortcuts")
        .build();
    left_bar_box.append(&new_shortcut_button);
    left_bar_box.append(&delete_shortcut_button);

    let media_box = build_media_box();

    let notes_view = build_notes_box();
    let notes_box = Frame::builder().child(&notes_view).build();

    // populate two inner rows
    top_row.append(&left_bar);
    top_row.append(shortcuts_display.box_widget());
    // top_row.append(&notes_box);
    outer_box.append(&notes_box);

    bottom_row.append(&media_box);
    // bottom_row.append(&bottom_bar);

    outer_box
}

fn build_notes_box() -> impl IsA<Widget> {
    let notes_box = TextView::builder().vexpand(true).build();
    let buffer = notes_box.buffer();
    if let Ok(s) = fs::read_to_string(notes_file_path()) {
        buffer.set_text(&s);
    }
    let notes_box_scroll_container = ScrolledWindow::builder()
        .width_request(400)
        .vexpand(true)
        .child(&notes_box)
        .build();

    // Async channel to save on a timer to avoid spamming the disk.
    let (sender, receiver) = async_channel::bounded(1);
    buffer.connect_text_notify(move |_| {
        // notifies the receiver that the text was edited.
        let _ = sender.try_send(());
    });

    spawn_future_local(clone!(
        #[weak]
        buffer,
        async move {
            while let Ok(()) = receiver.recv().await {
                glib::timeout_future(NOTES_BOX_SAVE_DELAY).await;
                if receiver.is_empty() {
                    let s = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                    let _ = fs::write(notes_file_path(), s);
                }
            }
        }
    ));

    notes_box_scroll_container
}

fn build_media_box() -> impl IsA<Widget> {
    let img = Image::builder()
        .valign(gtk4::Align::Center)
        .icon_name("speaker-0-symbolic")
        .height_request(128)
        .width_request(128)
        .pixel_size(128)
        .icon_size(gtk4::IconSize::Large)
        .build();
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(16)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();
    let right_side_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .spacing(8)
        .build();

    let controls_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .homogeneous(true)
        .spacing(16)
        .build();
    let play_pause_btn = Button::builder().icon_name("play-symbolic").build();
    let skip_fwd_btn = Button::builder()
        .icon_name("skip-forward-large-symbolic")
        .build();
    let skip_bck_btn = Button::builder()
        .icon_name("skip-backward-large-symbolic")
        .build();
    controls_box.append(&skip_bck_btn);
    controls_box.append(&play_pause_btn);
    controls_box.append(&skip_fwd_btn);

    let title_labels_box = Box::builder()
        .spacing(4)
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .build();
    let title_label = Label::builder()
        .label("No media playing")
        .overflow(gtk4::Overflow::Hidden)
        .max_width_chars(MAX_LABEL_SIZE)
        .build();
    let artist_label = Label::builder()
        .label("")
        .max_width_chars(MAX_LABEL_SIZE)
        .build();
    title_labels_box.append(&title_label);
    title_labels_box.append(&artist_label);
    right_side_box.append(&title_labels_box);
    right_side_box.append(&controls_box);
    outer_box.append(&img);
    outer_box.append(&right_side_box);

    let frame = Frame::builder()
        .width_request(400)
        .height_request(300)
        .child(&outer_box)
        .build();

    // Periodically probes playerctl to find track metadata
    glib::spawn_future_local(clone!(
        #[strong]
        img,
        #[strong]
        play_pause_btn,
        #[strong]
        title_label,
        #[strong]
        artist_label,
        async move {
            loop {
                let playing_result = Command::new("playerctl").arg("status").output();
                if let Ok(playing_res) = playing_result {
                    let string_output = String::from_utf8(playing_res.stdout);
                    match string_output {
                        Ok(s) => {
                            // "Playing" or "Paused"
                            if s.starts_with("P") {
                                let metadata = fetch_playerctl_metadata();
                                title_label.set_text(
                                    metadata
                                        .get("title")
                                        // stupid jank strings
                                        .unwrap_or(&"Untitled".to_owned()),
                                );
                                if let Some(album) = metadata.get("album").filter(|s| !s.is_empty())
                                {
                                    artist_label.set_text(&format!(
                                        "{} - {}",
                                        metadata
                                            .get("artist")
                                            .unwrap_or(&"Unknown Artist".to_owned()),
                                        album
                                    ));
                                } else {
                                    artist_label.set_text(
                                        metadata
                                            .get("artist")
                                            .unwrap_or(&"Unknown Artist".to_owned()),
                                    );
                                }
                                // gtk won't handle URLs so we do this instead. works with firefox media player so hey
                                if let Some(url) = metadata.get("artUrl")
                                    && url.starts_with("file://")
                                {
                                    if url.starts_with("file://") {
                                        img.set_from_file(Some(&url[7..]));
                                    }
                                } else {
                                    img.set_icon_name(Some("music-note-single-symbolic"));
                                }

                                // Finally handle playing or paused
                                // 2nd == 'l' --> means the word is "Playing" not "Paused"
                                if s.chars().nth(1).filter(|c| *c == 'l').is_some() {
                                    play_pause_btn.set_icon_name("pause-symbolic");
                                } else {
                                    play_pause_btn.set_icon_name("play-symbolic");
                                }
                            } else {
                                title_label.set_text("No media playing");
                                artist_label.set_text("");
                                img.set_icon_name(Some("speaker-0-symbolic"));
                            }
                        }
                        Err(e) => {
                            title_label.set_text("Error fetching media players");
                            artist_label.set_text(&e.to_string());
                            img.set_icon_name(Some("cross-large-symbolic"));
                        }
                    }
                } else {
                    title_label.set_text("Cannot run playerctl");
                    artist_label.set_text("please install playerctl to display media!");
                    img.set_icon_name(Some("cross-large-symbolic"));
                }

                // Run this loop once per second
                glib::timeout_future(Duration::from_secs(1)).await;
            }
        }
    ));

    // Actions for the media player buttons
    play_pause_btn.connect_clicked(|btn| {
        if btn
            .icon_name()
            .expect("should always be set")
            .starts_with("pl")
        {
            btn.set_icon_name("pause-symbolic");
            let _ = Command::new("playerctl").arg("play").spawn();
        } else {
            btn.set_icon_name("play-symbolic");
            let _ = Command::new("playerctl").arg("pause").spawn();
        }
    });
    skip_bck_btn.connect_clicked(|_| {
        let _ = Command::new("playerctl").arg("previous").spawn();
    });
    skip_fwd_btn.connect_clicked(|_| {
        let _ = Command::new("playerctl").arg("next").spawn();
    });

    frame
}

fn fetch_playerctl_metadata() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let metadata_result = Command::new("playerctl").arg("metadata").output();
    if let Ok(output) = metadata_result {
        if let Ok(s) = String::from_utf8(output.stdout) {
            for (k, v) in s.lines().map(|l| {
                let start_cut = &l[l.find(':').map(|i| i + 1).unwrap_or(0)..];
                // if this 'find' fails the output is just bad :/
                let (left, right) = start_cut.split_at(start_cut.find(' ').unwrap_or(0));
                (left.trim().to_owned(), right.trim().to_owned())
            }) {
                map.insert(k, v);
            }
        }
    }

    map
}
