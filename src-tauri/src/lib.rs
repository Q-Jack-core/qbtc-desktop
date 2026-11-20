use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_shell::process::CommandChild;

struct AuthState {
    token: String,
}

struct SidecarState {
    child: Mutex<Option<CommandChild>>,
}

#[tauri::command]
fn get_rpc_token(state: State<'_, AuthState>) -> String {
    state.token.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id() as u128;
    let ptr = &now as *const _ as u128;
    let token = format!("{:032x}{:032x}", now ^ ptr, pid ^ now);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<SidecarState>();
                if let Ok(mut lock) = state.child.lock() {
                    if let Some(child) = lock.take() {
                        let _ = child.kill();
                    }
                }
                window.app_handle().exit(0);
            }
        })
        .setup({
            let safe_token = token.clone();
            move |app| {
                use tauri_plugin_shell::ShellExt;

                let app_data_dir = app
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data directory");
                if !app_data_dir.exists() {
                    let _ = std::fs::create_dir_all(&app_data_dir);
                }

                let (_rx, child) = app
                    .shell()
                    .sidecar("quantum-btc")
                    .expect("Failed to initialize sidecar configuration")
                    .current_dir(app_data_dir)
                    .env("QBTC_RPC_TOKEN", &safe_token)
                    .spawn()
                    .expect("Failed to spawn Q-BTC Core node");

                app.manage(SidecarState {
                    child: Mutex::new(Some(child)),
                });

                println!("✅ Q-BTC Mainnet Node sidecar launched with Security Token.");
                Ok(())
            }
        })
        .manage(AuthState { token })
        .invoke_handler(tauri::generate_handler![get_rpc_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}