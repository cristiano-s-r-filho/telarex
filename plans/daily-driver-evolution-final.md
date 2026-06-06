# Plan: Daily Driver Finalization (Motions, Effects, & Theming)

This plan completes the "Daily Driver" evolution of TelaRex by polishing the motion engine, implementing a stable and modular visual effect system, and introducing a CSS-inspired TOML theming engine.

## Objectives
1.  **Vim-Lite Motions**: Implement the missing `WordEnd` (`e`) motion and ensure robust boundary handling.
2.  **TachyonFX Modular Stability**: Implement a `TransientEffectManager` for duration-capped animations (Glitch, Scanline, Pulse).
3.  **CSS-Style TOML Theming**: Transition to a hierarchical TOML theme format with CSS-like selectors.
4.  **Granular Config Panel**: Redesign the configuration UI for categorized, detailed settings.

## 1. Motion Engine Polish (`telarex-core`)
### Changes in `src/buffer/motions.rs`
- Add `Motion::WordEnd` to the enum.
- Implement `WordEnd` logic:
  - If at the end of a word, move to the end of the next word.
  - If inside a word, move to its end.
- Update `find_motion_range` to handle these cases precisely.

### Verification
- Add unit tests in `motions.rs` covering:
  - `WordForward` over multiple spaces.
  - `WordBackward` from start of line.
  - `WordEnd` from middle of word vs. end of word.

## 2. TachyonFX Modular Effects (`telarex-tui`)
### `TransientEffectManager` (`src/utils/effects.rs`)
A wrapper around TachyonFX's `EffectManager` that handles the lifecycle of "transient" (temporary) effects.

```rust
pub struct TransientEffectManager {
    manager: EffectManager,
    active_effects: Vec<Uuid>,
}

impl TransientEffectManager {
    pub fn trigger_glitch(&mut self, area: Rect, duration: u32);
    pub fn trigger_scanline(&mut self, area: Rect, color: Color);
    pub fn update(&mut self, dt: Duration);
    pub fn render(&self, buffer: &mut Buffer, area: Rect);
}
```

### Modular Effects Library
- **`fx_glitch`**: Sequence of `fx::dissolve` and `fx::color_shift`.
- **`fx_pulse`**: Oscillator-based background glow using `tachyonfx::Interpolation::SineInOut`.
- **`fx_scanline`**: A `fx::sweep_in` combined with a temporary brightness boost.

## 3. CSS-Style TOML Theming (`telarex-core`)
### `ThemeEngine` (`src/config/theme_engine.rs`)
Load themes from TOML that follow a hierarchical structure.

```toml
# themes/catppuccin.toml
[ui]
"bg" = "#1e1e2e"
"fg" = "#cdd6f4"
"editor.gutter.bg" = "#1e1e2e"

[syntax]
"keyword" = { color = "#c678dd", bold = true }
"function.method" = "#61afef"
"comment" = { color = "#5c6370", italic = true }
```

### Implementation
- Use `serde` to deserialize the TOML into a `StyleSheet`.
- Support "Theme Overrides" in the main `config.toml`.

## 4. Granular Configuration Panel (`telarex-tui`)
### UI Overhaul (`src/components/modals/config_modal.rs`)
- Split the modal into a sidebar (Categories) and a main area (Settings).
- **Categories**:
  - **Editor**: Tab size, Line numbers, Vim mode, Auto-save.
  - **Appearance**: Theme selection, Cursor style, Effect intensity.
  - **Network**: Username, Bootstrap nodes, P2P status.
  - **Keymaps**: List and eventually edit keybindings.

## Implementation Phases

### Phase 1: Motions & Core Logic
1. Implement `WordEnd` and tests in `telarex-core`.
2. Map `e` in Normal Mode to `WordEnd`.

### Phase 2: Effect Infrastructure
1. Implement `TransientEffectManager`.
2. Add "View Change Glitch" (200ms) when switching tabs.

### Phase 3: Theming Engine
1. Implement TOML parser for `StyleSheet`.
2. Add "Theme" selection to Config Modal.

### Phase 4: Config Panel Polish
1. Refactor `ConfigModal` to the two-pane layout.
2. Ensure all fields in `TelaRexConfig` are editable.

## Verification Plan
1. **Motions**: Verify `e` works as expected in various edge cases.
2. **Effects**: Verify glitches trigger on tab switch and disappear correctly.
3. **Themes**: Load a custom TOML theme and verify syntax and UI colors update.
4. **Config**: Change `tab_size` in the panel and see it reflected in the editor instantly.
