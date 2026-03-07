# notes — Customization Guide

notes is a personal notes harness with CRUD and search. Ships with JSON file storage
and a dark mobile-friendly UI. Three ports to customize.

## Ports

### `APP_COLOR`

**What it does:** Primary accent color for the UI — buttons, focus rings, the bottom bar label.
**Default:** `#7c6af7` (indigo)
**How to customize:** `APP_COLOR=#e05555 ./serve.sh`

Any valid CSS hex color. The dimmed variant (`--accent-dim`) is derived automatically
via `color-mix`, so only one variable is needed.

### `PORT`

**What it does:** TCP port the HTTP server binds to.
**Default:** `3001` (set by EPC via `PORT` env var)
**How to customize:** `PORT=9090 ./serve.sh`

### `DATA_DIR`

**What it does:** Directory where `notes.json` is stored.
**Default:** The working directory (project root).
**How to customize:** `NOTES_DATA_DIR=/Users/nick/.notes ./serve.sh`

The resolution is in `fn notes_file()` in `src/main.rs`.

## Getting Started

```sh
cargo build --release
./target/release/notes
# open http://localhost:3001
```

Or deploy via EPC:

```sh
epc deploy notes --local /path/to/notes
```

## Common Customizations

### Stable data directory

```sh
mkdir -p ~/.notes
NOTES_DATA_DIR=~/.notes ./serve.sh
```

### Add tags to notes

Extend the `Note` struct in `src/main.rs` to add a `tags: Vec<String>` field.
Update the `notes.html` card template to render and filter by tags.
