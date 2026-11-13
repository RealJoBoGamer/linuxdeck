#!/usr/bin/env bash
echo "install_opendeck.sh is retained for upstream compatibility."
echo "Prefer: ./install_linuxdeck.sh"
exec "$(dirname "$0")/install_linuxdeck.sh" "$@"
