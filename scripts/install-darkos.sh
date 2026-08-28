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

# Grant input device permissions for non-root users (like 'ark')
echo "Setting up input & console permissions..."
usermod -a -G input,tty,video ark || true
if [ -n "$SUDO_USER" ]; then
    usermod -a -G input,tty,video "$SUDO_USER" || true
fi
chmod 666 /dev/input/event* || true
chmod 666 /dev/tty1 || true
cat << 'UDEV' > /etc/udev/rules.d/99-gamepad-input.rules
KERNEL=="event*", SUBSYSTEM=="input", MODE="0666"
KERNEL=="tty1", MODE="0666"
UDEV
udevadm control --reload-rules || true
udevadm trigger || true

# Install compact fonts and tmux if missing
echo "Ensuring console fonts and tmux are installed..."
DEBIAN_FRONTEND=noninteractive apt-get update -qq && \
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq -o Dpkg::Options::="--force-confdef" -o Dpkg::Options::="--force-confold" kbd console-data console-setup fonts-terminus tmux || true

# Create EmulationStation Ports / Tools launcher shortcut if folders exist
for dir in /roms/ports /roms/tools /roms2/ports /roms2/tools /storage/roms/ports /userdata/roms/ports; do
    if [ -d "$dir" ]; then
        echo "Creating EmulationStation shortcut in $dir/Cyberdeck.sh..."
        cat << 'PORT' > "$dir/Cyberdeck.sh"
#!/bin/bash
# 1. Switch VT and ensure TTY1 is accessible
sudo chmod 666 /dev/tty1 2>/dev/null || true
sudo chvt 1 2>/dev/null || true

# 2. Set compact font on /dev/tty1
sudo setfont -C /dev/tty1 /usr/share/consolefonts/Uni3-Terminus12.psf.gz 2>/dev/null || \
sudo setfont -C /dev/tty1 /usr/share/consolefonts/Uni3-TerminusBold14.psf.gz 2>/dev/null || \
sudo setfont -C /dev/tty1 /usr/share/consolefonts/Uni3-Terminus16.psf.gz 2>/dev/null || \
sudo setfont -C /dev/tty1 /usr/share/consolefonts/Uni3-Terminus8x8.psf.gz 2>/dev/null || \
sudo setfont -C /dev/tty1 /usr/share/consolefonts/default8x16.psf.gz 2>/dev/null || true

# 3. Direct all I/O to the physical framebuffer console
exec > /dev/tty1 2>&1 < /dev/tty1

# 4. Clear and launch Cyberdeck
clear
printf "\033[?25h"
/opt/cyberdeck/bin/cyberdeck-kb
PORT
        chmod +x "$dir/Cyberdeck.sh"
    fi
done

echo "[3/4] Writing configuration ($CONFIG_DIR/config.toml)..."
cat << 'CFG' > "$CONFIG_DIR/config.toml"
# Cyberdeck-KB & DarkOS Configuration
# Default to tmux session 'main' for persistence & pairing over SSH
shell = "tmux new-session -A -s main"
gamepad_device = "/dev/input/event4"
keyboard_height_ratio = 0.45
min_keyboard_height = 9
max_keyboard_height = 14
repeat_delay_ms = 250
repeat_rate_ms = 50
CFG

# Setup friendly ~/.tmux.conf with touch/mouse split switching & clean style
for home_dir in /home/ark /root; do
    if [ -d "$home_dir" ] && [ ! -f "$home_dir/.tmux.conf" ]; then
        cat << 'TMUXCFG' > "$home_dir/.tmux.conf"
# Enable mouse / touchscreen to click between split panes
set -g mouse on
set -g default-terminal "xterm-256color"
set -g history-limit 10000
set -g status-bg colour235
set -g status-fg colour136
TMUXCFG
        if [ "$home_dir" = "/home/ark" ]; then
            chown ark:ark /home/ark/.tmux.conf || true
        fi
    fi
done

echo "[4/4] Setting up systemd background service..."
cat << 'SVC' > "$SERVICE_PATH"
[Unit]
Description=DarkOS Cyberdeck Mode Launcher Daemon
After=multi-user.target

[Service]
Type=simple
ExecStart=/opt/cyberdeck/bin/deck-launcher --device /dev/input/event4 --process emulationstation --cyberdeck /opt/cyberdeck/bin/cyberdeck-kb
Restart=always
RestartSec=3
StandardInput=null
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
