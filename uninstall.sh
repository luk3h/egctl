#!/usr/bin/env bash
# Remove egctl and egctl-gui for the current user.
set -uo pipefail

cargo uninstall egctl 2>/dev/null || true
rm -f "$HOME/.local/bin/egctl" "$HOME/.local/bin/egctl-gui"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/egctl.svg"
rm -f "$HOME/.local/share/applications/egctl.desktop"
command -v kbuildsycoca6 >/dev/null && kbuildsycoca6 --noincremental >/dev/null 2>&1 || true

echo "Removed egctl. The udev rule is left in place; remove it with:"
echo "  sudo rm /etc/udev/rules.d/60-endgamegear.rules"
