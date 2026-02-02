#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, State,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Service {
    name: String,
    ip: String,
    port: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppStateData {
    services: Vec<Service>,
    interval_secs: u64,
    #[serde(default = "default_icon_set")]
    icon_set: String, // "default" or "alt"
    #[serde(skip)]
    is_healthy: bool, // Runtime only, defaults to true
}

fn default_icon_set() -> String {
    "default".to_string()
}

// Global state now includes the persistence path
struct AppState {
    data: Arc<Mutex<AppStateData>>,
    file_path: Arc<Mutex<PathBuf>>,
}

// Helper to save state
fn save_state(data: &AppStateData, path: &PathBuf) {
    // println!("Saving state to {:?}", path);
    match serde_json::to_string_pretty(data) {
        Ok(json) => {
            if let Err(e) = fs::write(path, &json) {
                println!("Failed to write state: {}", e);
            } else {
                println!("State saved. Content: {}", json);
            }
        }
        Err(e) => println!("Failed to serialize state: {}", e),
    }
}

// Helper to update tray icon
fn update_tray_icon(app: &tauri::AppHandle, icon_set: &str, is_healthy: bool) {
    if let Ok(resource_path) = app
        .path()
        .resolve("icons", tauri::path::BaseDirectory::Resource)
    {
        // println!("Resolved icons path: {:?}", resource_path);
        let icon_name = if icon_set == "alt" {
            if is_healthy {
                "checked.png"
            } else {
                "cross.png"
            }
        } else {
            // Default
            if is_healthy {
                "green.png"
            } else {
                "red.png"
            }
        };

        let icon_path = resource_path.join(icon_name);

        if let Ok(icon_bytes) = fs::read(&icon_path) {
            if let Ok(icon) = Image::from_bytes(&icon_bytes) {
                if let Some(tray) = app.tray_by_id("main") {
                    let is_template = icon_set == "alt";
                    // Set icon first, then template status. Some platforms reset status on icon change.
                    let _ = tray.set_icon(Some(icon));
                    let _ = tray.set_icon_as_template(is_template);
                } else {
                    println!("Failed to find tray by id 'main'");
                }
            } else {
                println!("Failed to parse icon image");
            }
        } else {
            println!("Failed to read icon file: {:?}", icon_path);
        }
    }
}

#[tauri::command]
fn add_service(
    state: State<AppState>,
    name: String,
    ip: String,
    port: String,
) -> Result<Vec<Service>, String> {
    let mut data = state.data.lock().map_err(|_| "Failed to lock state")?;
    data.services.push(Service { name, ip, port });

    // Save
    let path = state.file_path.lock().map_err(|_| "Failed to lock path")?;
    save_state(&data, &path);

    Ok(data.services.clone())
}

#[tauri::command]
fn list_services(state: State<AppState>) -> Result<Vec<Service>, String> {
    let data = state.data.lock().map_err(|_| "Failed to lock state")?;
    Ok(data.services.clone())
}

#[tauri::command]
fn remove_service(state: State<AppState>, index: usize) -> Result<Vec<Service>, String> {
    let mut data = state.data.lock().map_err(|_| "Failed to lock state")?;
    if index < data.services.len() {
        data.services.remove(index);

        // Save
        let path = state.file_path.lock().map_err(|_| "Failed to lock path")?;
        save_state(&data, &path);

        Ok(data.services.clone())
    } else {
        Err("Index out of bounds".to_string())
    }
}

#[tauri::command]
fn update_service(
    state: State<AppState>,
    index: usize,
    name: String,
    ip: String,
    port: String,
) -> Result<Vec<Service>, String> {
    let mut data = state.data.lock().map_err(|_| "Failed to lock state")?;
    if index < data.services.len() {
        data.services[index] = Service { name, ip, port };

        // Save
        let path = state.file_path.lock().map_err(|_| "Failed to lock path")?;
        save_state(&data, &path);

        Ok(data.services.clone())
    } else {
        Err("Index out of bounds".to_string())
    }
}

#[tauri::command]
fn set_interval(state: State<AppState>, interval: u64) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|_| "Failed to lock state")?;
    data.interval_secs = interval;

    // Save
    let path = state.file_path.lock().map_err(|_| "Failed to lock path")?;
    save_state(&data, &path);

    Ok(())
}

#[tauri::command]
fn get_interval(state: State<AppState>) -> Result<u64, String> {
    let data = state.data.lock().map_err(|_| "Failed to lock state")?;
    Ok(data.interval_secs)
}

#[tauri::command]
fn set_icon_set(
    app: tauri::AppHandle,
    state: State<AppState>,
    preference: String,
) -> Result<(), String> {
    // println!(
    //     "Command 'set_icon_set' invoked with preference: {}",
    //     preference
    // );
    let mut data = state.data.lock().map_err(|_| "Failed to lock state")?;
    data.icon_set = preference.clone();

    // Immediate Update using current health state
    update_tray_icon(&app, &preference, data.is_healthy);

    // Save
    let path = state.file_path.lock().map_err(|_| "Failed to lock path")?;
    save_state(&data, &path);

    Ok(())
}

#[tauri::command]
fn get_icon_set(state: State<AppState>) -> Result<String, String> {
    let data = state.data.lock().map_err(|_| "Failed to lock state")?;
    Ok(data.icon_set.clone())
}

// Returns a vector of tuples: (Service, is_healthy)
fn check_lab_status(services: &[Service]) -> Vec<(Service, bool)> {
    let mut results = Vec::new();

    for service in services {
        let address = format!("{}:{}", service.ip, service.port);
        // Timeout set to 2 seconds
        let is_healthy = TcpStream::connect_timeout(
            &address.parse().unwrap_or("0.0.0.0:0".parse().unwrap()),
            Duration::from_secs(2),
        )
        .is_ok();

        if !is_healthy {
            println!("❌ {} ({}) is DOWN", service.name, address);
        }

        results.push((service.clone(), is_healthy));
    }

    // Only print if everything is okay
    if results.iter().all(|(_, healthy)| *healthy) && !results.is_empty() {
        println!("✅ All Systems Normal");
    }

    results
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Set Activation Policy to Accessory (No Dock Icon, No App Switcher)
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 1. Resolve Config Path
            let app_context = app.path();
            let app_data_dir = app_context
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));

            if !app_data_dir.exists() {
                let _ = fs::create_dir_all(&app_data_dir);
            }

            let file_path = app_data_dir.join("settings.json");
            println!("Configuration file: {:?}", file_path);

            // 2. Load State or Default
            let mut initial_data = AppStateData {
                services: vec![
                    Service {
                        name: "Google DNS".into(),
                        ip: "8.8.8.8".into(),
                        port: "53".into(),
                    },
                    Service {
                        name: "Localhost HTTP".into(),
                        ip: "127.0.0.1".into(),
                        port: "80".into(),
                    },
                ],
                interval_secs: 10,
                icon_set: default_icon_set(),
                is_healthy: true,
            };

            if file_path.exists() {
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        match serde_json::from_str::<AppStateData>(&content) {
                            Ok(saved_data) => {
                                println!("Loaded settings from disk.");
                                initial_data = saved_data;
                                // Reset runtime flag just in case
                                initial_data.is_healthy = true;
                            }
                            Err(e) => {
                                println!("Failed to deserialize settings: {}", e);
                            }
                        }
                    }
                    Err(e) => println!("Failed to read settings file: {}", e),
                }
            }

            // 3. Init State
            let app_state = AppState {
                data: Arc::new(Mutex::new(initial_data)),
                file_path: Arc::new(Mutex::new(file_path)),
            };

            // Manage state manually since we are inside setup?
            // Usually .manage() calls happen before setup, but we need dynamic path in setup.
            // But we can call app.manage() inside setup for tauri v1.
            // In v2 check docs, but app.manage() should work on AppHandle or App.
            app.manage(app_state);

            // Create initial menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Manage Services", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&show_i, &sep, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let handle = app.handle().clone();
            // Get a reference to the state to pass to the thread
            let state = app.state::<AppState>();
            let shared_data = state.data.clone();

            tauri::async_runtime::spawn(async move {
                let mut last_check = Instant::now();
                // Hack: subtract a large duration to force immediate check
                last_check = last_check - Duration::from_secs(3600);

                loop {
                    // 1. Get current interval and service list
                    let (interval, services, icon_set) = {
                        let data = shared_data.lock().unwrap();
                        (
                            data.interval_secs,
                            data.services.clone(),
                            data.icon_set.clone(),
                        )
                    };

                    // 2. Check if it's time to run
                    if last_check.elapsed() >= Duration::from_secs(interval) {
                        let health_results = check_lab_status(&services);
                        last_check = Instant::now();

                        // Determine overall health (Red if ANY service is down)
                        let is_overall_healthy = health_results.iter().all(|(_, healthy)| *healthy);

                        // Store current health status in state for immediate updates
                        if let Ok(mut data) = shared_data.lock() {
                            data.is_healthy = is_overall_healthy;
                        }

                        // Update Icon using helper
                        update_tray_icon(&handle, &icon_set, is_overall_healthy);

                        // Update Menu
                        if let Some(tray) = handle.tray_by_id("main") {
                            let show_i = MenuItem::with_id(
                                &handle,
                                "show",
                                "Manage Services",
                                true,
                                None::<&str>,
                            );
                            let quit_i =
                                MenuItem::with_id(&handle, "quit", "Quit", true, None::<&str>);
                            let sep = PredefinedMenuItem::separator(&handle);

                            if let (Ok(show), Ok(quit), Ok(sep)) = (show_i, quit_i, sep) {
                                let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> =
                                    vec![Box::new(show), Box::new(sep.clone())];

                                for (svc, healthy) in &health_results {
                                    let icon = if *healthy { "✅" } else { "❌" };
                                    let text = format!("{} {}", icon, svc.name);
                                    if let Ok(item) = MenuItem::with_id(
                                        &handle,
                                        "status",
                                        &text,
                                        false,
                                        None::<&str>,
                                    ) {
                                        items.push(Box::new(item));
                                    }
                                }

                                if let Ok(sep2) = PredefinedMenuItem::separator(&handle) {
                                    items.push(Box::new(sep2));
                                }
                                items.push(Box::new(quit));

                                let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
                                    items.iter().map(|b| b.as_ref()).collect();
                                if let Ok(menu) = Menu::with_items(&handle, &item_refs) {
                                    let _ = tray.set_menu(Some(menu));
                                }
                            }
                        }
                    }

                    // Check every 1 second
                    thread::sleep(Duration::from_secs(1));
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            add_service,
            list_services,
            remove_service,
            update_service,
            set_interval,
            get_interval,
            set_icon_set,
            get_icon_set
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
