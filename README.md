# Uptime Watcher

A beautiful, lightweight, and stealthy macOS menu bar application to monitor your network services. Built with [Tauri](https://tauri.app/), [React](https://react.dev/), and Rust.

## Features

*   **System Tray Monitoring**: A discrete icon in your menu bar shows the overall health of your services at a glance.
*   **Accessory Mode**: The app is completely hidden from the Dock and Command+Tab switcher. It lives entirely in your menu bar.
*   **Dynamic Service Management**: Add, edit, and remove unlimited services (TCP/HTTP) directly from the UI.
*   **Real-time Updates**:
    *   **Green/Red**: Default status indicators.
    *   **Adaptive Icons**: Optional "Check/Cross" icons that automatically adapt to your macOS Light/Dark theme.
*   **Configurable Intervals**: Set check frequency from 10 seconds to 1 hour.
*   **Persistence**: All your services and settings are saved automatically.
*   **Liquid Glass UI**: A modern, frosted-glass interface designed to feel native to macOS.
*   **Minimize to Tray**: Closing the window hides it back to the tray without quitting the app.

## Tech Stack

*   **Frontend**: React, TypeScript, Vite.
*   **Backend**: Rust (Tauri).
*   **Styling**: Vanilla CSS (Liquid Glass aesthetic).

## Development

### Prerequisites

*   Node.js & Yarn
*   Rust & Cargo

### Commands

**Run in Development Mode:**
```bash
yarn tauri dev
```
*Note: In dev mode, the app activation policy might behave differently (Dock icon may appear).*

**Build for Production (Mac Universal):**
```bash
yarn tauri build --target universal-apple-darwin
```
This produces a universal binary compatible with both Apple Silicon (M1/M2/M3) and Intel Macs.

## Usage

1.  Launch the app. It will appear in your system tray (top right).
2.  Click the tray icon to see a quick status list of all services.
3.  Click **"Manage Services"** to open the dashboard.
4.  Add services by Name, IP, and Port.
5.  Change the **Check Interval** or **Icon Set** in the settings bar.
6.  Close the window to minimize it back to the tray.
7.  To fully quit, select **"Quit"** from the tray menu.

## License

MIT
