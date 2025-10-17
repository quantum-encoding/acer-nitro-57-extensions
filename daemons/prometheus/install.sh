#!/bin/bash
set -e

echo "Installing Prometheus Performance Control Daemon..."

# Copy binary
sudo cp target/release/prometheus-daemon /usr/local/bin/
sudo chmod 755 /usr/local/bin/prometheus-daemon

# Copy systemd service
sudo cp prometheus.service /etc/systemd/system/
sudo chmod 644 /etc/systemd/system/prometheus.service

# Copy polkit rule
sudo cp 50-prometheus.rules /etc/polkit-1/rules.d/
sudo chmod 644 /etc/polkit-1/rules.d/50-prometheus.rules

# Copy D-Bus policy
sudo cp org.jesternet.Prometheus.conf /usr/share/dbus-1/system.d/
sudo chmod 644 /usr/share/dbus-1/system.d/org.jesternet.Prometheus.conf

# Reload systemd and dbus
sudo systemctl daemon-reload
sudo systemctl reload dbus

echo "Installation complete!"
echo ""
echo "To enable and start the service:"
echo "  sudo systemctl enable --now prometheus.service"
echo ""
echo "To check status:"
echo "  sudo systemctl status prometheus.service"
echo ""
echo "To test the D-Bus interface:"
echo "  busctl --system call org.jesternet.Prometheus /org/jesternet/Prometheus org.jesternet.Prometheus SetPerformanceProfile s warspeed"
