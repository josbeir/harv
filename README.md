<p align="center">
  <img src="assets/harv-banner.svg" alt="Harv Banner" width="600" />
</p>

<p align="center">

`harv` — Because remembering to punch the clock is harder than writing the code. A Rust CLI for [Harvest](https://www.getharvest.com/) that respects your terminal, your config, and your deadline.

</p>

<div align="center">

[![CI](https://github.com/josbeir/harv/actions/workflows/ci.yml/badge.svg)](https://github.com/josbeir/harv/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/josbeir/harv/branch/main/graph/badge.svg)](https://codecov.io/gh/josbeir/harv)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

</div>

## Installation

### From GitHub

```bash
cargo install --git https://github.com/josbeir/harv harv-cli
```

This compiles in release mode and installs `harv` to `~/.cargo/bin/` (which must be in your `$PATH`).

### From local source

```bash
git clone https://github.com/josbeir/harv
cd harv
cargo install --path crates/harv-cli
```

### Shell completions

Add to `~/.bashrc` (or the equivalent for your shell):

```bash
source <(harv completion bash)
```

Other shells: replace `bash` with `zsh`, `fish`, `elvish`, or `powershell`.

## Quick Start

### 1. Authenticate

```bash
harv connect
```

Opens your browser to authenticate with Harvest via OAuth2. Credentials are stored at `~/.config/harv/config.json`.

### 2. Track time

```bash
harv track
```

An interactive wizard that prompts for:

- **Project** — fuzzy search, pick with arrow keys
- **Task** — filtered to the selected project
- **Date** — defaults to today
- **Hours** — decimal (`1.5`) or HH:MM (`1:30`); enter 0 or leave empty to start a running timer
- **Notes** — optional

Once you track time, your last-used project and task are remembered — next time you run `harv track`, that project appears at the top with a `●` for a quick Enter skip.

### 3. Quick commands

```bash
harv start [alias]       # Start a running timer
harv stop                # Stop the running timer
harv log 2.5 [alias]     # Log 2.5 hours
harv note                # Edit running timer notes (append by default)
harv status              # Show current timer + today's entries
```

### 4. Aliases

Create shortcuts for frequently used project/task pairs:

```bash
harv alias create dev    # Interactive: pick project + task
harv alias list
harv alias delete dev
```

Use aliases to skip prompts:

```bash
harv start dev
harv log 1.5 dev
```

## Commands


```bash
▗▖ ▗▖ ▗▄▖ ▗▄▄▖ ▗▖  ▗▖
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌
▐▛▀▜▌▐▛▀▜▌▐▛▀▚▖▐▌  ▐▌
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌ ▝▚▞▘ 
   HARV CLI v0.1.0

CLI for Harvest time tracking

Usage: harv [OPTIONS] <COMMAND>

Commands:
  connect     Authenticate with Harvest via OAuth2
  config      Show or modify configuration
  track       Interactive time entry wizard
  start       Start a running timer
  stop        Stop the current running timer
  log         Log time with specified hours
  note        Edit notes on the running timer
  status      Show current timer status and today\'s entries
  projects    List your project assignments
  tasks       List tasks for a project
  alias       Manage project/task aliases
  completion  Generate shell completion script
  help        Print this message or the help of the given subcommand(s)

Options:
  -o, --output <OUTPUT>  [default: table] [possible values: table, json]
  -h, --help             Print help
  -V, --version          Print version
```

| Command | Description |
|---------|-------------|
| `harv connect` | Authenticate with Harvest via OAuth2 |
| `harv config` | Show full configuration |
| `harv config get <key>` | Get a config value (e.g. `cache-ttl`) |
| `harv config set <key> <val>` | Set a config value (e.g. `cache-ttl 48`) |
| `harv track` | Interactive time entry wizard |
| `harv start [alias]` | Start a running timer |
| `harv stop` | Stop the current running timer |
| `harv log <hours> [alias]` | Log time with specified hours |
| `harv note` | Edit notes on the running timer |
| `harv status` | Show current timer + today's entries |
| `harv projects` | List project assignments |
| `harv tasks <project-id>` | List tasks for a project |
| `harv alias` | Manage project/task aliases |
| `harv completion <shell>` | Generate shell completion script |

## Global Options

| Flag | Description |
|------|-------------|
| `-o, --output <table\|json>` | Output format (default: table) |
| `-R, --refresh` | Force-refresh cached data from the API |

## Configuration

Config is stored at `~/.config/harv/config.json`. View with `harv config`, modify with `harv config set`.

| Setting | Default | Description |
|---------|---------|-------------|
| `cache-ttl` | `24` | Cache lifetime in hours (0 = always fetch) |

Project assignments are cached with the configured TTL. Subsequent `track`/`start`/`log` commands return instantly. Use `--refresh` to bypass the cache.

### Custom OAuth2 Application

By default `harv` ships with a built-in Harvest OAuth2 client ID. To use your own application (registered at [id.getharvest.com/developers](https://id.getharvest.com/developers)), set the `HARV_CLIENT_ID` environment variable at compile time:

```bash
HARV_CLIENT_ID="your-app-id" cargo install --git https://github.com/josbeir/harv harv-cli
```

When registering your app, set the redirect URI to `http://localhost:5006`.

## Development

### Prerequisites

- Rust 1.80+

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

### Coverage

```bash
cargo tarpaulin --workspace
```

## Architecture

```
harv-core (domain types, errors)
  ↓
harv-sdk  (Harvest API v2 client)
  ↓
harv-cli  (CLI binary)
```

## Disclaimer

This project is **not affiliated, associated, authorized, endorsed by, or in any way officially connected** with [Harvest](https://www.getharvest.com/) or its parent company. "Harvest" is a registered trademark of Iridesco, LLC. This is an independent, community-built CLI client for the Harvest public API.

## License

MIT
