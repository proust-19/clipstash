# 📋 ClipStash

A fast, lightweight, keyboard-first clipboard history manager for Linux (Wayland & X11).

Built with **Rust** and **egui**, ClipStash offers both a high-performance CLI tool and a sleek floating GUI desktop overlay.

---

## ✨ Features

- 🪟 **Floating Desktop Overlay & Floating Bubble Mode**:
  - **Full Overlay**: Resizable, floating desktop window with dark theme aesthetics.
  - **Compact Chat Head / Bubble Mode**: Collapse to a minimal desktop widget (`160x52px`) and expand on click.
  - **Always-on-Top Toggle**: Pin window to stay on top (`📌 Pinned` / `📍 Float`).
- 📐 **Responsive Text Wrapping**: Long text entries automatically wrap nicely on small or narrow screen sizes without clipping action buttons.
- ↕️ **Multi-line Expand & Collapse**: Multi-line clipboard entries (> 2 lines) include collapsible cards (`▼ Expand` / `▲ Collapse`).
- 📌 **Pin & Protect Entries**: Pin important text snippets so they are preserved across history clear operations.
- 🔍 **Real-time Instant Search**: Search through your entire clipboard history instantaneously.
- 📋 **Single-Click Copying & Toasts**: Click any card or the `📋` button to copy snippets back to your clipboard with clean notification toasts.
- 🗑️ **One-Click Deletion**: Delete individual items (`🗑`) or clear all unpinned entries in one tap.
- ⚙️ **Dual Interface (CLI & GUI)**: Use command-line workflows or the floating GUI interface seamlessly.
- 🔄 **Automatic Clipboard Monitoring**: Real-time clipboard polling daemon for Wayland & X11 compositors (Hyprland, Sway, GNOME, KDE, etc.).

---

## 🚀 Installation & Setup

### Prerequisites

Ensure Rust and Cargo are installed on your system. If not, install via [rustup.rs](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install from Source

```bash
# 1. Clone repository
git clone https://github.com/proust-19/clipstash.git
cd ClipStash

# 2. Build and install binary to ~/.cargo/bin
cargo install --path .

# 3. (Optional) Copy binary to ~/.local/bin for global environment access
mkdir -p ~/.local/bin
cp ~/.cargo/bin/clipstash ~/.local/bin/clipstash
```

### Create Desktop Application Shortcut

To launch ClipStash from your application launcher (Rofi, dmenu, GNOME App Grid, etc.) or pin it to your desktop panel:

1. Create a `.desktop` file at `~/.local/share/applications/clipstash.desktop`:
```ini
[Desktop Entry]
Name=ClipStash
Comment=Clipboard History Manager
Exec=/home/YOUR_USERNAME/.local/bin/clipstash gui
Icon=edit-paste
Terminal=false
Type=Application
Categories=Utility;
Keywords=clipboard;copy;paste;history;
StartupNotify=true
```
*(Replace `YOUR_USERNAME` with your actual username or use `$HOME/.local/bin/clipstash gui` if supported).*

2. Update desktop database:
```bash
update-desktop-database ~/.local/share/applications/
```

> ⚠️ **Note on Updating**: When updating ClipStash source code, run `cargo install --path .` and sync the updated binary to `~/.local/bin/clipstash`:
> ```bash
> cargo install --path . && cp ~/.cargo/bin/clipstash ~/.local/bin/clipstash
> ```

---

## 💻 Usage

### 1. Floating GUI Interface

Launch the interactive overlay window (default mode):
```bash
clipstash
# or explicitly:
clipstash gui
```

### 2. Clipboard Daemon

Run the background daemon to monitor and record clipboard changes automatically:
```bash
clipstash daemon --max-entries 100
```

#### Systemd User Service (Recommended)
To run ClipStash daemon automatically at login, create `~/.config/systemd/user/clipstash.service`:
```ini
[Unit]
Description=ClipStash Clipboard Monitoring Daemon
After=graphical-session.target

[Service]
ExecStart=%h/.local/bin/clipstash daemon
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

Enable and start the service:
```bash
systemctl --user daemon-reload
systemctl --user enable --now clipstash.service
```

### 3. CLI Commands

```bash
# List recent clipboard entries (default limit: 20)
clipstash list

# Search clipboard history
clipstash list --search "query" --limit 50

# Copy an entry back to clipboard by ID
clipstash select <ID>

# Show the most recent copied item
clipstash latest

# Show storage status and statistics
clipstash status

# Clear unpinned entries (or clear all with --keep-pinned=false)
clipstash clear
```

---

## ⚙️ Configuration

### Storage Path
ClipStash stores history as JSON.
- **Default path**: `~/.local/share/clipstash/history.json`
- Custom history path:
  ```bash
  clipstash --history-file /custom/path/history.json list
  ```

### Max Entries limit
- Default limit: `100` entries.
- Change limit during daemon startup:
  ```bash
  clipstash daemon --max-entries 200
  ```

---

## ⌨️ Compositor Keybindings

Bind a global hotkey to pop up ClipStash instantly:

### Hyprland (`~/.config/hypr/hyprland.conf`)
```ini
bind = $mainMod, V, exec, clipstash gui
```

### Sway (`~/.config/sway/config`)
```bash
bindsym $mod+v exec clipstash gui
```

### GNOME / KDE / X11
Add a Custom Shortcut in System Settings:
- **Name**: ClipStash
- **Command**: `clipstash gui` (or `/home/username/.local/bin/clipstash gui`)
- **Shortcut**: `Super + V`

---

## 🛠 Integration with Launchers

### Rofi
```bash
clipstash list --limit 30 | rofi -dmenu -p "Clipboard" | awk -F'.' '{print $1}' | xargs -I {} clipstash select {}
```

### FZF
```bash
clipstash list --limit 30 | fzf | awk -F'.' '{print $1}' | xargs -I {} clipstash select {}
```

### Wofi (Wayland)
```bash
clipstash list --limit 30 | wofi -d -p "Clipboard" | awk -F'.' '{print $1}' | xargs -I {} clipstash select {}
```

---

## 🛠️ Development & Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run unit tests
cargo test
```

---

## 📄 License

Distributed under the [MIT License](LICENSE).

