# colored-logcat

A blazing-fast TUI for Android logcat, built in Rust.

![1.6MB binary](https://img.shields.io/badge/binary_size-1.6MB-brightgreen)
![Rust](https://img.shields.io/badge/built_with-Rust-orange)

## Features

- **Color-coded log levels** — Verbose, Debug, Info, Warn, Error, Fatal each get distinct colors
- **Interactive filtering** — regex pattern, tag, and package name filters with live input
- **Level toggles** — enable/disable individual log levels with `1`-`6` keys
- **Package filtering** — filter by app package name with automatic PID resolution
- **JSON syntax highlighting** — detects JSON in log messages and colorizes keys, strings, numbers, booleans
- **Scrollback & freeze** — pause the stream, scroll through history, resume tailing
- **Multi-panel layout** — split view with crash/ANR panel or device list sidebar
- **Export** — save filtered logs to a timestamped file
- **Crash monitoring** — dedicated panel for crashes, ANRs, and fatal errors
- **Tiny footprint** — ~1.6MB release binary, ~100k entry ring buffer

## Requirements

- [ADB](https://developer.android.com/tools/adb) on your PATH
- An Android device connected via USB with USB debugging enabled

## Installation

```bash
# Clone and build
git clone https://github.com/sharif-smj/colored-logcat.git
cd colored-logcat
cargo build --release

# Copy to your PATH
cp target/release/colored-logcat.exe ~/.cargo/bin/
```

## Usage

```bash
colored-logcat
```

## Keybindings

| Key | Action |
|-----|--------|
| `?` | Show help overlay |
| `/` | Filter by regex pattern |
| `t` | Filter by tag |
| `p` | Filter by package name |
| `1`-`6` | Toggle log levels V/D/I/W/E/F |
| `Space` | Pause / Resume tailing |
| `j`/`k` or `↑`/`↓` | Scroll (when paused) |
| `PgUp` / `PgDn` | Page scroll |
| `Home` | Jump to top |
| `End` / `G` | Jump to bottom / resume tailing |
| `x` | Toggle crash/ANR panel |
| `d` | Toggle device panel |
| `s` | Save visible logs to file |
| `c` | Clear logcat buffer |
| `Esc` | Clear filters / cancel input |
| `q` / `Ctrl+C` | Quit |

## License

MIT
