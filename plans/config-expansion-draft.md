# DRAFT Plan: Granular Keymap & Theme Configuration

This document outlines the proposed design for the next-generation TelaRex configuration engine, moving beyond hardcoded defaults to a fully user-defined system.

## 1. Keymap Architecture
We will move to a nested TOML structure that allows for context-sensitive keybindings.

### Proposed `keymaps.toml`
```toml
[global]
"ctrl-q" = "Quit"
"ctrl-e" = "SwitchFocus"

[editor.normal]
"j" = "MoveDown"
"k" = "MoveUp"
"h" = "MoveLeft"
"l" = "MoveRight"
"w" = "MoveWordForward"
"e" = "MoveWordEnd"
"b" = "MoveWordBackward"

[editor.insert]
"esc" = "ExitMode"

[explorer]
"enter" = "OpenFile"
"j" = "NavDown"
"k" = "NavUp"
```

### Implementation Logic
- **`ActionRegistry`**: A new module in `telarex-core` that maps string identifiers (e.g., "MoveDown") to internal `UIAction` variants.
- **Dynamic Mapping**: The `KeyMapper` will load these maps at runtime and provide an O(1) lookup during the event loop.

## 2. Granular Theming Architecture
We will adopt a CSS-inspired hierarchical selector system.

### Proposed `theme.toml`
```toml
[ui]
"bg" = "#1e1e2e"
"fg" = "#cdd6f4"
"border.active" = "#b4befe"
"status_bar.bg" = "#181825"
"status_bar.pill.mode.insert" = "#a6e3a1"

[syntax]
"keyword" = { color = "#c678dd", bold = true }
"function" = "#61afef"
"comment" = { color = "#5c6370", italic = true }
"type" = { color = "#e5c07b", bold = true }
"string" = "#98c379"
```

### Implementation Logic
- **`StyleResolver`**: A recursive resolver that checks for the most specific selector first (e.g., `status_bar.pill.mode.insert`) and falls back to more general ones (e.g., `ui.bg`).
- **Live Reload**: Watch the theme file for changes and re-synchronize the `StyleRegistry` instantly, allowing users to "live-design" their editor.

## 3. TUI Configuration UI
The `ConfigModal` will be expanded to include:
- **Interactive Keymap List**: View all active bindings grouped by mode.
- **Binding Conflict Detection**: Visually flag if a custom key overrides a critical global shortcut.
- **Theme Color Picker**: Basic hex input within the TUI to tweak colors on the fly.

## 4. Stage-by-Stage Rollout
- **Stage A**: Migrate hardcoded keys to the internal `KeymapConfig` struct.
- **Stage B**: Implement the `StyleSheet` loader for syntax colors.
- **Stage C**: Add TOML serialization for user-facing editing of these files.
