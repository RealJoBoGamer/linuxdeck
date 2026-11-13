# LinuxDeck

**LinuxDeck** is a Linux-focused fork of [OpenDeck](https://github.com/nekename/OpenDeck) — open-source Stream Deck host software that runs official Elgato Stream Deck plugins.

This fork keeps the OpenDeck codebase and plugin compatibility, with SteamOS / Steam Deck oriented docs, packaging names, OpenDeck config migration, and no upstream Aptabase telemetry.

> Hardware shown working on this machine: **Elgato Stream Deck MK.2** (`0fd9:0080`).

## Features (from OpenDeck)

- Official Stream Deck SDK plugin support (including many Windows-only plugins via Wine)
- Multi Actions, Toggle Actions, profiles, app-based profile switching, brightness control
- Cross-platform Tauri app (this fork prioritises Linux / SteamOS)
- GPL-3.0-or-later

## Quick start on Steam Deck / SteamOS

Use **Desktop Mode**.

### Option A — run OpenDeck Flatpak today (compatible)

LinuxDeck can migrate OpenDeck configs on first run. Until LinuxDeck has its own Flatpak/release build:

```bash
flatpak install -y flathub me.amankhanna.opendeck
```

Install udev rules (needs sudo password once):

```bash
curl -fsSL https://raw.githubusercontent.com/RealJoBoGamer/linuxdeck/main/src-tauri/bundle/40-streamdeck.rules \
  | sudo tee /etc/udev/rules.d/40-streamdeck.rules >/dev/null
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then unplug/replug the Stream Deck (or reboot) and launch OpenDeck / LinuxDeck.

### Option B — build LinuxDeck from source

On SteamOS, build inside Distrobox (host root is awkward for WebKitGTK deps):

```bash
git clone https://github.com/RealJoBoGamer/linuxdeck.git
cd linuxdeck
distrobox enter archbuild -- bash -lc '
  source "$HOME/.cargo/env"
  export PATH="$HOME/.deno/bin:$PATH"
  cd /home/deck/linuxdeck
  deno install
  deno task tauri build
'
```

Binary output is under `src-tauri/target/release/linuxdeck` (and bundle artifacts under `src-tauri/target/release/bundle/`).

## Upstream

- OpenDeck by Aman Khanna (nekename): https://github.com/nekename/OpenDeck
- Please star / sponsor upstream if this software helps you

## License

GNU GPL v3 or later — see `LICENSE.md`. Same license as OpenDeck.
