use std::{fs, thread, time::Duration};

use gtk4::{
    Box, Frame, ScrolledWindow, TextIter, TextView, Widget,
    builders::ScrolledWindowBuilder,
    gio::{self, spawn_blocking},
    glib::{self, clone, ffi::g_main_context_wait, object::IsA, spawn_future_local},
    prelude::{BoxExt, TextBufferExt, TextViewExt},
};
use smithay_client_toolkit::data_device_manager::data_offer::receive;

use crate::{config::notes_file_path, shortcuts::ShortcutsDisplay};

const NOTES_BOX_SAVE_DELAY: Duration = Duration::from_millis(500);

pub fn build_main_widgets(shortcuts_display: &ShortcutsDisplay) -> impl IsA<Widget> {
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .focusable(true)
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

    let media_box = Frame::builder()
        .width_request(300)
        .height_request(200)
        .build();

    let notes_box = build_notes_box();

    // populate two inner rows
    top_row.append(&left_bar);
    top_row.append(shortcuts_display.box_widget());
    top_row.append(&notes_box);

    bottom_row.append(&media_box);
    bottom_row.append(&bottom_bar);

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
