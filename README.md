# ClipStash

A fast, keyboard-first clipboard history manager for Wayland/Linux.

## Features

- **Floating GUI Window**: Launch interactive overlay window with single-click copy & deletion
- **Clipboard Monitoring**: Automatically captures clipboard content
- **History Management**: Keep track of copied text
- **Search & Filter**: Find previously copied content instantly
- **Pin Entries**: Keep important items pinned in history
- **CLI & GUI Interface**: Fast keyboard-driven CLI and visual GUI floating window
- **Wayland & X11 Support**: Works seamlessly on GNOME, Hyprland, Sway, KDE, etc.


## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/proust-19/clipstash.git
cd clipstash

# Build and install
cargo install --path .
```

### From AUR (Arch Linux)

```bash
yay -S clipstash
```

### Flatpak

```bash
flatpak install flathub com.clipstash.ClipStash
```

## Usage

### Launch Floating GUI Window
```bash
# Launch the floating window GUI (default when run without arguments)
clipstash
# or explicitly:
clipstash gui
```

### Start the Daemon
```bash
clipstash daemon
```


The daemon runs in the background and monitors your clipboard for changes.

### Systemd Service (Recommended)

```bash
systemctl --user enable --now clipstash.service
```

### CLI Commands

```bash
# List clipboard history
clipstash list

# Search clipboard history
clipstash list --search "query"

# Copy an entry to clipboard
clipstash select <id>

# Show the latest entry
clipstash latest

# Clear history
clipstash clear

# Show status
clipstash status
```

## Configuration

### History File Location

Default: `~/.local/share/clipstash/history.json`

Override with:
```bash
clipstash --history-file /path/to/history.json list
```

### Maximum Entries

Default: 100 entries

Override with:
```bash
clipstash daemon --max-entries 200
```

## Integration with Other Tools

### Rofi

Create a script `clipstash-select.sh`:
```bash
#!/bin/bash
clipstash list --limit 20 | rofi -dmenu -p "Clipboard" | awk '{print $1}' | xargs -I {} clipstash select {}
```

### FZF

```bash
clipstash list --limit 20 | fzf | awk '{print $1}' | xargs -I {} clipstash select {}
```

### Wofi

```bash
clipstash list --limit 20 | wofi -d -p "Clipboard" | awk '{print $1}' | xargs -I {} clipstash select {}
```

## Wayland Compositor Keybinds

### Hyprland

```ini
bind = $mainMod, V, exec, clipstash toggle
```

### Sway

```bash
bindsym $mod+v exec clipstash toggle
```

### GNOME

Use GNOME Settings > Keyboard > Custom Shortcuts.

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## License

MIT
