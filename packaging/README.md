# jsh Packaging

This directory contains scripts and configuration for building distribution packages.

## Quick Start

### Build All Packages

```bash
./packaging/build-all.sh
```

### Build Debian Package Only

```bash
./packaging/build-deb.sh
```

### Build RPM Package Only

```bash
./packaging/build-rpm.sh
```

### Build Arch Linux Package

```bash
cd packaging
makepkg -si
```

## Prerequisites

### Debian/Ubuntu

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-deb
cargo install cargo-deb
```

### Fedora/RHEL/CentOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-generate-rpm
cargo install cargo-generate-rpm
```

### Arch Linux

```bash
# Install build tools (rust is pulled as makedepends)
sudo pacman -S base-devel
```

## Package Locations

After building:

- **Debian packages**: `target/debian/jsh_*.deb`
- **RPM packages**: `target/generate-rpm/jsh-*.rpm`
- **Arch packages**: `packaging/jsh-*.pkg.tar.zst`

## Installation

### Debian/Ubuntu

```bash
# Install
sudo dpkg -i target/debian/jsh_*.deb

# Or with dependency resolution
sudo apt install ./target/debian/jsh_*.deb

# Verify installation
jsh --version
which jsh
```

### Fedora/RHEL

```bash
# Install
sudo dnf install target/generate-rpm/jsh-*.rpm

# Or on older systems
sudo rpm -i target/generate-rpm/jsh-*.rpm

# Verify installation
jsh --version
which jsh
```

### Arch Linux

```bash
# Build and install from PKGBUILD
cd packaging
makepkg -si

# Or for development version (from git)
cp PKGBUILD-git PKGBUILD
makepkg -si

# Verify installation
jsh --version
which jsh
```

## Setting jsh as Default Shell

After installation, jsh is automatically added to `/etc/shells`. To set it as your default shell:

```bash
# Change your default shell
chsh -s /usr/bin/jsh

# Verify
cat /etc/passwd | grep $USER
```

## Package Contents

Both packages install:

| Path | Description |
|------|-------------|
| `/usr/bin/jsh` | The jsh binary |
| `/usr/share/doc/jsh/README` | Documentation |
| `/usr/share/doc/jsh/CHANGELOG` | Changelog |

RPM additionally includes:

| Path | Description |
|------|-------------|
| `/usr/share/licenses/jsh/LICENSE-MIT` | MIT License |
| `/usr/share/licenses/jsh/LICENSE-APACHE` | Apache 2.0 License |

## Manual Building (Alternative Methods)

### Traditional Debian Build

```bash
cd /path/to/jsh
dpkg-buildpackage -us -uc -b
```

### Traditional RPM Build

```bash
# Create source tarball
tar -czvf jsh-0.1.0.tar.gz --transform 's,^,jsh-0.1.0/,' \
    src/ Cargo.toml Cargo.lock README.md CHANGELOG.md LICENSE-*

# Build with rpmbuild
rpmbuild -bb rpm/jsh.spec --define "_sourcedir $(pwd)"
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build Packages

on:
  release:
    types: [created]

jobs:
  build-deb:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-deb
      - run: cargo deb
      - uses: actions/upload-artifact@v4
        with:
          name: debian-package
          path: target/debian/*.deb

  build-rpm:
    runs-on: ubuntu-latest
    container: fedora:latest
    steps:
      - uses: actions/checkout@v4
      - run: dnf install -y rust cargo
      - run: cargo install cargo-generate-rpm
      - run: cargo build --release
      - run: cargo generate-rpm
      - uses: actions/upload-artifact@v4
        with:
          name: rpm-package
          path: target/generate-rpm/*.rpm
```

## Troubleshooting

### cargo-deb fails with "not found"

Ensure Cargo's bin directory is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Missing dependencies during dpkg install

Use apt instead of dpkg:

```bash
sudo apt install ./target/debian/jsh_*.deb
```

### RPM scriptlet failures

Check the scriptlet logs:

```bash
rpm -qp --scripts target/generate-rpm/jsh-*.rpm
```

## Version Bumping

When releasing a new version:

1. Update `version` in `Cargo.toml`
2. Update `debian/changelog` with `dch -i` or manually
3. Update `rpm/jsh.spec` version and changelog
4. Rebuild packages

## License

jsh is dual-licensed under MIT and Apache 2.0. See LICENSE-MIT and LICENSE-APACHE for details.

