# uptime-watcher

## Project Overview

**uptime-watcher** is a desktop application built with [Tauri](https://tauri.app/), designed to monitor the uptime of network services. It sits in the system tray and provides visual feedback (green/red icons) based on the reachability of user-configured IP addresses and ports.

*   **Frontend:** React (TypeScript) via [Vite](https://vitejs.dev/).
*   **Backend:** Rust (Tauri).
*   **Primary Function:** Periodically checks TCP connectivity to a list of services and updates the system tray icon and menu accordingly.

## Key Features

*   **Dynamic Service Management:** Add, Edit, and Remove services (Name, IP, Port) via the dashboard.
*   **Real-time Tray Status:**
    *   Tray Icon turns **Green** (All Healthy) or **Red** (Any Issue).
    *   Tray Menu lists each service with a status emoji (✅/❌).
*   **Configurable Interval:** Set health check check frequency (10s, 1m, 10m, 30m, 1hr).
*   **Persistence:** Services and settings are saved automatically to `settings.json` in the AppData directory and restored on launch.
*   **Modern UI:** "MacOS Liquid Glass" aesthetic with smooth animations and dark mode styling.

## Architecture & Implementation

### Backend (`src-tauri/`)

*   **Entry Point:** `src-tauri/src/main.rs`
    *   **State configuration:** Manages `AppState` which holds the service list and configuration path.
    *   **Background Thread:** Spawns an async loop that checks TCP connectivity based on the user-selected interval.
    *   **Persistence:** Implements `load_state` and `save_state` to sync data with `settings.json`.
    *   **Commands:** Exposes `add_service`, `remove_service`, `update_service`, `set_interval`, `get_interval` to the frontend.
    *   **Tray Logic:** Dynamically rebuilds the system tray menu on every check to reflect current service health.
*   **Configuration:** `src-tauri/tauri.conf.json`
    *   Window setup ("Uptime Watcher").
    *   Permissions and bundle settings.

### Frontend (`src/`)

*   **UI:** `src/App.tsx`
    *   **Dashboard:** Displays the list of monitored services.
    *   **Form:** Allows adding and editing services.
    *   **Settings:** Dropdown to configure the global check interval.
*   **Styling:** `src/App.css`
    *   Implements a custom "Liquid Glass" design system using `backdrop-filter`, gradients, and CSS animations.

## Building and Running

### Prerequisites

*   Node.js & Yarn
*   Rust & Cargo

### Commands

*   **Install Frontend Dependencies:**
    ```bash
    yarn install
    ```
*   **Run in Development Mode:**
    ```bash
    yarn tauri dev
    ```

*   **Build for Production:**
    ```bash
    yarn tauri build
    ```

## Development Notes

*   **Data Storage:** Data is stored in `settings.json` within the OS-specific AppData folder (e.g., `~/Library/Application Support/com.uptime-watcher.app/` on macOS).
*   **Icons:** The app requires `green.png` and `red.png` in the `src-tauri/icons/` directory.
