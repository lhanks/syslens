# IDE Setup Guide

Recommended IDE extensions and configuration for Syslens development.

## Visual Studio Code (Recommended)

### Required Extensions

#### Rust Development
| Extension | ID | Purpose |
|-----------|----|---------|
| **rust-analyzer** | `rust-lang.rust-analyzer` | Rust language server (IntelliSense, diagnostics, code actions) |
| **CodeLLDB** | `vadimcn.vscode-lldb` | Rust debugging support |
| **crates** | `serayuzgur.crates` | Cargo.toml dependency version hints |

#### Angular Development
| Extension | ID | Purpose |
|-----------|----|---------|
| **Angular Language Service** | `angular.ng-template` | Angular template IntelliSense, diagnostics |
| **ESLint** | `dbaeumer.vscode-eslint` | JavaScript/TypeScript linting |

#### Styling
| Extension | ID | Purpose |
|-----------|----|---------|
| **Tailwind CSS IntelliSense** | `bradlc.vscode-tailwindcss` | Tailwind class autocomplete, hover preview |

### Optional Extensions

| Extension | ID | Purpose |
|-----------|----|---------|
| **Tauri** | `tauri-apps.tauri-vscode` | Tauri project support |
| **Even Better TOML** | `tamasfe.even-better-toml` | TOML file support for Cargo.toml |
| **Error Lens** | `usernamehw.errorlens` | Inline error/warning display |
| **GitLens** | `eamodio.gitlens` | Enhanced Git integration |
| **Prettier** | `esbenp.prettier-vscode` | Code formatting (HTML, CSS, JSON) |

### Quick Install

Copy and run in VS Code terminal (Ctrl+`):

```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension serayuzgur.crates
code --install-extension angular.ng-template
code --install-extension dbaeumer.vscode-eslint
code --install-extension bradlc.vscode-tailwindcss
```

Optional extensions:
```bash
code --install-extension tauri-apps.tauri-vscode
code --install-extension tamasfe.even-better-toml
code --install-extension usernamehw.errorlens
```

---

## Workspace Settings

Create or update `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": null,

  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },

  "[typescript]": {
    "editor.defaultFormatter": "dbaeumer.vscode-eslint",
    "editor.formatOnSave": true
  },

  "[html]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },

  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",

  "eslint.workingDirectories": ["projects/ui"],

  "tailwindCSS.includeLanguages": {
    "html": "html",
    "typescript": "typescript"
  },
  "tailwindCSS.experimental.classRegex": [
    ["class\\s*=\\s*[\"']([^\"']*)[\"']", "([^\"'\\s]*)"]
  ],

  "files.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/.angular": true
  },

  "search.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/dist": true
  }
}
```

---

## Launch Configurations

Create or update `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Tauri Dev",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["build", "--manifest-path", "projects/backend/Cargo.toml"]
      },
      "preLaunchTask": "npm: start - projects/ui"
    },
    {
      "name": "Debug Rust Tests",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": ["test", "--no-run", "--manifest-path", "projects/backend/Cargo.toml"]
      },
      "cwd": "${workspaceFolder}/projects/backend"
    }
  ]
}
```

---

## Tasks Configuration

Create or update `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Tauri Dev",
      "type": "shell",
      "command": "cargo tauri dev",
      "options": { "cwd": "${workspaceFolder}/projects/backend" },
      "group": { "kind": "build", "isDefault": true },
      "problemMatcher": ["$rustc", "$tsc"]
    },
    {
      "label": "Build UI",
      "type": "npm",
      "script": "build",
      "path": "projects/ui",
      "problemMatcher": ["$tsc"]
    },
    {
      "label": "Test Rust",
      "type": "shell",
      "command": "cargo test",
      "options": { "cwd": "${workspaceFolder}/projects/backend" },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "Test Angular",
      "type": "npm",
      "script": "test",
      "path": "projects/ui",
      "problemMatcher": []
    },
    {
      "label": "Lint All",
      "type": "shell",
      "command": "npm run lint && cargo clippy -- -D warnings",
      "options": { "cwd": "${workspaceFolder}/projects/ui" },
      "problemMatcher": ["$eslint-stylish", "$rustc"]
    }
  ]
}
```

---

## Verifying Setup

After installing extensions and restarting VS Code:

### Rust (rust-analyzer)
1. Open `projects/backend/src/main.rs`
2. Hover over a type - should show type info
3. Errors should appear inline with red underlines
4. `Ctrl+Space` should show completions

### Angular (Angular Language Service)
1. Open `projects/ui/src/app/app.component.ts`
2. In the template, type `<app-` - should show component suggestions
3. Property binding errors should show inline

### Tailwind CSS
1. Open any component HTML
2. Type `class="bg-"` - should show color suggestions
3. Hover over Tailwind classes to see CSS preview

### ESLint
1. Open any `.ts` file in `projects/ui`
2. Lint errors should appear with yellow/red underlines
3. Quick fixes available via `Ctrl+.`

---

## Troubleshooting

### rust-analyzer not working
```bash
# Restart rust-analyzer
Ctrl+Shift+P → "rust-analyzer: Restart server"

# Check rust-analyzer output
View → Output → Select "rust-analyzer" from dropdown
```

### Angular Language Service not working
```bash
# Restart Angular LS
Ctrl+Shift+P → "Angular: Restart Angular Language Server"

# Ensure node_modules installed
cd projects/ui && npm install
```

### Tailwind IntelliSense not working
- Ensure `tailwind.config.js` exists in `projects/ui`
- Check that the extension is enabled for the workspace
- Restart VS Code

---

## Alternative IDEs

### JetBrains (WebStorm + RustRover)
- **RustRover** or **IntelliJ with Rust plugin**: Full Rust support
- **WebStorm**: Angular + TypeScript support
- Both include Tailwind CSS support built-in

### Neovim
- **rust-tools.nvim**: Rust LSP integration
- **nvim-lspconfig**: Angular Language Server setup
- **tailwindcss-colorizer-cmp.nvim**: Tailwind support
