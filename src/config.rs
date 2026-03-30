pub struct ConfigData {
    root_shortcut_node: ShortcutNode,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            root_shortcut_node: ShortcutNode {
                character: 'r',
                exec: None,
                // children: Vec::new(),
                children: vec![
                    ShortcutNode {
                        character: 't',
                        exec: None,
                        children: Vec::new(),
                        icon: None,
                    },
                    ShortcutNode {
                        character: 's',
                        exec: Some(vec!["steam".to_string()]),
                        children: Vec::new(),
                        icon: Some("steam".to_string()),
                    },
                ],
                icon: None,
            },
        }
    }
}

impl ConfigData {
    pub fn root_shortcut_node(&self) -> &ShortcutNode {
        &self.root_shortcut_node
    }
}

#[derive(Clone)]
pub struct ShortcutNode {
    pub character: char,
    pub exec: Option<Vec<String>>,
    pub children: Vec<ShortcutNode>,
    pub icon: Option<String>,
}
