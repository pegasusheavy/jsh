# Changelog

All notable changes to the jsh documentation site will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Syntax highlighting for code blocks
- Search functionality
- Mobile navigation menu
- Interactive REPL playground

---

## [1.0.0] - 2025-12-21

### Added

#### Documentation Pages
- **Home Page** - Hero section with feature overview, multi-shell compatibility grid, enhanced syntax examples, and quick install guide
- **Getting Started** - Installation instructions, configuration guide, and next steps navigation
- **Bash/ZSH Compatibility** - Variables, control flow (if/for/while/case), functions, pipes & redirections, command substitution, arithmetic, and logical operators
- **Fish Compatibility** - `set` command, `string` manipulation, `math` operations, `contains`, `status`, `functions`, and `abbr` documentation
- **Ash/POSIX Compatibility** - Shell options table, positional parameters, variable declarations, source command, and test command reference
- **jsh Enhanced Syntax** - `match` pattern matching, `let`/`const` bindings, `loop` statement, `try`/`catch`/`finally` error handling, and `fn` function shorthand
- **Builtins Reference** - Comprehensive reference for core commands, variable commands, I/O commands, flow control, Fish-compatible builtins, theme system, and directory stack

#### Layout & Navigation
- Sticky header with responsive navigation
- Logo and branding (🦀 jsh)
- External link to GitHub repository
- Footer with copyright, GitHub, Issues, and Patreon links
- Lazy-loaded routes for optimal performance

#### Theming
- Integration with `@pegasusheavy/tailswatch` Oxide theme
- Rust-inspired color scheme with orange (#f74c00) accents
- Dark mode design with proper contrast
- Custom CSS variables for easy customization
- Code block styling with monospace fonts
- Custom scrollbar styling

#### Infrastructure
- Angular 21 standalone components
- Tailwind CSS 4 integration
- `@pegasusheavy/ngx-tailwindcss` component library
- pnpm package manager configuration
- TypeScript strict mode

#### Documentation
- README.md with setup instructions
- TODO.md with roadmap
- CHANGELOG.md (this file)

### Technical Details
- Built with Angular 21.0.0
- Uses Tailwind CSS 4.1.12
- Standalone components architecture
- Lazy-loaded page modules
- Production build size: ~228 KB initial, ~34 KB lazy chunks

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 1.0.0 | 2025-12-21 | Initial release with full documentation |

---

## Migration Guide

### From Pre-release to 1.0.0

This is the initial release. No migration required.

---

## Links

- [jsh Repository](https://github.com/pegasusheavy/jsh)
- [Report Issues](https://github.com/pegasusheavy/jsh/issues)
- [Pegasus Heavy Industries](https://github.com/pegasusheavy)

---

*Maintained by Pegasus Heavy Industries LLC*

