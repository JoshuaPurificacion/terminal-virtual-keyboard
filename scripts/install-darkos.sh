#!/bin/bash
set -e

INSTALL_DIR="/opt/cyberdeck"
BIN_DIR="$INSTALL_DIR/bin"
CONFIG_DIR="/etc/cyberdeck-kb"
SERVICE_PATH="/etc/systemd/system/deck-launcher.service"

echo "========================================================="
echo "   DarkOS / RG353M Cyberdeck Integration Installer       "
echo "========================================================="

# Ensure running as root
if [ "$(id -u)" -ne 0 ]; then
    echo "Please run this script as root (e.g. sudo bash install-darkos.sh)"
    exit 1
fi

echo "[1/4] Creating installation directories..."
mkdir -p "$BIN_DIR"
mkdir -p "$CONFIG_DIR"

echo "[2/4] Installing binaries..."
if [ ! -f "target/release/cyberdeck-kb" ] || [ ! -f "target/release/deck-launcher" ]; then
    echo "Compiling release binaries..."
    if [ -n "$SUDO_USER" ]; then
        sudo -u "$SUDO_USER" cargo build --release || cargo build --release
    else
        cargo build --release
    fi
fi

cp target/release/cyberdeck-kb "$BIN_DIR/"
cp target/release/deck-launcher "$BIN_DIR/"

chmod +x "$BIN_DIR/cyberdeck-kb"
chmod +x "$BIN_DIR/deck-launcher"

# Create symlinks in /usr/local/bin
ln -sf "$BIN_DIR/cyberdeck-kb" /usr/local/bin/cyberdeck-kb
ln -sf "$BIN_DIR/deck-launcher" /usr/local/bin/deck-launcher

echo "[3/4] Writing default configuration ($CONFIG_DIR/config.toml)..."
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat << 'CFG' > "$CONFIG_DIR/config.toml"
# Cyberdeck-KB & DarkOS Configuration
shell = "/bin/bash"
gamepad_device = "/dev/input/event4"
keyboard_height_ratio = 0.45
min_keyboard_height = 9
max_keyboard_height = 14
repeat_delay_ms = 250
repeat_rate_ms = 50
CFG
fi

echo "[4/4] Setting up systemd background service..."
cat << 'SVC' > "$SERVICE_PATH"
[Unit]
Description=DarkOS Cyberdeck Mode Launcher Daemon
After=multi-user.target

[Service]
Type=simple
ExecStart=/opt/cyberdeck/bin/deck-launcher --process emulationstation --cyberdeck /opt/cyberdeck/bin/cyberdeck-kb
Restart=always
RestartSec=3
StandardInput=tty
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SVC

systemctl daemon-reload || true

echo ""
echo "========================================================="
echo " ✅ Installation Complete!                               "
echo "========================================================="
echo " To enable at boot:    sudo systemctl enable deck-launcher"
echo " To start now:         sudo systemctl start deck-launcher"
echo " To test manually:     /opt/cyberdeck/bin/deck-launcher"
echo " Hotkey in ES:         Press [SELECT + START] or [F]"
echo " Exit Cyberdeck Mode:  Press [Ctrl + Q] on virtual keyboard"
echo "========================================================="
