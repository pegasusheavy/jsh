# jsh Documentation Site

The official documentation website for jsh - a modern shell combining Bash, ZSH, Fish, and Ash compatibility with enhanced scripting features.

## 🚀 Quick Start

```bash
# Install dependencies
pnpm install

# Start development server
pnpm start

# Build for production
pnpm build
```

The development server runs at `http://localhost:4200` by default.

## 📁 Project Structure

```
docs/
├── src/
│   ├── app/
│   │   ├── pages/
│   │   │   ├── home/              # Landing page
│   │   │   ├── getting-started/   # Installation & setup guide
│   │   │   ├── bash-zsh/          # Bash/ZSH compatibility docs
│   │   │   ├── fish/              # Fish shell compatibility docs
│   │   │   ├── ash-posix/         # Ash/POSIX compatibility docs
│   │   │   ├── jsh-syntax/        # Enhanced jsh syntax docs
│   │   │   └── builtins/          # Built-in commands reference
│   │   ├── app.ts                 # Root component with layout
│   │   ├── app.config.ts          # Application configuration
│   │   └── app.routes.ts          # Route definitions
│   ├── styles.scss                # Global styles & theme
│   ├── index.html                 # HTML entry point
│   └── main.ts                    # Application bootstrap
├── angular.json                   # Angular CLI configuration
├── package.json                   # Dependencies & scripts
└── tsconfig.json                  # TypeScript configuration
```

## 🎨 Theming

The documentation site uses the **Oxide** theme from `tailswatch`, a Rust-inspired dark theme with orange accents.

### Color Palette

| Variable | Color | Usage |
|----------|-------|-------|
| `--rust-orange` | `#f74c00` | Primary accent |
| `--rust-dark` | `#1a1a1a` | Surface background |
| `--rust-darker` | `#0d0d0d` | Page background |
| `--color-text` | `#e0e0e0` | Primary text |
| `--color-text-muted` | `#a0a0a0` | Secondary text |

### Customization

Edit `src/styles.scss` to customize the theme variables:

```scss
:root {
  --color-primary: #f74c00;
  --color-background: #0d0d0d;
  // ... other variables
}
```

## 📖 Documentation Pages

| Page | Route | Description |
|------|-------|-------------|
| Home | `/` | Overview, features, quick install |
| Getting Started | `/getting-started` | Installation and configuration |
| Bash/ZSH | `/bash-zsh` | Traditional shell compatibility |
| Fish | `/fish` | Fish shell features (set, string, math) |
| Ash/POSIX | `/ash-posix` | POSIX compliance and shell options |
| jsh Syntax | `/jsh-syntax` | Enhanced syntax (match, loop, try/catch) |
| Builtins | `/builtins` | Complete built-in command reference |

## 🛠️ Development

### Prerequisites

- Node.js 18+
- pnpm 10+

### Commands

```bash
# Development server with hot reload
pnpm start

# Production build
pnpm build

# Run tests
pnpm test

# Lint check
pnpm ng lint
```

### Adding a New Page

1. Create a new directory in `src/app/pages/`
2. Create a component file (e.g., `my-page.ts`)
3. Add a route in `src/app/app.routes.ts`
4. Add navigation link in `src/app/app.ts`

Example component:

```typescript
import { Component } from '@angular/core';

@Component({
  selector: 'app-my-page',
  template: `
    <div class="max-w-4xl mx-auto px-4 py-12">
      <h1 class="text-4xl font-bold mb-4">My Page</h1>
      <!-- Content -->
    </div>
  `
})
export class MyPage {}
```

## 📦 Dependencies

- **Angular 21** - Application framework
- **Tailwind CSS 4** - Utility-first CSS
- **tailswatch** - Theme library (Oxide theme)
- **ngx-tailwindcss** - Angular Tailwind components

## 🚢 Deployment

Build the production bundle:

```bash
pnpm build
```

The output will be in `dist/docs/`. Deploy this directory to any static hosting service:

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages

### GitHub Pages Example

```bash
# Build and deploy to gh-pages branch
pnpm build
npx gh-pages -d dist/docs
```

## 📄 License

This documentation site is part of the jsh project, dual-licensed under MIT and Apache 2.0.

© 2025 Joseph R. Quinn
