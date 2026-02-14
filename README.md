# picotui

[![en](https://img.shields.io/badge/lang-en-red.svg)](README.md)
[![ru](https://img.shields.io/badge/lang-ru-green.svg)](README.ru.md)

[![CI](https://github.com/vkrivopalov/picotui/actions/workflows/ci.yml/badge.svg)](https://github.com/vkrivopalov/picotui/actions/workflows/ci.yml)
[![License: BSL-1.0](https://img.shields.io/badge/License-BSL--1.0-blue.svg)](https://opensource.org/licenses/BSL-1.0)

Terminal UI for [Picodata](https://picodata.io/en/) cluster management, built with [ratatui](https://ratatui.rs/).

## Features

- **Cluster Overview**: View cluster name, version, memory usage, and instance counts
- **Multiple View Modes**: Switch between Tiers (tree), Replicasets (flat), and Instances (flat) views
- **Hierarchical Tree View**: Navigate tiers → replicasets → instances with expand/collapse
- **Sorting**: Sort instances by name or failure domain, ascending or descending
- **Filtering**: Filter instances by name, tier, replicaset, address, or failure domain
- **Instance Details**: View detailed information including addresses, failure domains, and state
- **JWT Authentication**: Login support when authentication is enabled
- **Persistent Sessions**: Optional "Remember me" to save login across sessions
- **Auto-refresh**: Automatic data refresh with configurable interval
- **Debug Mode**: Log all API requests/responses for troubleshooting

## Installation

### From source

```bash
git clone https://github.com/vkrivopalov/picotui.git
cd picotui
cargo build --release
```

The binary will be at `target/release/picotui`.

### Requirements

- Rust 1.70+ (for building from source)
- A running [Picodata](https://picodata.io/en/) cluster with HTTP API enabled

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

### Command-line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-u`, `--url` | Picodata HTTP API URL | `http://localhost:8080` |
| `-r`, `--refresh` | Auto-refresh interval in seconds (0 to disable) | `5` |
| `-d`, `--debug` | Enable debug logging to `picotui.log` | off |
| `-h`, `--help` | Show help message | |
| `-V`, `--version` | Show version | |

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `→` / `l` | Expand selected item (Tiers view) |
| `←` / `h` | Collapse selected item (Tiers view) |
| `Enter` | Show instance details |

### View Modes
| Key | Action |
|-----|--------|
| `g` | Cycle through view modes (Tiers → Replicasets → Instances) |
| `1` | Switch to Tiers view (hierarchical tree) |
| `2` | Switch to Replicasets view (flat list) |
| `3` | Switch to Instances view (flat list with sorting/filtering) |

### Sorting (Instances view only)
| Key | Action |
|-----|--------|
| `s` | Cycle sort field (Name → Failure Domain) |
| `S` | Toggle sort order (ascending ↑ / descending ↓) |

### Filtering (Instances view only)
| Key | Action |
|-----|--------|
| `/` | Start filter mode |
| *type* | Filter text (while in filter mode) |
| `Enter` | Apply filter and exit filter mode |
| `Esc` | Clear filter and exit filter mode |
| `Backspace` | Delete last character (while in filter mode) |

### Actions
| Key | Action |
|-----|--------|
| `r` | Refresh data |
| `X` | Logout and exit (clears saved session) |
| `q` | Quit |
| `Ctrl+C` | Quit |
| `Esc` | Close popup / Clear filter |

### Login Screen
| Key | Action |
|-----|--------|
| `Tab` / `↑` / `↓` | Navigate between fields |
| `Space` | Toggle checkbox (Remember me) |
| `Ctrl+S` | Show/hide password |
| `Enter` | Submit login |
| `Esc` / `q` | Quit |

## Screenshots

![picotui screenshot](images/picotui.png)

### Tiers View (hierarchical tree)
```
┌─ picotui - Picodata Cluster Monitor ────────────────────────[Tiers]─┐
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
│ ↑↓/jk Navigate  ←→/hl Collapse/Expand  Enter Details  g View  ...  │
└─────────────────────────────────────────────────────────────────────┘
```

### Instances View (flat list with sorting/filtering)
```
┌─ picotui - Picodata Cluster Monitor ─────────────────────[Instances]┐
├─────────────────────────────────────────────────────────────────────┤
│ Cluster: my-cluster │ ...                                          │
├─ Instances  Filter: "dc1" ──────────────────────── Sort: Name ↑ ───┤
│ ★ i1 [Online]  RS: r1  10.0.0.1:3301  datacenter:dc1               │
│   i2 [Online]  RS: r1  10.0.0.2:3301  datacenter:dc1               │
│   i4 [Online]  RS: r2  10.0.0.4:3301  datacenter:dc1               │
├─────────────────────────────────────────────────────────────────────┤
│ ↑↓/jk Navigate  Enter Details  g View  s Sort  S Order  / Filter   │
└─────────────────────────────────────────────────────────────────────┘
```

## View Modes

Picotui offers three different ways to view your cluster data, similar to the [Picodata](https://picodata.io/en/) web UI:

### Tiers View (default)

Hierarchical tree view showing the full cluster structure:
- Tiers at the top level (expandable)
- Replicasets nested under tiers (expandable)
- Instances nested under replicasets

Use `→`/`l` to expand and `←`/`h` to collapse nodes. The tree shows memory usage, instance counts, replication factor, and bucket counts at each level.

### Replicasets View

Flat list of all replicasets across all tiers. Each row shows:
- Replicaset name and state (Online/Offline/Expelled)
- Parent tier name
- Instance count
- Memory usage and capacity percentage

### Instances View

Flat list of all instances with sorting and filtering capabilities:
- Instance name with leader indicator (★)
- Current state
- Parent replicaset
- Binary address
- Failure domain (if set)

## Sorting

Sorting is available in the **Instances view** only.

| Sort Field | Description |
|------------|-------------|
| **Name** | Sort by instance name alphabetically |
| **Failure Domain** | Sort by failure domain values, then by name |

Press `s` to cycle through sort fields. Press `S` (Shift+s) to toggle between ascending (↑) and descending (↓) order.

The current sort setting is shown in the bottom-right corner of the instances panel.

## Filtering

Filtering is available in the **Instances view** only.

Press `/` to enter filter mode. Type your filter text to narrow down the displayed instances. The filter matches against:

- **Instance name** (e.g., `i3` matches instance "i3")
- **Tier name** (e.g., `storage` matches all instances in the "storage" tier)
- **Replicaset name** (e.g., `r1` matches all instances in replicaset "r1")
- **Binary address** (e.g., `10.0.0.1` matches instances on that IP)
- **Failure domain values** (e.g., `dc1` matches instances in datacenter "dc1")

All matching is case-insensitive and matches substrings anywhere in the field.

Press `Enter` to apply the filter and continue navigating. Press `Esc` to clear the filter. The active filter is shown in the title bar.

## API Endpoints Used

The TUI connects to these [Picodata](https://picodata.io/en/) HTTP API endpoints:

- `GET /api/v1/config` - Check if authentication is enabled
- `POST /api/v1/session` - Login with username/password
- `GET /api/v1/session` - Refresh session tokens
- `GET /api/v1/cluster` - Get cluster overview
- `GET /api/v1/tiers` - Get tiers with replicasets and instances

## Persistent Sessions

When "Remember me" is checked during login (enabled by default), your session token is saved locally:

| Platform | Token Location |
|----------|----------------|
| Linux/FreeBSD | `~/.config/picotui/tokens.json` |
| macOS | `~/Library/Application Support/picotui/tokens.json` |
| Windows | `%APPDATA%\picotui\tokens.json` |

On next launch, picotui will automatically use the saved token, skipping the login screen.

To clear saved sessions, press `X` (Shift+x) to logout and exit. This deletes the stored token.

## Debug Mode

When running with `--debug`, all API requests and responses are logged to `picotui.log`:

```bash
picotui --url http://localhost:8081 --debug

# In another terminal
tail -f picotui.log
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

[BSL-1.0](LICENSE) (Boost Software License 1.0)
