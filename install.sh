#!/usr/bin/env bash
# Install egctl (CLI) and egctl-gui for the current user.
set -euo pipefail
cd "$(dirname "$0")"

BIN_DIR="$HOME/.local/bin"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
APP_DIR="$HOME/.local/share/applications"
RULE=/etc/udev/rules.d/60-endgamegear.rules

if ! command -v cargo >/dev/null; then
    echo "error: cargo not found. Install Rust first (see README)." >&2
    exit 1
fi

echo "==> Building egctl and egctl-gui (first build takes a few minutes)"
cargo install --path . --features gui --locked

echo "==> Linking into $BIN_DIR"
mkdir -p "$BIN_DIR"
ln -sf "$HOME/.cargo/bin/egctl" "$BIN_DIR/egctl"
ln -sf "$HOME/.cargo/bin/egctl-gui" "$BIN_DIR/egctl-gui"

echo "==> Installing icon and app menu entry"
install -Dm644 assets/egctl.svg "$ICON_DIR/egctl.svg"
mkdir -p "$APP_DIR"
sed -e "s|^Exec=.*|Exec=$BIN_DIR/egctl-gui|" \
    -e "s|^Icon=.*|Icon=$ICON_DIR/egctl.svg|" \
    assets/egctl.desktop > "$APP_DIR/egctl.desktop"

if [ ! -f "$RULE" ]; then
    echo "==> Installing udev rule (needs sudo)"
    sudo install -m644 60-endgamegear.rules "$RULE"
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    echo "    Unplug and replug the mouse once."
fi

# Refresh the app menu (KDE, then others)
command -v kbuildsycoca6 >/dev/null && kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
command -v update-desktop-database >/dev/null && update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true

echo "==> Done. Run 'egctl info', or open 'egctl' from your app menu."
