use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn hide_to_tray(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().ok();
    }
}

#[tauri::command]
async fn update_tray_menu(
    app: tauri::AppHandle,
    task_count: u32,
    timer_running: bool,
    timer_label: String,
    current_task: String,
) -> Result<(), String> {
    let state = app.state::<TrayState>();
    *state.task_count.lock().unwrap() = task_count;
    *state.timer_running.lock().unwrap() = timer_running;
    *state.timer_label.lock().unwrap() = timer_label;
    *state.current_task_title.lock().unwrap() = current_task;

    if let Some(tray) = app.tray_by_id("main") {
        let menu = build_menu(&app).map_err(|e| e.to_string())?;
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

struct TrayState {
    task_count: std::sync::Mutex<u32>,
    timer_running: std::sync::Mutex<bool>,
    timer_label: std::sync::Mutex<String>,
    current_task_title: std::sync::Mutex<String>,
}

fn build_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let state = app.state::<TrayState>();
    let task_count = *state.task_count.lock().unwrap();
    let timer_running = *state.timer_running.lock().unwrap();
    let timer_label = state.timer_label.lock().unwrap().clone();
    let current_task = state.current_task_title.lock().unwrap().clone();

    let task_item = MenuItemBuilder::with_id("task_count", &format!("Today's Tasks: {}", task_count))
        .enabled(false)
        .build(app)?;

    let timer_status = if timer_running {
        format!("⏱ {} {}  • running", timer_label, current_task)
    } else if !current_task.is_empty() {
        format!("⏱ {}  • paused", current_task)
    } else {
        "⏱ No active timer".to_string()
    };
    let timer_item = MenuItemBuilder::with_id("timer_status", &timer_status)
        .enabled(false)
        .build(app)?;

    let show_item = MenuItemBuilder::with_id("show", "Show Aesis")
        .accelerator("Cmd+Shift+A")
        .build(app)?;

    let quick_add_item = MenuItemBuilder::with_id("quick_add", "Quick Add Task…")
        .accelerator("Cmd+Shift+T")
        .build(app)?;

    let quick_memo_item = MenuItemBuilder::with_id("quick_memo", "Quick Memo…")
        .accelerator("Cmd+Shift+M")
        .build(app)?;

    let quit_item = PredefinedMenuItem::quit(app, Some("Quit Aesis"))?;

    let menu = MenuBuilder::new(app)
        .item(&task_item)
        .item(&timer_item)
        .separator()
        .item(&show_item)
        .item(&quick_add_item)
        .item(&quick_memo_item)
        .separator()
        .item(&quit_item)
        .build()?;

    Ok(menu)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(TrayState {
            task_count: std::sync::Mutex::new(0),
            timer_running: std::sync::Mutex::new(false),
            timer_label: std::sync::Mutex::new(String::new()),
            current_task_title: std::sync::Mutex::new(String::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            hide_to_tray,
            update_tray_menu,
        ])
        .setup(|app| {
            let handle = app.handle();
            let menu = build_menu(handle)?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Aesis")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().ok().unwrap_or(false) {
                                    window.hide().ok();
                                } else {
                                    window.show().ok();
                                    window.set_focus().ok();
                                }
                            }
                        }
                        "quick_add" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                                window.emit("show-quick-add", ()).ok();
                            }
                        }
                        "quick_memo" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                                window.emit("show-quick-memo", ()).ok();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().ok().unwrap_or(false) {
                                window.hide().ok();
                            } else {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                    }
                })
                .build(app)?;

            let window = app.get_webview_window("main").unwrap();
            let w = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    w.hide().ok();
                    api.prevent_close();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
