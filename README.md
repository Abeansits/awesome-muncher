# awesome-muncher

A TUI for exploring awesome lists and discovering which projects are still alive vs abandoned.

```
╭─────────────────────────────────────────────────────────────────────╮
│ 🟡 awesome-muncher       ●1,456 Alive   👻312 Ghosts   ⚰️161 RIP    │
├─────────────────────────────────────────────────────────────────────┤
│ 🔍 [_______________]   📁 [All Categories ▼]   👁 [All Status ▼]    │
├─────────────────────────────────────────────────────────────────────┤
│ St │ Name                    │   ★    │ Last Active │ Category     │
├────┼─────────────────────────┼────────┼─────────────┼──────────────┤
│ ●  │ ratatui                 │ 17.8k  │ 2 days ago  │ TUI          │
│ ●  │ gitui                   │ 18.2k  │ 5 days ago  │ Git          │
│ 👻 │ forgotten-cli           │ 892    │ 8 months    │ Utils        │
│ ⚰️  │ rip-project             │ 234    │ Archived    │ Legacy       │
╰─────────────────────────────────────────────────────────────────────╯
```

## Features

- **Parse awesome lists** from GitHub (markdown format)
- **Fetch GitHub metadata** (stars, issues, PRs, last commit)
- **Identify abandoned projects** (no commits in 60+ days)
- **Filter and sort** by status, category, stars, activity
- **Pac-Man themed** loading animations
- **Witty status messages** during sync
- **Easter eggs** (try the Konami code!)

## Status Indicators

| Icon | Status | Description |
|------|--------|-------------|
| ● | Active | Last commit < 30 days |
| ◐ | Stale | Last commit 30-60 days |
| 👻 | Ghost | Last commit > 60 days (abandoned) |
| ⚰️ | RIP | Archived on GitHub |
| ○ | Unknown | Not yet fetched |

## Installation

```bash
# Clone and build
cd awesome-muncher
cargo build --release

# Run
./target/release/awesome-muncher
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` / `Home` | Go to first |
| `G` / `End` | Go to last |
| `PgDn` / `PgUp` | Page down/up |
| `/` | Search |
| `s` | Cycle sort column |
| `c` | Cycle category filter |
| `t` | Cycle status filter |
| `Enter` / `o` | Open in browser |
| `S` | Sync/refresh data |
| `?` | Show help |
| `Esc` | Clear filters |
| `q` | Quit |

### First Run

On first run, awesome-muncher will:
1. Fetch the [awesome-cli-apps-in-a-csv](https://github.com/toolleeo/awesome-cli-apps-in-a-csv) list
2. Parse all projects (~1,900+)
3. Start fetching GitHub metadata in the background

The UI is immediately usable while metadata fetches in the background.

### GitHub Token (Recommended)

Without a token, you're limited to 60 GitHub API requests per hour.
With a token, you get 5,000 requests per hour.

Create a config file at:
- macOS: `~/Library/Application Support/com.awesome-muncher.awesome-muncher/config.toml`
- Linux: `~/.config/awesome-muncher/config.toml`

```toml
github_token = "ghp_your_token_here"
abandoned_days = 60
stale_days = 30
```

## Data Storage

- **Config**: `~/.config/awesome-muncher/config.toml` (Linux) or `~/Library/Application Support/` (macOS)
- **Database**: SQLite database in the data directory

## Tech Stack

- **Rust** with async/await
- **Ratatui** for the TUI
- **Tokio** for async runtime
- **rusqlite** for SQLite storage
- **reqwest** for HTTP requests

## License

MIT
