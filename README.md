# spew

**Stream Parser & Event Watcher**

spew is a terminal UI for viewing and searching JSON log streams in real time. You pipe a log source into it and get a live, scrollable, filterable table instead of a wall of raw JSON.

<img width="781" height="820" alt="demo" src="https://github.com/user-attachments/assets/683c3250-2161-4a9e-b0be-8a8d56ad3ac8" />

```bash
kubectl logs -f pod/backend-xyz | spew
```

---

## The Problem

When a backend throws an intermittent error under load, the logs are flying past at hundreds of lines per second. Raw JSON is unreadable. `grep` finds the error but strips everything that happened before it. Datadog costs money. Nothing just works from a pipe.

spew sits between your log source and your eyes. It parses the stream, renders it cleanly, and lets you freeze time, filter down to what matters, and inspect any entry in full without leaving the terminal.

---

## Features

- **Zero config** -> pipe anything in and it works
- **Auto-parses JSON** -> renders structured fields in a readable table, falls back to raw text for anything it cannot parse
- **Freeze the stream** -> hit `Space` to lock the viewport at an exact moment in time while ingestion keeps running silently in the background
- **Live filter** -> press `/` and type to filter, results update on every keystroke
- **Context lines** -> matched lines show the 5 lines before and after them so you never lose what happened around a crash
- **Expand any entry** -> press `Enter` on a line to open a syntax-highlighted JSON panel with the full payload
- **Gruvbox theme** -> baked in by default, fully overridable via a config file
- **Single binary** -> no runtime, no Docker, no dependencies

---

## Install

### From source

```bash
git clone https://github.com/yourusername/spew
cd spew
cargo build --release
cp target/release/spew ~/.local/bin/
```

### Cargo

```bash
cargo install spew
```

---

## Usage

```bash
# Kubernetes
kubectl logs -f pod/backend-xyz | spew

# Docker
docker logs -f my-container | spew

# Any file
tail -f /var/log/app/production.log | spew

# Local dev
cargo run 2>&1 | spew
```

---

## Keybindings

| Key | Action |
|-----|--------|
| `Space` | Freeze / unfreeze the stream |
| `/` | Open filter bar |
| `Esc` | Clear filter and return to live |
| `↑ ↓` | Navigate lines (auto-freezes) |
| `Enter` | Expand or collapse the detail panel |
| `q` | Quit |

---

## Theme

spew ships with Gruvbox dark as the default. To override any color, create `~/.config/spew/config.toml`:

```toml
[colors]
error = "#cc241d"
warn  = "#d79921"
info  = "#928374"
dim   = "#504945"
db    = "#458588"
auth  = "#b16286"
conn  = "#d65d0e"
ok    = "#98971a"
```

Any field you leave out falls back to the Gruvbox default.

---

## Log Format Support

spew currently parses JSON logs that use these field names:

| Field | Meaning |
|-------|---------|
| `level` | Log severity as a string: `info`, `warn`, `error` |
| `msg` | The log message |
| `ts` | Timestamp |

This matches the output of **Go backends using zap or zerolog**, which is the standard for Kubernetes workloads.

Lines that do not match this format are displayed as raw text without breaking the stream.

### Known Limitations

- **Pino (Node.js)** uses numeric levels (`30` for info, `50` for error) instead of strings. These lines will render but will not be color-coded by severity.
- **structlog (Python)** uses `event` instead of `msg` and `timestamp` instead of `ts`. These lines will render as raw text.
- **journalctl** uses systemd field names (`MESSAGE`, `PRIORITY`) which spew does not map. Pipe output will show as raw text.
- The detail panel does not scroll. If a JSON payload is taller than half your terminal, the bottom will be cut off.
- No support for logfmt (`key=value` format) yet.

Support for more formats is planned. If your stack uses a different format, open an issue with a sample log line.

---

## Roadmap

- [ ] Pino numeric level support
- [ ] structlog field mapping
- [ ] logfmt parsing
- [ ] Scrollable detail panel
- [ ] Export filtered results to file
- [ ] Multiple pane support

---

## License

MIT
