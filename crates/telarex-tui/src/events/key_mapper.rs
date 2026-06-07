//! Key mapper — resolves [`KeyEvent`]s to [`UIAction`]s using configurable and built-in mappings.
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use crate::events::UIAction;
use telarex_core::config::schema::KeymapConfig;

/// Maps [`KeyEvent`]s to [`UIAction`]s using global, editor, and explorer keymaps.
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
            let result = match (key.code, key.modifiers) {
                (KeyCode::Char('v'), KeyModifiers::NONE) => Some(UIAction::SplitVertical),
                (KeyCode::Char('s'), KeyModifiers::NONE) => Some(UIAction::SplitHorizontal),
                (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, KeyModifiers::NONE) => Some(UIAction::FocusLeft),
                (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => Some(UIAction::FocusRight),
                (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => Some(UIAction::FocusUp),
                (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => Some(UIAction::FocusDown),
                (KeyCode::Char('c'), KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => Some(UIAction::ClosePane),
                (KeyCode::Char('w'), KeyModifiers::CONTROL) => None, // Dedup: discards duplicate Ctrl+W events
                _ => Some(UIAction::ExitMode), // Any other key exits window mode
            };
            log::info!("[KM] window mode: code={:?}, mods={:?}, result={:?}", key.code, key.modifiers, result);
            return result;
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

        // 3. Alt Layer — tab navigation, secondary bindings
        if key.modifiers.contains(KeyModifiers::ALT) && !key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('[') => return Some(UIAction::PrevTab),
                KeyCode::Char(']') => return Some(UIAction::NextTab),
                _ => {}
            }
        }

        // 4. Global Layer (Hardcoded overrides)
        if key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('p') => return Some(UIAction::EnterCommandMode),
                KeyCode::Char('f') => return Some(UIAction::EnterSearchMode),
                KeyCode::Char('m') => return Some(UIAction::ToggleMacroPalette),
                KeyCode::Char('t') => return Some(UIAction::NewTab),
                KeyCode::PageUp => return Some(UIAction::PrevTab),
                KeyCode::PageDown => return Some(UIAction::NextTab),
                KeyCode::Char('w') => {
                    log::info!("[KM] hardcoded match: Ctrl+W -> EnterWindowMode");
                    return Some(UIAction::EnterWindowMode);
                }
                KeyCode::Char('W') => return Some(UIAction::CloseTab),
                KeyCode::Char('e') => return Some(UIAction::SwitchFocus),
                KeyCode::Char('b') => return Some(UIAction::ToggleExplorer),
                KeyCode::Char('s') => return Some(UIAction::Core(telarex_core::command::Command::Save)),
                KeyCode::Char('c') => return Some(UIAction::Copy),
                KeyCode::Char('v') => return Some(UIAction::Paste),
                KeyCode::Char('l') => return Some(UIAction::ToggleLodgeIdVisibility),
                KeyCode::Char('y') => return Some(UIAction::ToggleLodgeIdFormat),
                _ => {}
            }
        }

        // Git status (Ctrl+g)
        if key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('g') {
            return Some(UIAction::Core(telarex_core::command::Command::GitStatus));
        }

        // 5. Fallback: handle raw control characters (0x01-0x1A) as Ctrl+letter
        if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) {
            if let KeyCode::Char(c) = key.code {
                let code = c as u32;
                if (1..=26).contains(&code) {
                    let letter = char::from_u32(code + 0x60).unwrap();
                    log::info!("[KM] raw control char {:?} ({}) -> remapped to Ctrl+{}", c, code, letter);
                    let fallback_key = KeyEvent::new_with_kind(KeyCode::Char(letter), KeyModifiers::CONTROL, key.kind);
                    return self.resolve(fallback_key, mode, component);
                }
            }
        }

        // 6. Fallback for character keys (Never swallow them)
        if let KeyCode::Char(_) = key.code {
             // AltGr (reported as Ctrl+Alt) should NOT trigger shortcuts
             let is_strictly_control = key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT);
             let is_strictly_alt = key.modifiers.contains(KeyModifiers::ALT) && !key.modifiers.contains(KeyModifiers::CONTROL);
             
             if !is_strictly_control && !is_strictly_alt {
                 return None; // Let the editor handle raw chars
             }
        }

         // 6. Config global map
        let result = self.global.get(&key).cloned();
        log::info!("[KM] resolve: code={:?}, mods={:?}, mode={:?}, result={:?}", key.code, key.modifiers, mode, result);
        result
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
        "GitStatus" => Some(UIAction::Core(Command::GitStatus)),
        "GitStageAll" => Some(UIAction::Core(Command::GitStageAll)),
        "GitCommit" => Some(UIAction::Core(Command::GitCommit)),
        "GitPush" => Some(UIAction::Core(Command::GitPush)),
        "GitPull" => Some(UIAction::Core(Command::GitPull)),
        "GitLog" => Some(UIAction::Core(Command::GitLog)),
        "ToggleLodgeIdVisibility" => Some(UIAction::ToggleLodgeIdVisibility),
        "ToggleLodgeIdFormat" => Some(UIAction::ToggleLodgeIdFormat),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind};

    fn make_key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        let mut k = KeyEvent::new(code, mods);
        k.kind = KeyEventKind::Press;
        k
    }

    fn mapper() -> KeyMapper {
        let config = KeymapConfig::default_mappings();
        KeyMapper::from_config(&config)
    }

    #[test]
    fn test_parse_key_event_simple_char() {
        let result = parse_key_event("a").unwrap();
        assert_eq!(result.code, KeyCode::Char('a'));
        assert_eq!(result.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_key_event_ctrl() {
        let result = parse_key_event("ctrl-s").unwrap();
        assert_eq!(result.code, KeyCode::Char('s'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_key_event_ctrl_shift() {
        let result = parse_key_event("ctrl-shift-p").unwrap();
        assert_eq!(result.code, KeyCode::Char('p'));
        assert!(result.modifiers.contains(KeyModifiers::CONTROL));
        assert!(result.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_key_event_named() {
        let tests = [
            ("enter", KeyCode::Enter),
            ("esc", KeyCode::Esc),
            ("tab", KeyCode::Tab),
            ("backspace", KeyCode::Backspace),
            ("up", KeyCode::Up),
            ("down", KeyCode::Down),
            ("left", KeyCode::Left),
            ("right", KeyCode::Right),
        ];
        for (s, expected) in &tests {
            let result = parse_key_event(s).unwrap();
            assert_eq!(result.code, *expected, "failed for {}", s);
        }
    }

    #[test]
    fn test_parse_key_event_invalid() {
        assert!(parse_key_event("").is_none());
        assert!(parse_key_event("ctrl-").is_none());
    }

    #[test]
    fn test_parse_action_all() {
        let map: [(&str, UIAction); 23] = [
            ("Quit", UIAction::Quit),
            ("Save", UIAction::Core(telarex_core::command::Command::Save)),
            ("OpenFile", UIAction::Core(telarex_core::command::Command::OpenFile)),
            ("EnterCommandMode", UIAction::EnterCommandMode),
            ("EnterSearchMode", UIAction::EnterSearchMode),
            ("ToggleExplorer", UIAction::ToggleExplorer),
            ("SwitchFocus", UIAction::SwitchFocus),
            ("Copy", UIAction::Copy),
            ("Paste", UIAction::Paste),
            ("NextTab", UIAction::NextTab),
            ("PrevTab", UIAction::PrevTab),
            ("NewTab", UIAction::NewTab),
            ("ExitMode", UIAction::ExitMode),
            ("LeaveLodge", UIAction::LeaveWorkspace),
            ("Disconnect", UIAction::DisconnectNetwork),
            ("GitStatus", UIAction::Core(telarex_core::command::Command::GitStatus)),
            ("GitStageAll", UIAction::Core(telarex_core::command::Command::GitStageAll)),
            ("GitCommit", UIAction::Core(telarex_core::command::Command::GitCommit)),
            ("GitPush", UIAction::Core(telarex_core::command::Command::GitPush)),
            ("GitPull", UIAction::Core(telarex_core::command::Command::GitPull)),
            ("GitLog", UIAction::Core(telarex_core::command::Command::GitLog)),
            ("ToggleLodgeIdVisibility", UIAction::ToggleLodgeIdVisibility),
            ("ToggleLodgeIdFormat", UIAction::ToggleLodgeIdFormat),
        ];
        for (s, expected) in &map {
            assert_eq!(parse_action(s), Some(expected.clone()), "failed for {}", s);
        }
        assert_eq!(parse_action("Unknown"), None);
    }

    #[test]
    fn test_resolve_ctrl_p_opens_command_mode() {
        let m = mapper();
        let key = make_key(KeyCode::Char('p'), KeyModifiers::CONTROL);
        let action = m.resolve(key, "normal", Some("editor"));
        assert_eq!(action, Some(UIAction::EnterCommandMode));
    }

    #[test]
    fn test_resolve_ctrl_s_saves() {
        let m = mapper();
        let key = make_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let action = m.resolve(key, "normal", Some("editor"));
        assert_eq!(action, Some(UIAction::Core(telarex_core::command::Command::Save)));
    }

    #[test]
    fn test_resolve_window_mode() {
        let m = mapper();
        let v = make_key(KeyCode::Char('v'), KeyModifiers::NONE);
        assert_eq!(m.resolve(v, "window", None), Some(UIAction::SplitVertical));

        let h = make_key(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(m.resolve(h, "window", None), Some(UIAction::FocusLeft));
    }

    #[test]
    fn test_resolve_window_mode_unknown_exits() {
        let m = mapper();
        let x = make_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(m.resolve(x, "window", None), Some(UIAction::ExitMode));
    }

    #[test]
    fn test_resolve_raw_char_passthrough() {
        let m = mapper();
        let a = make_key(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(m.resolve(a, "normal", Some("editor")), None);
    }

    #[test]
    fn test_resolve_ignores_release_events() {
        let m = mapper();
        let mut key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        key.kind = KeyEventKind::Release;
        assert_eq!(m.resolve(key, "normal", Some("editor")), None);
    }

    #[test]
    fn test_resolve_explorer_mode() {
        let m = mapper();
        let key = make_key(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert_eq!(m.resolve(key, "normal", Some("editor")), Some(UIAction::SwitchFocus));
    }

    #[test]
    fn test_resolve_toggle_explorer() {
        let m = mapper();
        let key = make_key(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert_eq!(m.resolve(key, "normal", Some("editor")), Some(UIAction::ToggleExplorer));
    }
}
