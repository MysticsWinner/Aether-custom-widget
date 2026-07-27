# Marketplace & Package Manager CLI Reference

The **Phase 12 Package Manager** (`crates/package_manager`) provides an npm-like CLI interface for discovering, installing, updating, and managing desktop widgets and plugins.

---

## 🛠️ Package Manager Commands

### 1. `install <package-name>`
Installs a cryptographically verified widget package from the marketplace registry into `~/.custom_widgets/packages/`.

```bash
# Install Weather Widget
custom-widget-pkg install weather-widget

# Install Spotify Media Controller
custom-widget-pkg install spotify-widget

# Install Taskbar Enhancement Suite
custom-widget-pkg install taskbar-plus
```

### 2. `uninstall <package-name>`
Removes an installed widget package and purges its local cache.

```bash
custom-widget-pkg uninstall weather-widget
```

### 3. `list`
Lists all currently installed widget packages alongside their versions and health status.

```bash
custom-widget-pkg list
```

### 4. `search <query>`
Searches the marketplace registry for matching packages.

```bash
custom-widget-pkg search weather
```

### 5. `update [package-name]`
Checks for updated `.cwp` releases signed with valid Ed25519 signatures.

```bash
custom-widget-pkg update
```

---

## 📦 `.cwp` Package Structure

Marketplace widget packages are bundled as `.cwp` (Custom Widget Package) zip archives containing:

```
weather-widget.cwp
├── widget.toml           # Package Manifest Schema
├── signature.ed25519     # Cryptographic Digital Signature
├── index.lua             # Entrypoint Script (or index.wasm / binary)
└── assets/
    ├── icon.png
    └── styles.json
```

---

## 📄 Manifest Schema (`widget.toml`)

```toml
[metadata]
id = "weather-widget"
name = "Live Weather Overlay"
version = "1.2.0"
author = "Community"
description = "Real-time hardware-accelerated desktop weather widget"
main_entrypoint = "index.lua"

[dependencies]
"theme.cyberpunk" = ">=1.0.0"

[capabilities]
requested = [
  "capability.telemetry.read",
  "capability.network.http"
]
```
