use std::cell::RefCell;

use gtk4::{
    Box, Image, Label,
    glib::{self, property::PropertySet, variant::ToVariant},
    prelude::{BoxExt, WidgetExt},
};
use libadwaita::{CallbackAnimationTarget, Easing, TimedAnimation, prelude::AnimationExt};

use crate::{config::ShortcutNode, icon_from_name};

// #[derive(Default)]
pub struct ShortcutsDisplay {
    current_nodes: RefCell<Vec<ShortcutNode>>,
    outer_box: Box,
}

impl Default for ShortcutsDisplay {
    fn default() -> Self {
        Self {
            current_nodes: RefCell::new(Vec::new()),
            outer_box: Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .vexpand(true)
                .spacing(16)
                .margin_top(16)
                .margin_bottom(16)
                .build(),
        }
    }
}

impl ShortcutsDisplay {
    pub fn new(nodes_list: &[ShortcutNode]) -> Self {
        let s = Self {
            current_nodes: RefCell::new(nodes_list.to_owned()),
            ..Default::default()
        };
        let row_1 = build_shortcuts_row(&s.current_nodes.borrow());
        s.box_widget().append(&row_1);

        s
    }
    pub fn handle_key_pressed(&self, key: char) -> bool {
        let cur_nodes = self.current_nodes.borrow();
        let mut swap_node = None;
        for child in cur_nodes.iter() {
            if child.character == key {
                if let Some(exec) = &child.exec {
                    let _ = <Box as WidgetExt>::activate_action(
                        &self.outer_box,
                        "wlshud.exec",
                        Some(&exec.to_variant()),
                    );
                } else if !child.children.is_empty() {
                    // Activate children
                    swap_node = Some(child.clone());
                }
            }
        }

        drop(cur_nodes);
        if let Some(node) = swap_node {
            let row = build_shortcuts_row(&node.children);
            // Start an animation
            let fade_in_target = CallbackAnimationTarget::new(gtk4::glib::clone!(
                #[weak]
                row,
                move |val| {
                    row.set_opacity(val);
                }
            ));
            let fade_in = TimedAnimation::builder()
                .value_from(0.0)
                .value_to(1.0)
                .easing(Easing::EaseOutSine)
                .widget(&row)
                .target(&fade_in_target)
                .duration(150)
                .build();
            self.outer_box.append(&row);
            self.current_nodes.set(node.children.clone());
            fade_in.play();
            true
        } else {
            false
        }
    }

    pub fn box_widget(&self) -> &Box {
        &self.outer_box
    }
}

fn build_shortcuts_row(nodes: &[ShortcutNode]) -> Box {
    let row = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .homogeneous(true)
        .build();

    for child in nodes {
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
        icon.set_icon_size(gtk4::IconSize::Large);

        let label = Label::builder().label(child.character.to_string()).build();

        child_box.append(&icon);
        child_box.append(&label);
        row.append(&child_box);
    }

    row
}
