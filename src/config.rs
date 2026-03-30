pub struct ConfigData {
    root_shortcut_node: ShortcutNode,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            root_shortcut_node: ShortcutNode {
                character: 'a',
                exec: None,
                children: Vec::new(),
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
