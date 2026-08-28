#!/bin/bash
set -e

echo "========================================================="
echo "   Installing Antigravity CLI (agy) on Linux ARM64       "
echo "========================================================="

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR" /tmp/agy_install

echo "[1/3] Downloading release package for linux_arm64..."
python3 - << 'PYEOF'
import urllib.request
import sys

url = "https://storage.googleapis.com/antigravity-public/antigravity-cli/1.1.22-5711547746615296/linux-arm/cli_linux_arm64.tar.gz"
dest = "/tmp/agy_install/agy.tar.gz"

def report(count, block_size, total_size):
    if total_size > 0:
        percent = int(count * block_size * 100 / total_size)
        sys.stdout.write(f"\rDownloading: {percent}% [{count * block_size / (1024*1024):.1f}MB / {total_size / (1024*1024):.1f}MB]")
        sys.stdout.flush()

urllib.request.urlretrieve(url, dest, reporthook=report)
print("\nDownload complete!")
PYEOF

echo "[2/3] Extracting package..."
rm -rf /tmp/agy_install/extracted
mkdir -p /tmp/agy_install/extracted
tar -xzf /tmp/agy_install/agy.tar.gz -C /tmp/agy_install/extracted

echo "[3/3] Installing binary to $BIN_DIR/agy..."
if [ -f /tmp/agy_install/extracted/antigravity ]; then
    cp /tmp/agy_install/extracted/antigravity "$BIN_DIR/agy"
elif [ -f /tmp/agy_install/extracted/agy ]; then
    cp /tmp/agy_install/extracted/agy "$BIN_DIR/agy"
else
    find /tmp/agy_install/extracted -type f -executable -exec cp {} "$BIN_DIR/agy" \;
fi

chmod +x "$BIN_DIR/agy"
sudo ln -sf "$BIN_DIR/agy" /usr/local/bin/agy 2>/dev/null || true

rm -rf /tmp/agy_install

echo "========================================================="
echo " ✅ Antigravity CLI installed successfully!              "
echo "========================================================="
echo " Location: $BIN_DIR/agy"
echo " To run:   agy"
echo "========================================================="
