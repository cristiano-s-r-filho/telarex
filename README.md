# TelaRex

<p align="center">
  <strong>Terminal-based collaborative text editor</strong><br>
  <em>P2P sync · Post-quantum identity · Tree-sitter highlighting</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/telarex-tui"><img src="https://img.shields.io/crates/v/telarex-tui?style=flat&label=telarex-tui" alt="crates.io"></a>
  <a href="https://github.com/cristiano-s-r-filho/telarex/actions/workflows/ci.yml"><img src="https://github.com/cristiano-s-r-filho/telarex/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/cristiano-s-r-filho/telarex/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" alt="License"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.80+-orange" alt="Rust"></a>
  <br>
  <a href="https://github.com/cristiano-s-r-filho/telarex/releases"><img src="https://img.shields.io/github/v/release/cristiano-s-r-filho/telarex?include_prereleases" alt="GitHub Release"></a>
  <a href="https://github.com/cristiano-s-r-filho/homebrew-telarex"><img src="https://img.shields.io/badge/brew-tap-f44" alt="Homebrew"></a>
  <a href="https://github.com/cristiano-s-r-filho/scoop-telarex"><img src="https://img.shields.io/badge/scoop-bucket-blue" alt="Scoop"></a>
</p>

---

## Features

- **Native P2P Collaboration** — Real-time collaborative editing via libp2p + Automerge CRDT. No servers, no accounts, no cloud.
- **Post-Quantum Identity** — Peer authentication using ML-DSA (Dilithium, FIPS 204), NIST's post-quantum signature standard.
- **Tree-sitter Highlighting** — Syntax highlighting for Rust, Python, JavaScript, TypeScript, JSON, TOML, YAML, HTML, CSS, and Markdown.
- **LSP Integration** — Completions, diagnostics, and go-to-definition (experimental, gated behind `lsp` feature).
- **Modal Editing** — Vim/Helix-inspired modes with fully configurable keymaps.
- **Git Sidecar** — Stage, commit, push, pull without leaving the editor.
- **Bento UI** — Tokyo Night, Catppuccin Mocha, Dracula, Everforest, Nordic Frost, Bloody Red — 6 built-in themes with custom TOML theme support.
- **Widget Set** — File explorer, tab bar, status bar, command palette, search overlay, split views.

## Installation

### Cargo

```bash
cargo install telarex-tui
```

### Homebrew (macOS / Linux)

```bash
brew tap cristiano-s-r-filho/homebrew-telarex
brew install trex
```

### Scoop (Windows)

```bash
scoop bucket add telarex https://github.com/cristiano-s-r-filho/scoop-telarex
scoop install telarex
```

### Build from source

```bash
git clone https://github.com/cristiano-s-r-filho/telarex
cd telarex
cargo build --release
./target/release/trex
```

The binary is named `trex`.

## Quick Start

```bash
# Open a file
trex path/to/file.rs

# Start with a specific session identity
trex --session my-session-id

# Open current directory
trex .
```

### Basic Controls

| Key | Action |
|-----|--------|
| `Ctrl+P` | Command palette |
| `Ctrl+F` | Search across project |
| `Ctrl+S` | Save file |
| `Ctrl+B` | Toggle file explorer |
| `Ctrl+E` | Toggle focus (editor / explorer) |
| `Ctrl+T` | New tab |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `Ctrl+Q` | Quit |
| `Ctrl+G` | Git status |

See the [Beta Guide](docs/BETA_GUIDE.md) for the full keybinding reference.

## LodgeNet — P2P Collaboration

LodgeNet enables serverless, real-time collaboration via libp2p with Automerge CRDT.

- **Lodge** — A shared workspace identified by a UUID.
- **Host** — Creates a lodge by sharing their workspace. Others discover it on the local network.
- **Guest** — Joins a lodge via automatic discovery or by entering the lodge ID.
- **Identity** — Quantum-resistant keypair (ML-DSA 3.1 / FIPS 204) for signed challenges.

```
┌──────────┐     libp2p (gossipsub)     ┌──────────┐
│  Host    │◄──────────────────────────►│  Guest   │
│ (Author) │                            │ (Peers)  │
└──────────┘                            └──────────┘
     │                                        │
     └──── Automerge CRDT sync ──────────────┘
     └──── All peers converge to same state ──┘
```

### How it works

1. **Host** presses `Ctrl+P` → selects **Share Workspace (Lodge)** → enters a lodge name.
2. **Guests** on the same local network discover the lodge via mDNS automatically.
3. From the welcome screen: press `L` to browse lodges, or `J` to join by ID.
4. Every edit is recorded as an Automerge operation. Peers exchange ops via libp2p and converge to the same state — eventual consistency, no central server.

## Git Sidecar

Git operations run as a manual, parallel workflow alongside Automerge sync. They never interfere — Git tracks file history, Automerge tracks real-time state.

| Command | Action |
|---------|--------|
| `Git Status` | Show working tree status |
| `Git Stage All` | Stage all changes |
| `Git Commit` | Enter a message and commit |
| `Git Push` | Push to remote origin |
| `Git Pull` | Pull from remote origin |
| `Git Log` | Show last 10 commits |

Access any command via the palette (`Ctrl+P`). The status bar shows the current branch and dirty file count.

## Themes

6 built-in dark themes:

| Theme | Author |
|-------|--------|
| **Tokyo Night** (default) | TelaRex Team |
| **Catppuccin Mocha** | TelaRex Team |
| **Dracula** | TelaRex Team |
| **Everforest** | TelaRex Team |
| **Nordic Frost** | TelaRex Team |
| **Bloody Red** | Gemini |

Change themes via `Ctrl+P` → **Open Configuration** → **Appearance** → **Theme**. Custom themes are TOML files — see the [Beta Guide](docs/BETA_GUIDE.md#theme-format) for the schema.

## Project Architecture

```
telarex/
├── crates/
│   ├── telarex-core/     Shared logic: buffers, CRDT, network, syntax, config
│   │   ├── src/
│   │   │   ├── buffer/       Document model with rope data structure, undo/redo, macros
│   │   │   ├── crdt/         Automerge sync engine
│   │   │   ├── network/      libp2p host, LodgeNet protocol, mDNS discovery
│   │   │   ├── syntax/       Tree-sitter highlighting with themeable stylesheets
│   │   │   ├── config/       TOML config, schema validation, theme engine
│   │   │   ├── lsp/          LSP client (completions, diagnostics)
│   │   │   ├── database/     SQLite persistence (lodges, sessions, recents)
│   │   │   └── identity/     Dilithium keypair generation and signing
│   │   └── tests/
│   └── telarex-tui/          Terminal UI (binary: `trex`)
│       └── src/
│           ├── components/   Ratatui widgets: editor, explorer, tab bar, status bar
│           ├── screens/      Welcome, file picker, search overlays
│           ├── events/       Input handling, key mapping, focus system
│           └── theme/        Theme definitions and palette resolution
├── themes/                   TOML theme files
├── docs/                     Architecture and beta documentation
├── test_fixtures/            LSP test fixture files
├── .brew/                    Homebrew formula
└── .scoop/                   Scoop manifest
```

### Key dependencies

| Crate | Purpose |
|-------|---------|
| [ratatui] + [crossterm] | Terminal UI framework |
| [automerge] | CRDT for real-time sync |
| [libp2p] | P2P networking (gossipsub, mDNS, Kademlia) |
| [tree-sitter] | Syntax highlighting |
| [pqc_dilithium] | Post-quantum signatures (FIPS 204) |
| [ropey] | Text buffer (rope data structure) |
| [rusqlite] | Local database |

[ratatui]: https://github.com/ratatui/ratatui
[crossterm]: https://github.com/crossterm-rs/crossterm
[automerge]: https://github.com/automerge/automerge-rs
[libp2p]: https://github.com/libp2p/rust-libp2p
[tree-sitter]: https://github.com/tree-sitter/tree-sitter
[pqc_dilithium]: https://github.com/arkworks-rs/pqc-dilithium
[ropey]: https://github.com/cessen/ropey
[rusqlite]: https://github.com/rusqlite/rusqlite

### Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `lsp` | yes | LSP client support |
| `clipboard` | yes | System clipboard integration |
| `git` | yes | Git sidecar operations |
| `unstable` | no | Unstable / experimental features |

## Configuration

Config is stored as TOML in the platform data directory:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/telarex/config.toml` |
| macOS | `~/Library/Application Support/telarex/config.toml` |
| Windows | `%APPDATA%/telarex/config.toml` |

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

[network]
listen_addr = "/ip4/0.0.0.0/tcp/0"
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for bug reports, feature requests, and pull request guidelines.

## License

Licensed under either of [MIT](LICENSE) or [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) at your option.
