# NativePHP Tauri Driver: High-Level Implementation Plan

This document outlines the complete architectural roadmap for integrating Tauri as a first-class driver for NativePHP, acting as an alternative to the existing Electron driver.

## 1. Overview
NativePHP currently relies on Electron (Node.js + Chromium) to provide the desktop window and system APIs. To support Tauri (Rust + system webviews), we must replace both the PHP scaffolding commands (which currently invoke `npm` and `electron-builder`) and the Javascript bridge (which handles NativePHP's IPC) with their Tauri and Rust equivalents.

## 2. Phase 1: PHP Driver Scaffolding
We need to create the PHP boilerplate that allows developers to run `php artisan native:install --driver=tauri` and use Tauri during development and builds.

### 2.1 Driver Structure
- **`src/Drivers/Tauri/TauriServiceProvider.php`**: Registers the Tauri commands and bindings.
- **`src/Drivers/Tauri/Commands/`**:
  - `InstallCommand`: Copies the Tauri boilerplate from `resources/tauri` into the user's application, patches `composer.json` and `package.json`, and installs Rust/Node dependencies.
  - `RunCommand`: Invokes `cargo tauri dev` or `npm run tauri dev` to start the development environment and spawns the PHP artisan server.
  - `BuildCommand`: Invokes `cargo tauri build` to compile the final binary.
  - `PublishCommand`: Handles signing and distributing the binaries (potentially leveraging Tauri's GitHub Actions or built-in updater).

### 2.2 Tauri Boilerplate (`resources/tauri`)
We will create a standard Tauri project template inside the package (`resources/tauri`). This template will include:
- A generic `src-tauri/Cargo.toml`.
- Basic `package.json` with Tauri CLI dependencies.
- A dummy frontend (e.g., standard Vite + Vanilla JS) that connects to the PHP server.

## 3. Phase 2: The Rust Bridge (The "Plugin")
Currently, NativePHP uses `resources/electron/electron-plugin` (over 10,000 lines of TypeScript) to act as a bridge between PHP and Electron. We must write a **Rust equivalent** for Tauri.

### 3.1 PHP Server Spawning
- The Rust application (`src-tauri/src/main.rs`) must find the bundled PHP binary (or the system binary in dev mode) and spawn `php artisan serve` as a child process when the Tauri app boots.
- It must manage the lifecycle of this process (e.g., kill PHP when Tauri closes).

### 3.2 IPC & API Server
- The Rust bridge must start a local REST or WebSocket server on an ephemeral port.
- When NativePHP in Laravel calls `Window::open()`, PHP sends an HTTP/WS request to this local Rust server.
- Rust receives this request and invokes the corresponding Tauri API (`tauri::WindowBuilder`).

### 3.3 Bridging NativePHP Features to Tauri
We need to map NativePHP Facades to Tauri APIs in Rust:
- **Window Management**: `tauri::Window` (Open, close, resize, focus).
- **Menu Bar**: `tauri::Menu` (App menus, context menus).
- **System Tray**: `tauri::SystemTray`.
- **Notifications**: `tauri::api::notification`.
- **Clipboard**: `tauri::api::clipboard`.
- **Global Shortcuts**: `tauri::api::global_shortcut`.
- **Dialogs**: `tauri::api::dialog`.

## 4. Phase 3: Integration and Distribution
- **Updater**: Integrate Tauri's built-in updater to work with NativePHP's update facade.
- **PHP Binaries**: Ensure that the Tauri build process bundles the correct static PHP binaries (Mac, Windows, Linux) into the final `.app` or `.exe`, similar to how `electron-builder` does it.
- **Testing**: End-to-end tests ensuring Laravel can communicate with the Rust bridge.
