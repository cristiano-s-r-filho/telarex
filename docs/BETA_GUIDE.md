# TelaRex Beta Guide

## Overview

TelaRex is a terminal-based collaborative editor with **native P2P collaboration**, **quantum-resistant identity**, and **tree-sitter syntax highlighting**.

Built for developers who want real-time collaboration without centralized servers.

## Quick Start

```bash
# Open a file
trex path/to/file.rs

# Start with a specific session identity
trex --session my-session-id

# Open current directory
trex .
```

### Controls

| Key | Action |
|-----|--------|
| `Ctrl+P` | Command palette |
| `Ctrl+F` | Search across project |
| `Ctrl+S` | Save file |
| `Ctrl+W` | Window mode (split/focus management) |
| `Ctrl+E` | Toggle focus between editor and file explorer |
| `Ctrl+B` | Toggle file explorer |
| `Ctrl+T` | New tab |
| `Tab` / `Shift+Tab` | Next/Previous tab |
| `Ctrl+C` (welcome) | Exit |
| `Ctrl+Q` | Quit |
| `Ctrl+G` | Git status |

### Navigation

| Key | Action |
|-----|--------|
| `Up/Down` or `Ctrl+P/Ctrl+N` | Move cursor |
| `Left/Right` | Move cursor |
| `PageUp/PageDown` | Scroll page |
| `Home/End` | Line start/end |

### Editing

| Key | Action |
|-----|--------|
| Type characters | Insert text |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `Enter` | New line |
| `Ctrl+C` | Copy selection |
| `Ctrl+V` | Paste |

## LodgeNet (P2P Collaboration)

LodgeNet enables **serverless, real-time collaboration** via libp2p with Automerge CRDT.

### Concepts

- **Lodge**: A shared workspace. Each lodge has a unique UUID.
- **Host**: Creates a lodge by sharing their workspace.
- **Guest**: Joins an existing lodge via discovery or ID.
- **Identity**: Quantum-resistant keypair (ML-DSA 3.1 / FIPS 204) for authentication.

### Creating a Lodge

1. In the editor, press `Ctrl+P` to open the command palette.
2. Select **Share Workspace (Lodge)**.
3. Enter a name for your lodge.
4. Other peers on the network will discover it automatically.

### Joining a Lodge

From the welcome screen:
- **L** — Browse discovered lodges on the local network
- **J** — Join by specific lodge UUID

### How Sync Works

LodgeNet uses **Automerge** CRDT (Conflict-free Replicated Data Type):

- Every edit creates a new operation in the document's operation log
- Operations are exchanged between peers as sync messages
- The CRDT ensures **eventual consistency** — all peers converge to the same state
- No central server required; peers communicate directly via libp2p

### Authentication

LodgeNet uses **UCAN-style capability tokens** signed by a Dilithium root key:

- Each identity keypair is generated from a seed stored in your config
- Join requests include a signed challenge proving identity
- Lodge owners can authorize or revoke members

## Git Sidecar

Git operations run as a **manual, parallel workflow** alongside Automerge sync. They never interfere with each other — Git tracks file history while Automerge tracks real-time collaborative state.

### Commands

Access via the command palette (`Ctrl+P`):

| Command | Action |
|---------|--------|
| **Git Status** | Show working tree status (logged to console) |
| **Git Stage All** | Stage all changes |
| **Git Commit** | Enter commit message and commit staged changes |
| **Git Push** | Push to remote origin |
| **Git Pull** | Fetch from remote origin |
| **Git Log** | Show last 10 commits |

### Status Bar

When inside a git repository, the status bar shows:

```
 main +3     ← branch name and dirty file count
```

## Themes

TelaRex ships with 6 built-in themes:

| Theme | Variant |
|-------|---------|
| **Tokyo Night** (default) | Dark |
| **Catppuccin Mocha** | Dark |
| **Dracula** | Dark |
| **Everforest** | Dark |
| **Nordic Frost** | Dark |
| **Bloody Red** | Dark |

### Changing Theme

1. Press `Ctrl+P` → **Open Configuration**
2. Select **Appearance** → **Theme**
3. Use `Left/Right` to cycle through available themes and confirm with `Enter`

### Theme Format

Custom themes are TOML files in the `themes/` directory:

```toml
name = "My Theme"

[metadata]
author = "You"
variant = "dark"

[palette]
bg = "#1a1b26"
fg = "#a9b1d6"
accent = "#7aa2f7"

[ui]
bg = "bg"
fg = "fg"
border_active = "accent"
selection_bg = { color = "accent", alpha = 0.3 }
# ... see existing themes for full reference

[syntax]
keyword = { color = "accent_secondary", bold = true }
function = { color = "accent" }
string = { color = "green" }
```

## Configuration

Configuration is stored as TOML in the platform data directory:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/telarex/config.toml` |
| macOS | `~/Library/Application Support/telarex/config.toml` |
| Windows | `%APPDATA%/telarex/config.toml` |

### Key sections

```toml
[editor]
tab_size = 4
vim_mode = false
line_numbers = true
auto_save = false
wrap_text = false
theme = "Tokyo Night"

[profile]
username = "my-username"
identity_seed = "auto-generated-uuid"

[network]
bootstrap_node = "/ip4/.../tcp/..."
listen_addr = "/ip4/0.0.0.0/tcp/0"
```

## Keybinding Customization

Keybindings are defined in the config under `[keymaps]`:

```toml
[keymaps.global]
"ctrl-q" = "Quit"
"ctrl-s" = "Save"

[keymaps.editor_normal]
"ctrl-f" = "EnterSearchMode"

[keymaps.explorer]
"enter" = "OpenFile"
```

Available action names: `Quit`, `Save`, `OpenFile`, `EnterCommandMode`, `EnterSearchMode`, `ToggleExplorer`, `SwitchFocus`, `Copy`, `Paste`, `NextTab`, `PrevTab`, `NewTab`, `ExitMode`, `LeaveLodge`, `Disconnect`, `GitStatus`, `GitStageAll`, `GitCommit`, `GitPush`, `GitPull`, `GitLog`.

## Known Limitations (Beta)

- **Automerge sync** currently uses local network discovery (mDNS) — WAN support via relay nodes is planned
- **Git operations** are manual only (no auto-commit)
- **LSP support** is experimental (Rust and JSON only)
- **Windows** keyboard enhancement may not work in all terminals (use Windows Terminal for best results)
- **Themes** with alpha support (`selection_bg`) use simple color fallback on terminals without true color support

## Building from Source

```bash
git clone https://github.com/telarex/telarex
cd telarex
cargo build --release
./target/release/trex
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `lsp` | yes | LSP client support |
| `clipboard` | yes | System clipboard integration |
| `git` | yes | Git Sidecar operations |
| `unstable` | no | Unstable/experimental features |

## Architecture

```
telarex/
├── crates/
│   ├── telarex-core/     # Shared logic: buffers, CRDT, network, syntax, config
│   └── telarex-tui/      # Terminal UI: ratatui screens, components, events
├── themes/               # TOML theme files
├── docs/                 # Architecture documentation
└── plans/                # Planning/spec documents
```

Key dependencies:
- **ratatui** + **crossterm** — TUI framework
- **automerge** — CRDT for real-time sync
- **libp2p** — P2P networking
- **tree-sitter** — syntax highlighting
- **pqc_dilithium** — post-quantum signatures (FIPS 204)
- **ropey** — text buffer with rope data structure
- **rusqlite** — local database (lodges, sessions, recent projects)
