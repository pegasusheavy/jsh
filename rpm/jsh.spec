Name:           jsh
Version:        0.1.0
Release:        1%{?dist}
Summary:        ZSH/Bash-compatible shell with enhanced scripting

License:        MIT OR Apache-2.0
URL:            https://github.com/pegasusheavy/jsh
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.75
BuildRequires:  rust >= 1.75
BuildRequires:  gcc

%description
jsh is a modern shell that combines full compatibility with Bash/ZSH
scripts while introducing cleaner, more intuitive syntax for flow
control and scripting.

Features include:
- Full Bash/ZSH compatibility
- Fish shell compatibility
- POSIX/Ash compatibility
- Modern syntax: match expressions, try/catch, named function parameters
- Oh-My-Zsh compatible theme system
- Built-in commands for string manipulation, math, and more

%prep
%autosetup

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
install -D -m 755 target/release/jsh %{buildroot}%{_bindir}/jsh

# Install man page if it exists
if [ -f man/jsh.1 ]; then
    install -D -m 644 man/jsh.1 %{buildroot}%{_mandir}/man1/jsh.1
fi

# Create config directory
install -d %{buildroot}%{_sysconfdir}/jsh

%check
cargo test --release

%post
# Add to /etc/shells
if [ -f /etc/shells ]; then
    if ! grep -q "^%{_bindir}/jsh$" /etc/shells; then
        echo "%{_bindir}/jsh" >> /etc/shells
    fi
fi

%preun
# Remove from /etc/shells on uninstall (not upgrade)
if [ $1 -eq 0 ]; then
    if [ -f /etc/shells ]; then
        sed -i '\|^%{_bindir}/jsh$|d' /etc/shells
    fi
fi

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md CHANGELOG.md
%{_bindir}/jsh
%dir %{_sysconfdir}/jsh

%changelog
* Sat Dec 21 2024 Pegasus Heavy Industries LLC <support@pegasusheavy.com> - 0.1.0-1
- Initial release
- Full Bash/ZSH compatibility
- Fish shell compatibility
- POSIX/Ash compatibility
- Modern syntax extensions (match, try/catch, named parameters)
- Oh-My-Zsh compatible theme system

