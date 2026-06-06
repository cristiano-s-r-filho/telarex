use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use crate::events::UIAction;
use telarex_core::config::schema::KeymapConfig;

pub struct KeyMapper {
    pub global: HashMap<KeyEvent, UIAction>,
    pub editor: HashMap<KeyEvent, UIAction>,
    pub explorer: HashMap<KeyEvent, UIAction>,
}

impl KeyMapper {
    pub fn from_config(config: &KeymapConfig) -> Self {
        Self {
            global: parse_map(&config.global),
            editor: parse_map(&config.editor_insert), // Use insert map as the base for the unified mode
            explorer: parse_map(&config.explorer),
        }
    }

    pub fn resolve(&self, key: KeyEvent, mode: &str, component: Option<&str>) -> Option<UIAction> {
        // DUPLICATION FIX: Only resolve on Press or Repeat
        if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat { return None; }

        // 1. Window sub-mode (Ctrl+W)
        if mode == "window" {
            return match (key.code, key.modifiers) {
                (KeyCode::Char('v'), KeyModifiers::NONE) => Some(UIAction::SplitVertical),
                (KeyCode::Char('s'), KeyModifiers::NONE) => Some(UIAction::SplitHorizontal),
                (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, KeyModifiers::NONE) => Some(UIAction::FocusLeft),
                (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => Some(UIAction::FocusRight),
                (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => Some(UIAction::FocusUp),
                (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => Some(UIAction::FocusDown),
                (KeyCode::Char('c'), KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => Some(UIAction::ClosePane),
                _ => Some(UIAction::ExitMode), // Any other key exits window mode
            };
        }

        // 2. Component Layer
        if let Some(comp) = component {
            match comp {
                "explorer" => {
                    if let Some(action) = self.explorer.get(&key) {
                        return Some(action.clone());
                    }
                }
                "editor" => {
                    if let Some(action) = self.editor.get(&key) {
                        return Some(action.clone());
                    }
                }
                _ => {}
            }
        }

        // 3. Global Layer (Hardcoded overrides)
        if key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('p') => return Some(UIAction::EnterCommandMode),
                KeyCode::Char('f') => return Some(UIAction::EnterSearchMode),
                KeyCode::Char('m') => return Some(UIAction::ToggleMacroPalette),
                KeyCode::Char('t') => return Some(UIAction::NewTab),
                KeyCode::Tab => return Some(UIAction::NextTab),
                KeyCode::Char('w') => return Some(UIAction::EnterWindowMode),
                KeyCode::Char('e') => return Some(UIAction::SwitchFocus),
                KeyCode::Char('b') => return Some(UIAction::ToggleExplorer),
                KeyCode::Char('s') => return Some(UIAction::Core(telarex_core::command::Command::Save)),
                KeyCode::Char('c') => return Some(UIAction::Copy),
                KeyCode::Char('v') => return Some(UIAction::Paste),
                _ => {}
            }
        }

        // 3. Fallback for character keys (Never swallow them)
        if let KeyCode::Char(_) = key.code {
             // AltGr (reported as Ctrl+Alt) should NOT trigger shortcuts
             let is_strictly_control = key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT);
             let is_strictly_alt = key.modifiers.contains(KeyModifiers::ALT) && !key.modifiers.contains(KeyModifiers::CONTROL);
             
             if !is_strictly_control && !is_strictly_alt {
                 return None; // Let the editor handle raw chars
             }
        }

        self.global.get(&key).cloned()
    }
}

fn parse_map(map: &HashMap<String, String>) -> HashMap<KeyEvent, UIAction> {
    let mut parsed = HashMap::new();
    for (key_str, action_str) in map {
        if let Some(key_event) = parse_key_event(key_str) {
            if let Some(action) = parse_action(action_str) {
                parsed.insert(key_event, action);
            }
        }
    }
    parsed
}

fn parse_key_event(s: &str) -> Option<KeyEvent> {
    let lowered = s.to_lowercase();
    let parts: Vec<&str> = lowered.split('-').collect();
    let mut mods = KeyModifiers::NONE;
    let mut code = KeyCode::Null;

    for part in parts {
        match part {
            "ctrl" => mods.insert(KeyModifiers::CONTROL),
            "alt" => mods.insert(KeyModifiers::ALT),
            "shift" => mods.insert(KeyModifiers::SHIFT),
            p if p.len() == 1 => code = KeyCode::Char(p.chars().next().unwrap()),
            "enter" => code = KeyCode::Enter,
            "esc" => code = KeyCode::Esc,
            "tab" => code = KeyCode::Tab,
            "backspace" => code = KeyCode::Backspace,
            "up" => code = KeyCode::Up,
            "down" => code = KeyCode::Down,
            "left" => code = KeyCode::Left,
            "right" => code = KeyCode::Right,
            _ => {}
        }
    }

    if code == KeyCode::Null {
        None
    } else {
        let mut event = KeyEvent::new(code, mods);
        event.kind = KeyEventKind::Press;
        Some(event)
    }
}

fn parse_action(s: &str) -> Option<UIAction> {
    use telarex_core::command::Command;
    match s {
        "Quit" => Some(UIAction::Quit),
        "Save" => Some(UIAction::Core(Command::Save)),
        "OpenFile" => Some(UIAction::Core(Command::OpenFile)),
        "EnterCommandMode" => Some(UIAction::EnterCommandMode),
        "EnterSearchMode" => Some(UIAction::EnterSearchMode),
        "ToggleExplorer" => Some(UIAction::ToggleExplorer),
        "SwitchFocus" => Some(UIAction::SwitchFocus),
        "Copy" => Some(UIAction::Copy),
        "Paste" => Some(UIAction::Paste),
        "NextTab" => Some(UIAction::NextTab),
        "PrevTab" => Some(UIAction::PrevTab),
        "NewTab" => Some(UIAction::NewTab),
        "ExitMode" => Some(UIAction::ExitMode),
        "LeaveLodge" => Some(UIAction::LeaveWorkspace),
        "Disconnect" => Some(UIAction::DisconnectNetwork),
        _ => None,
    }
}
