# Spec: TelaRex Aesthetic Revamp ("TelaRex Noir")

## 1. Motivation

Current TelaRex has:
- 2 themes (Nordic Frost, Bloody Red) — neither is a modern standard
- No semantic color system mapped to UI components
- No mixed typography (sans + mono)
- Plain, generic terminal look

Modern terminal apps (Warp, Kitty, iTerm2, Windows Terminal) and the Ratatui ecosystem (Catppuccin, Tokyo Night, Nord) have established a new visual standard for terminal UIs. This spec defines TelaRex's new design language.

## 2. Design Principles

1. **Deep but not black** — backgrounds use dark navy/charcoal, never `#000000`
2. **Surface-based elevation** — depth through layered backgrounds, not shadows
3. **Semantic color mapping** — every color has a purpose, mapped to component roles
4. **Mixed typography** — clean sans-serif for structure, monospace for content
5. **Restrained accent usage** — exactly 2 accent colors maximum, used deliberately
6. **Content-first** — minimize borders and chrome, maximize data density
7. **Consistent spacing** — 4px base grid, 4/8/12/16/24/32 scale

## 3. Default Theme: Tokyo Night

```toml
[meta]
name = "tokyo-night"
dark = true
accent_primary = "#7aa2f7"   # blue
accent_secondary = "#bb9af7" # purple

[colors]
bg = "#1a1b26"
bg_alt = "#16161e"
surface = "#24283b"
surface_alt = "#1f2335"
fg = "#a9b1d6"
fg_muted = "#565f89"
fg_dim = "#3b4261"

[status]
error = "#f7768e"
warning = "#ff9e64"
success = "#9ece6a"
info = "#7dcfff"

[syntax]
keyword = "#bb9af7"
string = "#9ece6a"
function = "#7aa2f7"
comment = "#565f89"
number = "#ff9e64"
type = "#ffdb6c"
operator = "#89ddff"
constant = "#ff9e64"
punctuation = "#a9b1d6"
```

## 4. Component Color Mapping

| Component | Token | How It's Used |
|-----------|-------|---------------|
| Main background | `bg` | Terminal background |
| Panel backgrounds | `surface` | Explorer, palette, modal, sidebar |
| Elevated panels | `surface_alt` | Active tab, hovered items |
| Primary text | `fg` | All content text |
| Secondary text | `fg_muted` | Line numbers, labels, metadata |
| Dim text | `fg_dim` | Disabled items, separators |
| Borders (inactive) | `fg_dim` at 50% alpha | 1px panel borders |
| Borders (focused) | `accent_primary` | 2px focus ring |
| Selection | `accent_primary` at 30% alpha | Text selection, selected file |
| Cursor | `accent_primary` | Text cursor |
| Mode indicator NORMAL | `accent_primary` | Status bar mode |
| Mode indicator INSERT | `success` | Status bar mode |
| Mode indicator VISUAL | `accent_secondary` | Status bar mode |
| Error messages | `error` | Error modal, status |
| Warning messages | `warning` | Warnings |
| Links / info | `info` | Help text, hints |

### 4.1 Accent Color Usage Rules

- `accent_primary` (blue) = primary actions, cursors, selection, active borders
- `accent_secondary` (purple) = visual mode, secondary highlights, tags
- Never use both on the same UI primitive
- Never use accent colors for body text
- Accent backgrounds should use alpha blending (e.g., `accent_primary` at 30% for selection)

## 5. Typography

| Role | Font Family | Weight | Fallback |
|------|------------|--------|----------|
| UI titles, labels | JetBrains Mono | 600 (semi-bold) | Cascadia Code, Fira Code |
| Code, text buffers | JetBrains Mono | 400 (regular) | Cascadia Code, Fira Code |
| Status bar info | JetBrains Mono | 400 | SF Mono, Consolas |
| Numerals / metrics | JetBrains Mono tabular | 500 | — |

Rationale: Using a single monospace family (JetBrains Mono) simplifies rendering and avoids the mixed-font alignment issues that plague terminal apps. The distinction between "UI" and "code" is made through weight and color, not font family. JetBrains Mono was chosen for:
- Tall x-height (readability at small sizes)
- Excellent ligature support for `->`, `=>`, `!=`, etc.
- Tabular numerals for aligned metrics
- Open-source and widely available

## 6. Layout & Spacing

### 6.1 Spacing Scale

Base unit: 4px
Scale: 4, 8, 12, 16, 24, 32, 48

| Element | Padding |
|---------|---------|
| Panel content | 16px (4 units) |
| Explorer items | 8px (2 units) horizontal, 4px (1 unit) vertical |
| Tab bar items | 8px (2 units) horizontal |
| List items | 8px (2 units) horizontal, 4px (1 unit) vertical |
| Modals | 24px (6 units) |
| Status bar | 8px (2 units) horizontal |
| Command palette | 16px (4 units) |

### 6.2 Border System

| Level | Style | Use |
|-------|-------|-----|
| 0 | No border | Content areas, editor |
| 1 | 1px `fg_dim` at 50% | Panel borders, inactive tabs |
| 2 | 1px `fg_muted` | Hovered items |
| 3 | 2px `accent_primary` | Focused panels, active selections |

Corner radius: 6px for all panels, 8px for modals (via `ratatui::widgets::Block`).

### 6.3 Surface Elevation

Depth is communicated solely through background color:
- `bg` — main terminal
- `surface` — floating panels, sidebar, explorer
- `surface_alt` — modals, command palette, active/hover items

## 7. Screen-Specific Guidelines

### 7.1 Welcome View

```
┌─────────────────────────────────────────────────┐
│  ████████  ████████  ██                         │  <- accent_primary ASCII logo
│  ██            ██    ██                         │     centered, 16px top margin
│  ██████        ██    ██                         │
│  ██            ██    ██                         │
│  ████████      ██    ████████                   │
│                                                   │
│  TelaRex v0.1.0                    <- fg_muted   │
│  Collaborative Technical Workspace  <- fg_muted   │
│                                                   │
│  ┌──────────┐  ┌──────────────┐                   │
│  │ Recent   │  │ Active       │                   │ <- surface panels
│  │ Projects │  │ Lodges       │                   │    1px border
│  │          │  │              │                   │
│  │ project1 │  │ Lodge Alpha  │                   │
│  │ project2 │  │ Lodge Beta   │                   │
│  └──────────┘  └──────────────┘                   │
│                                                   │
│  NORMAL  project.rs  12:34  UTF-8   No Lodge      │ <- status bar
└─────────────────────────────────────────────────┘
```

### 7.2 Editor View

```
┌──── file1.rs ──── file2.rs ──── x ──────────────┐  <- tab bar: dim for
│                                                     inactive, accent primary
│  ┌── src/ ───────────────────────────────────┐  │     for active underline
│  │  ► main.rs                               │  │     <- explorer panel
│  │  ► lib.rs                                │  │     surface bg, 1px border
│  │  ► utils/                                │  │     active file: accent bg 30%
│  │    ► helpers.rs                          │  │
│  └──────────────────────────────────────────┘  │
│                                                     <- editor pane: bg
│   1 │ use std::io;                           │        cursor: accent_primary
│   2 │                                         │        selection: accent bg 30%
│   3 │ fn main() -> io::Result<()> {          │        line numbers: fg_muted
│   4 │     println!("Hello");                 │        matching brackets: bold
│   5 │ }                                       │
│                                                 │
│  NORMAL  main.rs  12:34  UTF-8  Lodge: Alpha   │  <- status bar: single line
│  ─────────────────────────────────────────────  │     accent_primary for mode
└─────────────────────────────────────────────────┘
```

### 7.3 Command Palette

```
┌──────────────────────────────────────┐
│  > open file                      │  <- surface_alt bg, accent input
│                                      │
│  Open File [Ctrl+O]              │  <- matched chars in accent
│  Open Recent [Ctrl+R]            │     rest in fg
│  Open Lodge [Ctrl+L]             │     shortcut in fg_muted
│  Save File [Ctrl+S]              │
│  Quit [Ctrl+Q]                    │
└──────────────────────────────────────┘
```

### 7.4 Status Bar (Single Line)

Content: `[MODE] [file_name] [line:col] [encoding] [lodge_name]`

Styling:
- MODE: colored by mode (INSERT = success green, NORMAL = accent blue, VISUAL = accent purple)
- File name: `fg` (bold if modified)
- Position: `fg_muted`
- Encoding: `fg_muted`
- Lodge: `fg_muted` with `accent_secondary` dot indicator if connected

## 8. Bundled Themes

Ship with minimum 5 themes at launch:

| Theme | Type | Accent | Mood |
|-------|------|--------|------|
| Tokyo Night | dark | blue + purple | Technical, focused |
| Catppuccin Mocha | dark | pink + mauve | Warm, soft |
| Nord | dark | frost blue + green | Arctic, clean |
| Dracula | dark | purple + cyan | Iconic, bold |
| Everforest | dark | green | Forest, calm |

All themes follow the same semantic token schema. Users can add custom themes by placing `.toml` files in the themes directory.

## 9. Implementation Plan

### Phase 1: Theme Engine (core)
- [ ] Define semantic theme schema (TOML) with all component tokens
- [ ] Implement `ThemePalette` struct with semantic color accessors
- [ ] Migrate existing `StyleSheet` to new schema
- [ ] Convert `nordic_frost.toml` to new format
- [ ] Create `tokyo-night.toml`
- [ ] Create `catppuccin-mocha.toml`
- [ ] Create `nord.toml`
- [ ] Create `everforest.toml`

### Phase 2: UI Kit (tui)
- [ ] Implement `AppStyling` struct mapping theme tokens to ratatui `Style` values
- [ ] Refactor all `impl Widget` blocks to use `AppStyling` instead of inline styles
- [ ] Standardize border, padding, and spacing across all components
- [ ] Rewrite status bar with mode colors
- [ ] Rewrite tab bar with accent underlines
- [ ] Add selection highlighting with alpha blending
- [ ] Style command palette with matched-char highlighting

### Phase 3: Polish
- [ ] Remove Bloody Red theme (replace with Dracula)
- [ ] Add theme cycling command/keybind
- [ ] Test all themes against dark terminals
- [ ] Ensure all syntax tokens are mapped
- [ ] Write integration tests for theme loading
- [ ] Add config option for font family

### Non-Goals
- GPU rendering (use ratatui's backend)
- Animations or transitions
- True pixel-precise rendering
- Light themes (defer to post-beta)
