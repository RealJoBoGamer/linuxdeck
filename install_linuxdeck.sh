#!/usr/bin/env bash
# LinuxDeck helper installer for Linux / SteamOS.
# Prefers Flatpak OpenDeck (compatible) until LinuxDeck publishes its own releases,
# and always offers to install Stream Deck udev rules.
set -euo pipefail

GITHUB_REPO="RealJoBoGamer/linuxdeck"
UPSTREAM_FLATPAK="me.amankhanna.opendeck"
UDEV_RULES_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main/src-tauri/bundle/40-streamdeck.rules"

if [ -t 1 ]; then
	RED="\033[0;31m"; GREEN="\033[0;32m"; YELLOW="\033[0;33m"; BLUE="\033[0;34m"; BOLD="\033[1m"; RESET="\033[0m"
else
	RED=""; GREEN=""; YELLOW=""; BLUE=""; BOLD=""; RESET=""
fi

msg_info() { echo -e "${BLUE}[*]${RESET} $*"; }
msg_ok() { echo -e "${GREEN}[✓] $*${RESET}"; }
msg_warn() { echo -e "${YELLOW}[!] $*${RESET}"; }
msg_error() { echo -e "${RED}[✗] $*${RESET}" >&2; }

has_cmd() { command -v "$1" >/dev/null 2>&1; }

confirm() {
	printf "%b$1 [y/N]: %b" "$YELLOW$BOLD" "$RESET"
	local ans
	if [ -r /dev/tty ]; then read -r ans </dev/tty; else read -r ans; fi
	case "$ans" in [yY]|[yY][eE][sS]) return 0 ;; *) return 1 ;; esac
}

is_steamos() {
	grep -qiE 'steamos|holo' /etc/os-release 2>/dev/null
}

install_udev() {
	msg_info "Installing Stream Deck udev rules..."
	local tmp
	tmp="$(mktemp)"
	curl -fsSL "$UDEV_RULES_URL" -o "$tmp"
	if ! has_cmd sudo; then
		msg_error "sudo required to install udev rules"
		rm -f "$tmp"
		return 1
	fi
	sudo cp "$tmp" /etc/udev/rules.d/40-streamdeck.rules
	sudo udevadm control --reload-rules
	sudo udevadm trigger || true
	rm -f "$tmp"
	msg_ok "udev rules installed — reconnect your Stream Deck or reboot"
}

install_flatpak_compat() {
	if ! has_cmd flatpak; then
		msg_warn "flatpak not found; skip Flatpak install"
		return 1
	fi
	msg_info "Installing compatible OpenDeck Flatpak (${UPSTREAM_FLATPAK})..."
	flatpak install -y flathub "$UPSTREAM_FLATPAK"
	msg_ok "OpenDeck Flatpak installed (LinuxDeck migrates its config on first native run)"
}

echo -e "${BOLD}LinuxDeck installer${RESET}"
echo "Repo: https://github.com/${GITHUB_REPO}"
if is_steamos; then
	msg_info "SteamOS / Holo detected — use Desktop Mode for Stream Deck apps"
fi

if confirm "Install udev rules for Elgato Stream Deck hardware?"; then
	install_udev || true
fi

if confirm "Install OpenDeck Flatpak for immediate use (recommended until LinuxDeck releases)?"; then
	install_flatpak_compat || true
fi

msg_ok "Done. Build native LinuxDeck with: deno task tauri build"
