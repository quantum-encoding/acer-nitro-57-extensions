#!/bin/bash
set -e

# Installation Script
# For Acer Nitro AN515-57 Only

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "========================================="
echo " Sovereign Control for Acer Nitro AN515-57"
echo " Installation Script"
echo "========================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "ERROR: This script must be run as root (use sudo)"
    exit 1
fi

# Detect hardware
echo "[1/8] Detecting hardware..."
PRODUCT_NAME=$(cat /sys/class/dmi/id/product_name 2>/dev/null || echo "Unknown")
echo "      Detected: $PRODUCT_NAME"

if [[ ! "$PRODUCT_NAME" =~ "Nitro AN515-57" ]]; then
    echo ""
    echo "⚠️  WARNING: This software is designed ONLY for Acer Nitro AN515-57"
    echo "   Detected hardware: $PRODUCT_NAME"
    echo ""
    echo "   Installing on unsupported hardware may cause:"
    echo "   - Hardware damage"
    echo "   - System instability"
    echo "   - Permanent component failure"
    echo ""
    read -p "Are you ABSOLUTELY SURE you want to continue? (type 'YES' to proceed): " confirm
    if [ "$confirm" != "YES" ]; then
        echo "Installation aborted."
        exit 1
    fi
fi

# Check for required dependencies
echo "[2/8] Checking dependencies..."
MISSING_DEPS=()

if ! command -v systemctl &> /dev/null; then
    MISSING_DEPS+=("systemd")
fi

if ! command -v busctl &> /dev/null; then
    MISSING_DEPS+=("dbus")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "ERROR: Missing dependencies: ${MISSING_DEPS[*]}"
    exit 1
fi
echo "      ✓ All dependencies found"

# Install Boreas daemon
echo "[3/8] Installing Boreas daemon..."
if [ -f "$PROJECT_ROOT/daemons/boreas/target/release/boreas-daemon" ]; then
    cp "$PROJECT_ROOT/daemons/boreas/target/release/boreas-daemon" /usr/local/bin/
    chmod 755 /usr/local/bin/boreas-daemon
    echo "      ✓ Binary installed"
else
    echo "ERROR: Boreas binary not found. Did you run 'cargo build --release'?"
    exit 1
fi

# Install Boreas service files
cp "$PROJECT_ROOT/daemons/boreas/boreas.service" /etc/systemd/system/
cp "$PROJECT_ROOT/daemons/boreas/org.jesternet.Boreas.conf" /usr/share/dbus-1/system.d/
cp "$PROJECT_ROOT/daemons/boreas/50-boreas.rules" /etc/polkit-1/rules.d/
echo "      ✓ Service files installed"

# Install Prometheus daemon
echo "[4/8] Installing Prometheus daemon..."
if [ -f "$PROJECT_ROOT/daemons/prometheus/target/release/prometheus-daemon" ]; then
    cp "$PROJECT_ROOT/daemons/prometheus/target/release/prometheus-daemon" /usr/local/bin/
    chmod 755 /usr/local/bin/prometheus-daemon
    echo "      ✓ Binary installed"
else
    echo "ERROR: Prometheus binary not found. Did you run 'cargo build --release'?"
    exit 1
fi

# Install Prometheus service files
cp "$PROJECT_ROOT/daemons/prometheus/prometheus.service" /etc/systemd/system/
cp "$PROJECT_ROOT/daemons/prometheus/org.jesternet.Prometheus.conf" /usr/share/dbus-1/system.d/
cp "$PROJECT_ROOT/daemons/prometheus/50-prometheus.rules" /etc/polkit-1/rules.d/
echo "      ✓ Service files installed"

# Load ec_sys module
echo "[5/8] Configuring ec_sys kernel module..."
echo "ec_sys" > /etc/modules-load.d/ec_sys.conf
echo "options ec_sys write_support=1" > /etc/modprobe.d/ec_sys.conf
modprobe ec_sys write_support=1 2>/dev/null || echo "      Note: ec_sys module will load on next boot"
echo "      ✓ Module configured"

# Load msr module
echo "[6/8] Configuring msr kernel module..."
echo "msr" > /etc/modules-load.d/msr.conf
modprobe msr 2>/dev/null || echo "      Note: msr module will load on next boot"
echo "      ✓ Module configured"

# Reload systemd and dbus
echo "[7/8] Reloading system services..."
systemctl daemon-reload
systemctl reload dbus 2>/dev/null || systemctl restart dbus
echo "      ✓ Services reloaded"

# Final instructions
echo "[8/8] Installation complete!"
echo ""
echo "========================================="
echo " Next Steps"
echo "========================================="
echo ""
echo "1. Enable and start the daemons:"
echo "   sudo systemctl enable --now boreas.service"
echo "   sudo systemctl enable --now prometheus.service"
echo ""
echo "2. Verify daemons are running:"
echo "   sudo systemctl status boreas.service"
echo "   sudo systemctl status prometheus.service"
echo ""
echo "3. Install the GNOME extension:"
echo "   ./scripts/install-extension.sh"
echo ""
echo "4. Restart GNOME Shell:"
echo "   Press Alt+F2, type 'r', press Enter"
echo ""
echo "5. Look for the weather icon in your top bar"
echo ""
echo "========================================="
echo ""
echo "⚠️  IMPORTANT REMINDERS:"
echo "   - Start with conservative profiles (Balanced)"
echo "   - Monitor temperatures with 'sensors'"
echo "   - Check logs if issues occur: journalctl -xeu boreas.service"
echo ""
echo "🔗 Report issues: https://github.com/quantumencoding/acer-nitro-57-extensions/issues"
echo ""
