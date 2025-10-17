# Sovereign Control for Acer Nitro AN515-57

**Complete thermal and performance control for the Acer Nitro AN515-57 gaming laptop on Linux.**

This project provides two privileged system daemons and a GNOME Shell extension that grant you direct, low-level control over:
- **Fan speeds** (CPU and GPU fans)
- **CPU performance** (frequency governors, turbo boost, energy preferences)

⚠️ **WARNING**: This software directly manipulates hardware registers. It is designed **ONLY** for the **Acer Nitro AN515-57**. Running it on unsupported hardware may cause permanent damage.

## Features

### 🌬️ Boreas - Thermal Control Daemon
- Direct EC (Embedded Controller) fan control
- Real-time fan speed adjustment (0-100%)
- Profiles: Silent, Balanced, MAX POWER, Auto
- Hardware safety locks prevent operation on wrong hardware

### 🔥 Prometheus - Performance Control Daemon
- CPU governor switching (powersave/performance)
- Energy Performance Preference (EPP) control
- Intel Turbo Boost management
- Profiles: Silent, Balanced, WarSpeed

### 🎮 Unified GNOME Shell Extension
- Single-click thermal and performance profiles
- "TOTAL WAR" mode: MAX POWER fans + WarSpeed CPU simultaneously
- System tray integration
- D-Bus controlled - no root access needed from UI

## Architecture

This is a **bifurcated, privilege-separated** design:

```
┌─────────────────────────────────────┐
│   GNOME Shell Extension (User)     │
│   - UI/Menu                         │
│   - No privileges required          │
└──────────┬──────────────────────────┘
           │ D-Bus (unprivileged)
           ▼
┌─────────────────────────────────────┐
│   Boreas Daemon (Root)              │
│   - EC register control             │
│   - Fan speed manipulation          │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│   Prometheus Daemon (Root)          │
│   - CPU governor control            │
│   - EPP/Turbo management            │
└─────────────────────────────────────┘
```

**Security**: User-level extension → D-Bus → Root daemons → Hardware
**Safety**: Hardware verification on daemon startup, input validation, fail-safe defaults

## Requirements

- **Hardware**: Acer Nitro AN515-57 ONLY
- **OS**: Arch Linux (or compatible)
- **Kernel**: Linux 5.0+ with `ec_sys` module support
- **Desktop**: GNOME Shell 45+
- **Dependencies**:
  - `systemd`
  - `dbus`
  - `polkit`
  - Rust toolchain (for building from source)

## Installation

### Quick Install (Pre-compiled - Arch Linux)

```bash
# Clone the repository
git clone https://github.com/quantumencoding/acer-nitro-57-extensions.git
cd acer-nitro-57-extensions

# Run the installer
chmod +x scripts/install.sh
sudo ./scripts/install.sh

# Enable and start the daemons
sudo systemctl enable --now boreas.service
sudo systemctl enable --now prometheus.service

# Install the GNOME extension
./scripts/install-extension.sh

# Restart GNOME Shell: Alt+F2, type 'r', press Enter
```

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build Boreas daemon
cd daemons/boreas
cargo build --release

# Build Prometheus daemon
cd ../prometheus
cargo build --release

# Install (run from repo root)
sudo ./scripts/install.sh
```

## Usage

### Via GNOME Extension

Click the weather icon in your top bar:

**⚡ THERMAL CONTROL**
- Silent Fans - Quiet operation
- Balanced Fans - Default
- MAX POWER Fans - 100% cooling
- Auto Fans - BIOS control

**🔥 PERFORMANCE CONTROL**
- Silent CPU - Power saving mode
- Balanced CPU - Standard performance
- WARSPEED CPU - Maximum performance

**⚔️ COMBINED SOVEREIGNTY**
- TOTAL WAR - Fans + CPU at maximum

### Via Command Line

```bash
# Set fan profile
busctl --system call org.jesternet.Boreas /org/jesternet/Boreas \
  org.jesternet.Boreas SetFanProfile s "maxpower"

# Set CPU profile
busctl --system call org.jesternet.Prometheus /org/jesternet/Prometheus \
  org.jesternet.Prometheus SetPerformanceProfile s "warspeed"

# Check daemon status
sudo systemctl status boreas.service
sudo systemctl status prometheus.service
```

## How It Works

### Boreas (Fan Control)

The Acer Nitro AN515-57 uses an embedded controller (EC) to manage fans. Standard tools like `lm-sensors` cannot control them. Boreas:

1. Loads the `ec_sys` kernel module with write support
2. Verifies hardware identity via DMI
3. Writes directly to EC registers via `/sys/kernel/debug/ec/ec0/io`
4. Controls CPU fan (registers 0x37) and GPU fan (registers 0x3A)

**EC Register Map (AN515-57)**:
- Register 3: Manual control enable (0x11)
- Register 19: CPU fan speed read
- Register 21: GPU fan speed read
- Register 34: CPU fan mode (0x0C = manual)
- Register 33: GPU fan mode (0x30 = manual)
- Register 55: CPU fan speed write (0-100)
- Register 58: GPU fan speed write (0-100)

### Prometheus (CPU Control)

Prometheus manipulates Intel P-State driver settings:

1. Verifies hardware compatibility
2. Writes to `/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor`
3. Adjusts `/sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference`
4. Controls turbo via `/sys/devices/system/cpu/intel_pstate/no_turbo`

## Safety Features

### Hardware Lock
Both daemons check `/sys/class/dmi/id/product_name` on startup. If not `Nitro AN515-57`, they **refuse to run** and exit with an error.

### Input Validation
- Fan speeds clamped to 0-100%
- Governor/EPP values validated against kernel options
- Invalid D-Bus commands rejected with clear errors

### Fail-Safe Defaults
- Fans default to "Auto" (BIOS control) on daemon stop
- CPU settings revert to `powersave` on daemon crash
- No permanent changes to system files

## Troubleshooting

### Daemon won't start
```bash
# Check hardware detection
sudo journalctl -xeu boreas.service | grep "Hardware"

# Ensure ec_sys module is loaded
lsmod | grep ec_sys
sudo modprobe ec_sys write_support=1
```

### Extension doesn't appear
```bash
# Check extension status
gnome-extensions list
gnome-extensions enable boreas@jesternet.org

# View errors
journalctl -f -o cat /usr/bin/gnome-shell
```

### Fans not responding
```bash
# Check EC access
ls -la /sys/kernel/debug/ec/ec0/io

# Verify daemon is running
sudo systemctl status boreas.service

# Test manual D-Bus call
busctl --system call org.jesternet.Boreas /org/jesternet/Boreas \
  org.jesternet.Boreas SetFanProfile s "balanced"
```

## Uninstallation

```bash
# Stop and disable daemons
sudo systemctl disable --now boreas.service prometheus.service

# Remove files
sudo rm /usr/local/bin/boreas-daemon /usr/local/bin/prometheus-daemon
sudo rm /etc/systemd/system/boreas.service /etc/systemd/system/prometheus.service
sudo rm /usr/share/dbus-1/system.d/org.jesternet.{Boreas,Prometheus}.conf
sudo rm /etc/polkit-1/rules.d/50-{boreas,prometheus}.rules

# Remove extension
rm -rf ~/.local/share/gnome-shell/extensions/boreas@jesternet.org

# Cleanup
sudo systemctl daemon-reload
sudo systemctl reload dbus
```

## Development

### Adding Support for Other Models

1. Extract EC register map using `nbfc-linux` or `ec_probe`
2. Update `SUPPORTED_MODELS` in both daemons
3. Add model-specific register addresses
4. Test thoroughly before release
5. Submit PR with hardware compatibility data

### Project Structure
```
├── daemons/
│   ├── boreas/          # Fan control daemon (Rust)
│   └── prometheus/      # CPU control daemon (Rust)
├── extension/           # GNOME Shell extension (JavaScript)
├── docs/                # Additional documentation
└── scripts/             # Installation scripts
```

## Contributing

Contributions welcome! Please:
1. Test on actual AN515-57 hardware
2. Follow the existing code style
3. Add hardware compatibility data to docs
4. Submit PRs with clear descriptions

## License

MIT License - See LICENSE file

## Disclaimer

⚠️ **USE AT YOUR OWN RISK**

This software directly manipulates hardware. While safety measures are in place:
- We are NOT responsible for hardware damage
- We are NOT responsible for data loss
- We are NOT responsible for system instability
- Warranty may be voided

Always:
- Keep backups
- Monitor temperatures
- Start with conservative profiles
- Understand what you're doing

## Credits

- Developed by: Richard Tune (info@quantumencoding.io)
- Inspired by: `nbfc-linux`, `intel-undervolt`
- EC register data: Community reverse engineering

## Links

- **Issues**: https://github.com/quantumencoding/acer-nitro-57-extensions/issues
- **Discussions**: https://github.com/quantumencoding/acer-nitro-57-extensions/discussions
- **Acer Nitro Community**: [Link to community forum]

---

*"Sovereignty over silicon. Control over the machine."*
