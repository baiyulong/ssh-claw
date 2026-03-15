# term-ssh-manager

A fast, lightweight **TUI (Terminal User Interface)** SSH connection manager written in Rust. Manage your remote servers efficiently without leaving the terminal.

## Features

✨ **Core Functionality**
- 🖥️ Interactive server list with highlighting
- ➕ Add, edit, and delete SSH server configurations  
- 🔐 Store server credentials (hostname, username, port)
- 🏷️ Organize servers with optional tags/notes
- ⚡ One-key SSH connection launch
- 💾 Persistent JSON-based configuration

🎨 **User Experience**
- Vim-inspired keyboard navigation (`j/k`, `↑/↓`)
- Intuitive form-based UI for server management
- Modal dialogs with confirmation prompts
- Clean, compact terminal interface
- Cross-platform support (Linux, macOS, Windows)

🛡️ **Reliability**
- Terminal state recovery on crash/panic
- Zero-warning Rust compilation
- Seamless SSH session handoff (no data loss)

## Installation

### From Source

Requires **Rust 1.56+** and a C compiler (for dependencies):

```bash
git clone https://github.com/baiyulong/ssh-claw.git
cd ssh-claw
cargo build --release
```

Binary location: `target/release/term-ssh-manager`

### Quick Start

```bash
# Run the app
cargo run

# Or with a custom config file
cargo run -- --config ~/my-servers.json
```

## Usage

### Launching the App

```bash
term-ssh-manager
# Servers are loaded from ~/.config/term-ssh-manager/servers.json
# If the file doesn't exist, it will be created on first server addition.
```

**Optional: Use a custom config file**
```bash
term-ssh-manager --config /path/to/servers.json
```

### Keyboard Shortcuts

#### Dashboard (Main List)
| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Connect to selected server |
| `a` | Add new server |
| `e` | Edit selected server |
| `d` | Delete selected server (shows confirm dialog) |
| `q` | Quit application |

#### Form (Add/Edit)
| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` / `BackTab` | Move to previous field |
| `Enter` | Save server configuration |
| `Esc` | Cancel and return to dashboard |
| `Backspace` | Delete character |
| Text input | Type normally |

#### Delete Confirmation
| Key | Action |
|-----|--------|
| `y` | Confirm deletion |
| `n` / `Esc` | Cancel deletion |

### Configuration File Format

Servers are stored in JSON format at `~/.config/term-ssh-manager/servers.json`:

```json
[
  {
    "alias": "Production DB",
    "host": "192.168.1.50",
    "username": "admin",
    "port": 22,
    "tags": "database, critical"
  },
  {
    "alias": "Dev Server",
    "host": "dev.example.com",
    "username": "user",
    "port": 2222,
    "tags": "development"
  }
]
```

**Fields:**
- `alias` — Display name in the server list
- `host` — IP address or hostname
- `username` — SSH username (optional; defaults to current user if empty)
- `port` — SSH port (default: 22)
- `tags` — Optional notes/labels for organization

### SSH Connection Flow

1. Select a server and press `Enter`
2. Terminal exits TUI mode and restores standard terminal state
3. System `ssh` command launches: `ssh -p <port> <username>@<host>`
4. After SSH session ends, press any key to return to the server list

## Architecture

### Elm-inspired State Management

The app follows a clean **Model-Update-View** pattern:

- **Model** (`app.rs`) — Application state: servers, selection, screen mode, form data
- **Update** (`input.rs`) — Event handler: translates keyboard input to state mutations
- **View** (`ui.rs`) — Renderer: converts state to terminal graphics using ratatui

### Key Components

| File | Purpose |
|------|---------|
| `main.rs` | Entry point, terminal setup/teardown, panic hook, main event loop |
| `app.rs` | App state struct, screen state machine, mutations |
| `ui.rs` | Rendering with ratatui: dashboard, forms, dialogs |
| `input.rs` | Keyboard event dispatch per screen mode |
| `server.rs` | Server struct, JSON serialization, config path resolution |
| `ssh.rs` | SSH process spawning |

## Dependencies

- **ratatui** (0.28) — Terminal UI framework
- **crossterm** (0.28) — Terminal backend, raw mode, event handling
- **serde** + **serde_json** (1.0) — Configuration serialization
- **directories** (5) — Standard config directory resolution
- **clap** (4) — Command-line argument parsing

## Building & Testing

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests (if any)
cargo test

# Check for warnings
cargo clippy
```

## Platform Support

- ✅ **Linux** — Fully supported
- ✅ **macOS** — Fully supported  
- ✅ **Windows** — Fully supported (via crossterm)

Requires a working `ssh` command in PATH (pre-installed on most systems).

## Troubleshooting

### "SSH command not found"
Ensure `ssh` is installed and in your system PATH:
```bash
which ssh
```

### "Permission denied (publickey)"
SSH connection failed — check your server credentials and key setup:
```bash
ssh -vv user@host -p port  # Debug SSH connection
```

### "Config file not found"
The app creates `~/.config/term-ssh-manager/servers.json` automatically on first use.  
To verify:
```bash
cat ~/.config/term-ssh-manager/servers.json
```

## Development

### Code Style
- No warnings: all code passes `cargo clippy`
- Minimal comments — only where logic clarity is needed
- Idiomatic Rust conventions

### Adding Features
1. Identify which module to modify (app logic, UI, input handling)
2. Update relevant functions
3. Test with `cargo build` and manual testing
4. Commit with clear messages

### Panic Safety
The app installs a panic hook that **always** restores terminal state, preventing corrupted terminal output on crash.

## License

[Add your license here]

## Contributing

Contributions welcome! Please:
1. Fork the repo
2. Create a feature branch
3. Submit a pull request with a clear description

---

**Made with ❤️ using Rust and ratatui**
