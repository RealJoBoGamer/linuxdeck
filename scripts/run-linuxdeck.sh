#!/usr/bin/env bash
# Run the locally built LinuxDeck binary.
# On SteamOS, prefer Distrobox when host WebKitGTK is missing.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/src-tauri/target/release/linuxdeck"

if [[ ! -x "$BIN" ]]; then
	echo "Binary not found: $BIN" >&2
	echo "Build first: distrobox enter archbuild -- bash -lc 'cd $ROOT && deno task tauri build'" >&2
	exit 1
fi

if ldd "$BIN" 2>/dev/null | grep -q 'not found'; then
	if command -v distrobox >/dev/null 2>&1; then
		exec distrobox enter archbuild -- "$BIN" "$@"
	fi
	echo "Missing shared libraries for $BIN (install webkit2gtk-4.1 or use Distrobox)." >&2
	exit 1
fi

exec "$BIN" "$@"
