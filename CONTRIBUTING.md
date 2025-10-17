# Contributing to Sovereign Control

Thank you for your interest in contributing! This project controls hardware directly, so contributions require careful review.

## How to Contribute

### Reporting Issues

When reporting bugs, please include:
- **Hardware**: Exact laptop model (e.g., "Acer Nitro AN515-57")
- **OS**: Distribution and kernel version (`uname -a`)
- **Logs**: Output from `sudo journalctl -xeu boreas.service`
- **Behavior**: What you expected vs. what happened

### Adding Hardware Support

To add support for a new laptop model:

1. **Extract EC Register Map**
   - Use `nbfc-linux` config or `ec_probe` tool
   - Document all register addresses
   - Test extensively before submitting

2. **Update Code**
   - Add model to `SUPPORTED_MODELS` array
   - Add model-specific register addresses if different
   - Update README.md with compatibility info

3. **Submit PR**
   - Include hardware verification output
   - Include test results (temperatures, fan speeds)
   - Explain any model-specific quirks

### Code Style

**Rust**:
- Follow `rustfmt` defaults
- Use `clippy` and address warnings
- Document public functions
- Add error context with `.context()`

**JavaScript**:
- Follow GNOME Shell extension conventions
- Use clear variable names
- Comment complex logic

### Testing Requirements

Before submitting:
1. **Compile without warnings**
   ```bash
   cargo clippy --all-targets
   ```

2. **Test on actual hardware**
   - Verify all profiles work
   - Check temperatures under load
   - Ensure safe failure modes

3. **Verify safety locks**
   - Daemon refuses to start on wrong hardware
   - Invalid inputs are rejected
   - Logs are clear and helpful

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-model-support`)
3. Make your changes
4. Test thoroughly
5. Update documentation
6. Submit PR with detailed description

### What We DON'T Accept

- ❌ Bypassing hardware safety checks
- ❌ Hardcoded credentials or personal data
- ❌ Untested code
- ❌ Support for virtual machines (EC access doesn't work in VMs)
- ❌ Features that could cause hardware damage

## Development Setup

```bash
# Clone your fork
git clone https://github.com/quantumencoding/acer-nitro-57-extensions.git
cd acer-nitro-57-extensions

# Build daemons
cd daemons/boreas
cargo build --release
cd ../prometheus
cargo build --release

# Test locally (without installing)
sudo target/release/boreas-daemon
```

## Questions?

Open a [Discussion](https://github.com/quantumencoding/acer-nitro-57-extensions/discussions) for:
- Feature requests
- Design questions
- Hardware compatibility questions

## Code of Conduct

- Be respectful and professional
- Focus on technical merit
- Help others learn
- Assume good faith

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
