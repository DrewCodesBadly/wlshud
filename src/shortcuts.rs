use std::cell::{Cell, RefCell};

use gtk4::{
    Box, Image, Label, Widget,
    glib::{self, Object, object::IsA, property::PropertySet, variant::ToVariant},
    prelude::{BoxExt, WidgetExt},
    subclass::{
        box_::BoxImpl,
        prelude::{ActionableImpl, TextBufferImpl},
        widget::WidgetImpl,
    },
};
use libadwaita::subclass::prelude::{ObjectImpl, ObjectSubclass};

use crate::{config::ShortcutNode, icon_from_name};

// #[derive(Default)]
pub struct ShortcutsDisplay {
    current_node: RefCell<ShortcutNode>,
    outer_box: Box,
}

impl Default for ShortcutsDisplay {
    fn default() -> Self {
        Self {
            current_node: RefCell::new(ShortcutNode {
                character: 's',
                exec: None,
                children: Vec::new(),
                icon: None,
            }),
            outer_box: Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build(),
        }
    }
}

impl ShortcutsDisplay {
    pub fn new(root_node: ShortcutNode) -> Self {
        let row_1 = build_shortcuts_row(&root_node);
        let s = Self {
            current_node: RefCell::new(root_node),
            ..Default::default()
        };
        s.box_widget().append(&row_1);

        s
    }
    pub fn handle_key_pressed(&self, key: char) -> bool {
        let cur_node = self.current_node.borrow();
        let mut swap_node = None;
        for child in &cur_node.children {
            if child.character == key {
                if let Some(exec) = &child.exec {
                    let _ = <Box as WidgetExt>::activate_action(
                        &self.outer_box,
                        "wlshud.exec",
                        Some(&exec.to_variant()),
                    );
                } else {
                    // Activate children
                    swap_node = Some(child.clone());
                    let row = build_shortcuts_row(&cur_node);
                    self.outer_box.append(&row);
                }
            }
        }

        drop(cur_node);
        if let Some(node) = swap_node {
            self.current_node.set(node);
            true
        } else {
            false
        }
    }

    pub fn box_widget(&self) -> &Box {
        &self.outer_box
    }
}

fn build_shortcuts_row(node: &ShortcutNode) -> impl IsA<Widget> {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    for child in node.children.iter().by_ref() {
        let child_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        // build icon
        let icon = if let Some(path) = &child.icon {
            icon_from_name(path)
        } else if child.exec.is_some() {
            // TODO: find better icon
            Image::from_icon_name("folder")
        } else {
            Image::from_icon_name("folder")
        };

        let label = Label::builder().label(child.character.to_string()).build();

        child_box.append(&icon);
        child_box.append(&label);
    }

    row
}
