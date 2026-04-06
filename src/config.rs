use std::{
    fs::{create_dir, read_to_string},
    io,
    path::PathBuf,
    str::Chars,
};

use gtk4::glib::user_config_dir;
use json::JsonValue;

pub struct ConfigData {
    shortcuts_list: Vec<ShortcutNode>,
}

impl Default for ConfigData {
    fn default() -> Self {
        // attempts to load config data first, then defaults
        Self {
            shortcuts_list: load_shortcuts_from_config(),
        }
    }
}

impl ConfigData {
    pub fn shortcuts_list(&self) -> &[ShortcutNode] {
        &self.shortcuts_list
    }
}

#[derive(Clone)]
pub struct ShortcutNode {
    pub character: char,
    pub exec: Option<Vec<String>>,
    pub children: Vec<ShortcutNode>,
    pub icon: Option<String>,
}

pub fn load_shortcuts_from_config() -> Vec<ShortcutNode> {
    let shortcuts_path = shortcuts_file_path();
    let shortucts_file = read_to_string(shortcuts_path)
        .and_then(|s| json::parse(&s).map_err(|_| io::Error::other("Failed to parse JSON")));
    if let Ok(parsed) = shortucts_file {
        parse_shortcuts_json(&parsed)
    } else {
        Vec::new()
    }
}

pub fn parse_shortcuts_json(data: &JsonValue) -> Vec<ShortcutNode> {
    let mut vec = Vec::new();
    if data.is_array() {
        for member in data.members() {
            // build node from member
            let exec_data = &member["exec"];
            let exec = if exec_data.is_array() {
                Some(
                    exec_data
                        .members()
                        .map(|s| s.as_str().unwrap_or("").to_owned())
                        .collect(),
                )
            } else {
                None
            };
            let character_data = &member["character"];
            if !character_data.is_string() {
                continue;
            }
            let char_str = character_data.to_string();
            if char_str.is_empty() {
                continue;
            }
            let children = parse_shortcuts_json(&member["children"]);
            let node = ShortcutNode {
                character: char_str.chars().next().unwrap(),
                exec,
                children,
                icon: member["icon"].as_str().map(|s| s.to_owned()),
            };
            vec.push(node);
        }
    }

    vec
}

pub fn save_shortcuts_json(shortcuts: &[ShortcutNode]) {
    let json_data = shortcut_array_to_json(shortcuts);
    let path = shortcuts_file_path();
    let _ = std::fs::write(path, json_data.dump());
}

fn shortcut_array_to_json(shortcuts: &[ShortcutNode]) -> JsonValue {
    let mut arr = json::array![];

    for node in shortcuts {
        let mut obj = json::object! {
            character: node.character.to_string(),
            children: shortcut_array_to_json(&node.children),
        };
        if let Some(icon) = &node.icon {
            obj["icon"] = JsonValue::String(icon.to_owned());
        }
        if let Some(exec) = &node.exec {
            let mut exec_arr = json::array![];
            for cmd in exec {
                let _ = exec_arr.push(JsonValue::String(cmd.to_owned()));
            }
            obj["exec"] = exec_arr;
        }

        let _ = arr.push(obj);
    }

    arr
}

pub fn insert_shortcut_node(
    char_path: &mut Chars,
    to_insert: ShortcutNode,
    into: &mut Vec<ShortcutNode>,
) {
    if let Some(c) = char_path.next() {
        if let Some(n) = into.iter_mut().find(|n| n.character == c) {
            insert_shortcut_node(char_path, to_insert, &mut n.children);
        } else {
            let mut new_node = ShortcutNode {
                character: c,
                exec: None,
                children: Vec::new(),
                icon: None,
            };
            insert_shortcut_node(char_path, to_insert, &mut new_node.children);
            into.push(new_node);
        }
    } else {
        into.push(to_insert);
    }
}

pub fn delete_shortcut_node(char_path: &mut Chars, from: &mut Vec<ShortcutNode>) {
    // Will silently fail if passed an empty Chars
    if let Some(c) = char_path.next() {
        let mut remove_idx = None;
        println!("Running recursive delete, at char {}", c);
        if let Some((idx, n)) = from.iter_mut().enumerate().find(|(_, n)| n.character == c) {
            println!("Found a valid child node");
            let next = char_path.next();
            if let Some(c2) = next {
                delete_shortcut_node_recursion(char_path, &mut n.children, c2);
            } else {
                println!("No remaining, setting remove_idx");
                remove_idx = Some(idx);
            }
        }

        if let Some(idx) = remove_idx {
            println!("Removing index {}", idx);
            from.remove(idx);
        }
    }
}

fn delete_shortcut_node_recursion(
    char_path: &mut Chars,
    from: &mut Vec<ShortcutNode>,
    current_char: char,
) {
    let mut remove_idx = None;
    println!("Running recursive delete, at char {}", current_char);
    if let Some((idx, n)) = from
        .iter_mut()
        .enumerate()
        .find(|(_, n)| n.character == current_char)
    {
        println!("Found a valid child node");
        let next = char_path.next();
        if let Some(c2) = next {
            delete_shortcut_node_recursion(char_path, &mut n.children, c2);
        } else {
            println!("No remaining, setting remove_idx");
            remove_idx = Some(idx);
        }
    }

    if let Some(idx) = remove_idx {
        println!("Removing index {}", idx);
        from.remove(idx);
    }
}

fn wlshud_config_dir() -> PathBuf {
    let mut dir = user_config_dir();
    dir.push("wlshud");
    if !dir.exists() {
        let _ = create_dir(&dir);
    }
    dir
}

pub fn shortcuts_file_path() -> PathBuf {
    let mut dir = wlshud_config_dir();
    dir.push("shortcuts.json");
    dir
}

pub fn notes_file_path() -> PathBuf {
    let mut dir = wlshud_config_dir();
    dir.push("notes.txt");
    dir
}

pub fn css_file_path() -> PathBuf {
    let mut dir = wlshud_config_dir();
    dir.push("style.css");
    dir
}

// this is kinda slow so don't use it if performance is super important
pub fn flatten_shortcuts_list(
    list: &[ShortcutNode],
    previous_chars: &str,
) -> Vec<(String, ShortcutNode)> {
    let mut flat_vec = Vec::new();

    for node in list {
        let new_chars = format!("{}{}", previous_chars, node.character);
        flat_vec.append(&mut flatten_shortcuts_list(&node.children, &new_chars));
        flat_vec.push((new_chars, node.clone()));
    }

    flat_vec.sort_by(|a, b| a.0.cmp(&b.0));

    flat_vec
}
