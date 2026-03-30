use gtk4::{
    Image, Label,
    ffi::gtk_widget_add_controller,
    glib::{self, object::IsA},
    prelude::BoxExt,
    subclass::{box_::BoxImpl, widget::WidgetImpl},
};
use libadwaita::subclass::prelude::{ObjectImpl, ObjectSubclass};

use crate::{config::ShortcutNode, icon_from_name};

pub struct ShortcutsDisplay {
    current_node: ShortcutNode,
}

#[glib::object_subclass]
impl ObjectSubclass for ShortcutsDisplay {
    const NAME: &'static str = "WlshudShortcutsDisplay";
    type Type = ShortcutsDisplay;
    type ParentType = gtk4::Box;
    // type Interfaces;
    // type Instance;
    // type Class;
}

impl ObjectImpl for ShortcutsDisplay {}

impl WidgetImpl for ShortcutsDisplay {}

impl BoxImpl for ShortcutsDisplay {}

impl ShortcutsDisplay {
    pub fn new(root_node: ShortcutNode) -> Self {
        Self {
            current_node: root_node,
        }
    }

    // Tries to handle a keypress, returns whether or not the event was handled.
    pub fn handle_key_pressed(&mut self, key: char) -> bool {
        for child in self.current_node.children {
            if child.character == key {
                if let Some(exec) = child.exec {
                    let _ = <Self as WidgetExt>::activate_action(
                        self,
                        "wlshud.exec",
                        Some(exec.to_variant()),
                    );
                } else {
                    // Activate children
                    self.current_node = child;
                    let row = build_shortcuts_row(&self.current_node);
                    self.append(&row);
                }
            }
        }
        false
    }
}

fn build_shortcuts_row(node: &ShortcutNode) -> impl IsA<Widget> {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    for child in node.children {
        let child_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        // build icon
        let icon = if let Some(path) = child.icon {
            icon_from_name(path)
        } else if child.exec.is_some() {
            // TODO: find better icon
            Image::from_icon_name("folder")
        } else {
            Image::from_icon_name("folder")
        };

        let label = Label::builder().label(child.character).build();

        child_box.append(&icon);
        child_box.append(&label);
    }

    row
}
