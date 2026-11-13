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

## Troubleshooting

### White window: `Could not connect to localhost: Connection refused`

Caused by webkit2gtk **2.46+** breaking Tauri’s `tauri://` custom scheme (Arch / SteamOS Distrobox). LinuxDeck serves the UI over `http://127.0.0.1:<port>` instead, and also vendors a patched `wry`.

Kill old processes, then relaunch the rebuilt binary:

```bash
pkill -f target/release/linuxdeck || true
/home/deck/linuxdeck/scripts/run-linuxdeck.sh
```

Do **not** use the AppImage on SteamOS (glibc mismatch).

## Official Elgato plugins on Linux

LinuxDeck runs normal Stream Deck SDK plugins (not only OpenDeck-specific ones):

| Plugin type | Needs | How LinuxDeck runs it |
|-------------|--------|------------------------|
| Linux binary (`CodePathLin`) | — | Native process |
| Node.js (`.js` / `.mjs` / `.cjs`) | **Node.js 20+** on PATH (nvm/fnm OK) | Host `node` |
| Windows-only (`.exe`) | **Wine + Wine Mono** | Wine + shared prefix mimicking `%AppData%\Elgato\StreamDeck\Plugins` |

Tips:

- Install plugins via the in-app Plugins UI (`.streamDeckPlugin` / marketplace zip) or drop a `*.sdPlugin` folder into `~/.config/linuxdeck/plugins/`.
- Settings → **Elgato plugin runtimes** shows whether Wine/Node were found.
- Prefer the shared Wine prefix (default) so plugins that probe Elgato’s Windows AppData paths keep working.
- Flatpak builds must use **host** Wine/Node (not Flatpak Wine/Node).

## Upstream

- OpenDeck by Aman Khanna (nekename): https://github.com/nekename/OpenDeck
- Please star / sponsor upstream if this software helps you

## License

GNU GPL v3 or later — see `LICENSE.md`. Same license as OpenDeck.
