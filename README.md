# picotui

Terminal UI for Picodata cluster management, built with [ratatui](https://ratatui.rs/).

## Features

- **Cluster Overview**: View cluster name, version, memory usage, and instance counts
- **Hierarchical Tree View**: Navigate tiers → replicasets → instances with expand/collapse
- **Instance Details**: View detailed information including addresses, failure domains, and state
- **JWT Authentication**: Login support when authentication is enabled
- **Persistent Sessions**: Optional "Remember me" to save login across sessions
- **Auto-refresh**: Automatic data refresh with configurable interval
- **Debug Mode**: Log all API requests/responses for troubleshooting

## Installation

```bash
cd picotui
cargo build --release
```

The binary will be at `target/release/picotui`.

## Usage

```bash
# Connect to local Picodata instance
picotui --url http://localhost:8080

# Connect with custom refresh interval (in seconds)
picotui --url http://localhost:8080 --refresh 10

# Disable auto-refresh
picotui --url http://localhost:8080 --refresh 0

# Enable debug logging (writes to picotui.log)
picotui --url http://localhost:8080 --debug
```

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `→` / `l` | Expand selected item |
| `←` / `h` | Collapse selected item |
| `Enter` | Show instance details |

### Actions
| Key | Action |
|-----|--------|
| `r` | Refresh data |
| `X` | Logout and exit (clears saved session) |
| `q` | Quit |
| `Ctrl+C` | Quit |
| `Esc` | Close popup |

### Login Screen
| Key | Action |
|-----|--------|
| `Tab` / `↑` / `↓` | Navigate between fields |
| `Space` | Toggle checkbox (Remember me) |
| `Ctrl+S` | Show/hide password |
| `Enter` | Submit login |
| `Esc` / `q` | Quit |

## Screenshots

```
┌─────────────────────────────────────────────────────────────────────┐
│  picotui - Picodata Cluster Monitor                                │
├─────────────────────────────────────────────────────────────────────┤
│ Cluster: my-cluster │ Version: 25.6.0 │ Picodata: 25.6.0           │
│ Instances: 6/6 online │ Plugins: none                              │
│ Memory: 1.2 GiB / 4.0 GiB (30.0%) ████████░░░░░░░░░░░░░░░░░░░░░░░  │
├─────────────────────────────────────────────────────────────────────┤
│ ▼ default  RS: 2  Inst: 6  RF: 3  Buckets: 3000  Vote: ✓           │
│   ├─▼ r1 [Online]  Inst: 3  Mem: 600 MiB/2 GiB (30.0%)             │
│   │  ├─ ★ i1 [Online]  10.0.0.1:3301  pg:10.0.0.1:5432             │
│   │  ├─   i2 [Online]  10.0.0.2:3301  pg:10.0.0.2:5432             │
│   │  └─   i3 [Offline] 10.0.0.3:3301  pg:10.0.0.3:5432             │
│   └─▶ r2 [Online]  Inst: 3  Mem: 600 MiB/2 GiB (30.0%)             │
│ ▶ storage  RS: 1  Inst: 3  RF: 3  Buckets: 0  Vote: ✗              │
├─────────────────────────────────────────────────────────────────────┤
│ ↑↓/jk Navigate  ←→/hl Collapse/Expand  Enter Details  r Refresh  X Logout  q Quit │
└─────────────────────────────────────────────────────────────────────┘
```

## API Endpoints Used

The TUI connects to these Picodata HTTP API endpoints:

- `GET /api/v1/config` - Check if authentication is enabled
- `POST /api/v1/session` - Login with username/password
- `GET /api/v1/session` - Refresh session tokens
- `GET /api/v1/cluster` - Get cluster overview
- `GET /api/v1/tiers` - Get tiers with replicasets and instances

## Persistent Sessions

When "Remember me" is checked during login (enabled by default), your session token is saved locally:

- **Linux/FreeBSD**: `~/.config/picotui/tokens.json`
- **macOS**: `~/Library/Application Support/picotui/tokens.json`
- **Windows**: `%APPDATA%\picotui\tokens.json`

On next launch, picotui will automatically use the saved token, skipping the login screen.

To clear saved sessions, press `X` (Shift+x) to logout and exit. This deletes the stored token.

## Debug Mode

When running with `--debug`, all API requests and responses are logged to `picotui.log`:

```bash
picotui --url http://localhost:8081 --debug

# In another terminal
tail -f picotui.log
```

## License

BSL-1.0 (Boost Software License 1.0)
