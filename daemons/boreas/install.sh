#!/bin/bash
set -e

echo "Installing Boreas Fan Control Daemon..."

# Copy binary
sudo cp target/release/boreas-daemon /usr/local/bin/
sudo chmod 755 /usr/local/bin/boreas-daemon

# Copy systemd service
sudo cp boreas.service /etc/systemd/system/
sudo chmod 644 /etc/systemd/system/boreas.service

# Copy polkit rule
sudo cp 50-boreas.rules /etc/polkit-1/rules.d/
sudo chmod 644 /etc/polkit-1/rules.d/50-boreas.rules

# Copy D-Bus policy
sudo cp org.jesternet.Boreas.conf /usr/share/dbus-1/system.d/
sudo chmod 644 /usr/share/dbus-1/system.d/org.jesternet.Boreas.conf

# Ensure ec_sys module loads on boot with write support
echo "ec_sys write_support=1" | sudo tee /etc/modules-load.d/ec_sys.conf
echo "options ec_sys write_support=1" | sudo tee /etc/modprobe.d/ec_sys.conf

# Reload systemd
sudo systemctl daemon-reload

echo "Installation complete!"
echo ""
echo "To enable and start the service:"
echo "  sudo systemctl enable --now boreas.service"
echo ""
echo "To check status:"
echo "  sudo systemctl status boreas.service"
echo ""
echo "To test the D-Bus interface:"
echo "  busctl --system call org.jesternet.Boreas /org/jesternet/Boreas org.jesternet.Boreas SetFanProfile s maxpower"
