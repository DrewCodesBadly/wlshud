use gtk4::{Box, Frame, Widget, glib::object::IsA, prelude::BoxExt};

use crate::shortcuts::ShortcutsDisplay;

pub fn build_main_widgets(shortcuts_display: &ShortcutsDisplay) -> impl IsA<Widget> {
    let outer_box = Box::builder()
        .orientation(gtk4::Orientation::Vertical)
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
    let notes_box = Frame::builder().width_request(400).build();

    // populate two inner rows
    top_row.append(&left_bar);
    top_row.append(shortcuts_display.box_widget());
    top_row.append(&notes_box);

    bottom_row.append(&media_box);
    bottom_row.append(&bottom_bar);

    outer_box
}
