use tauri::{AppHandle, Emitter, Manager, State};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::Serialize;
use sha2::Digest;
use std::time::{Instant, Duration};
use tauri_plugin_shell::process::CommandChild;

pub struct MinerState {
    pub is_mining: Arc<AtomicBool>,
}

struct AuthState {
    token: String,
}

struct SidecarState {
    child: Mutex<Option<CommandChild>>,
}

#[derive(Clone, Serialize)]
pub struct LogPayload {
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct HashRatePayload {
    pub mhs: f64,
}

#[derive(Clone)]
struct MiningJob {
    job_id: u64,
    target: u64,
    header_base: [u8; 120],
}

#[tauri::command]
fn get_rpc_token(state: State<'_, AuthState>) -> String {
    state.token.clone()
}

#[tauri::command]
async fn start_mining(wallet_name: String, state: State<'_, MinerState>, app: AppHandle) -> Result<bool, String> {
    if state.is_mining.load(Ordering::SeqCst) { return Ok(true); }
    state.is_mining.store(true, Ordering::SeqCst);
    let flag = state.is_mining.clone();
    
    tokio::spawn(async move {
        let _ = app.emit("miner-log", LogPayload { message: format!("Connecting to Q-BTC Stratum Network...") });
        
        if let Ok(stream) = TcpStream::connect("144.172.110.193:3334").await {
            let _ = app.emit("miner-log", LogPayload { message: format!("Connected. Authorizing vault: {}", wallet_name) });
            
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            
            let sub_req = serde_json::json!({ "id": 1, "method": "mining.subscribe", "params": [] });
            let _ = write_half.write_all(format!("{}\n", sub_req).as_bytes()).await;
            
            let auth_req = serde_json::json!({ "id": 2, "method": "mining.authorize", "params": [wallet_name.clone()] });
            let _ = write_half.write_all(format!("{}\n", auth_req).as_bytes()).await;
            
            let current_job = Arc::new(RwLock::new(None::<MiningJob>));
            let total_hashes = Arc::new(AtomicU64::new(0));
            let job_version = Arc::new(AtomicU64::new(0));
            
            let (tx_share, mut rx_share) = tokio::sync::mpsc::unbounded_channel::<(u64, u64)>();
            
            let total_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) as u64;
            let core_count = if total_cores >= 8 {
                total_cores - 2
            } else if total_cores > 2 {
                total_cores - 1
            } else {
                1
            };

            let _ = app.emit("miner-log", LogPayload { message: format!("🚀 Igniting {} Mining Cores (Reserved {} for Network/OS)...", core_count, total_cores - core_count) });

            for core_id in 0..core_count {
                let job_ref = current_job.clone();
                let hashes_ref = total_hashes.clone();
                let version_ref = job_version.clone();
                let tx = tx_share.clone();
                let thread_flag = flag.clone();
                
                std::thread::spawn(move || {
                    let offset = core_id * 1_000_000_000_000;
                    let mut local_nonce = offset; 
                    let mut local_version = 0u64;
                    let mut last_sleep_time = Instant::now();
                    
                    let mut active_base = [0u8; 120];
                    let mut active_target = 0u64;
                    let mut active_jid = 0u64;
                    let mut has_job = false;

                    while thread_flag.load(Ordering::Relaxed) {
                        let current_global_version = version_ref.load(Ordering::Relaxed);
                        if current_global_version != local_version {
                            if let Ok(guard) = job_ref.read() {
                                if let Some(job) = &*guard {
                                    active_base = job.header_base;
                                    active_target = job.target;
                                    active_jid = job.job_id;
                                    local_version = current_global_version;
                                    local_nonce = offset;
                                    has_job = true;
                                }
                            }
                        }

                        if has_job {
                            let mut batch_hashes = 0;
                            let mut found_share = false;
                            
                            for _ in 0..65536 {
                                local_nonce = local_nonce.wrapping_add(1);
                                active_base[104..112].copy_from_slice(&local_nonce.to_be_bytes());
                                
                                let mut hasher = sha2::Sha256::new();
                                sha2::Digest::update(&mut hasher, &active_base);
                                let final_hash: [u8; 32] = hasher.finalize().into();
                                
                                let mut target_bytes = [0u8; 8];
                                target_bytes.copy_from_slice(&final_hash[0..8]);
                                
                                if u64::from_be_bytes(target_bytes) <= active_target {
                                    found_share = true;
                                    break;
                                }
                                batch_hashes += 1;
                            }
                            
                            hashes_ref.fetch_add(batch_hashes, Ordering::Relaxed);
                            
                            if found_share {
                                let _ = tx.send((active_jid, local_nonce));
                            }
                            
                            std::thread::yield_now();
                            if last_sleep_time.elapsed() >= Duration::from_millis(50) {
                                std::thread::sleep(Duration::from_millis(1));
                                last_sleep_time = Instant::now();
                            }
                        } else {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                });
            }
            
            let job_ref_net = current_job.clone();
            let version_ref_net = job_version.clone();
            let hashes_ref_net = total_hashes.clone();
            let app_net = app.clone();
            let flag_net = flag.clone();
            
            tokio::spawn(async move {
                let mut line = String::new();
                while flag_net.load(Ordering::SeqCst) {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(n) => {
                            if n == 0 { break; }
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                                if parsed["method"] == "mining.notify" {
                                    let params = &parsed["params"];
                                    if params.as_array().map(|a| a.len()).unwrap_or(0) >= 7 {
                                        let jid = params[0].as_u64().unwrap_or(0);
                                        let prev = params[1].as_str().unwrap_or("");
                                        let merkle = params[2].as_str().unwrap_or("");
                                        let commit = params[3].as_str().unwrap_or("");
                                        let ts = params[4].as_u64().unwrap_or(0);
                                        let target_val = params[5].as_u64().unwrap_or(0);
                                        let pool_t = params[6].as_u64().unwrap_or(0);
                                        
                                        let mut header = [0u8; 120];
                                        header[0..8].copy_from_slice(&ts.to_be_bytes());
                                        for i in 0..32 {
                                            header[8+i] = u8::from_str_radix(&prev[i*2..i*2+2], 16).unwrap_or(0);
                                            header[40+i] = u8::from_str_radix(&merkle[i*2..i*2+2], 16).unwrap_or(0);
                                            header[72+i] = u8::from_str_radix(&commit[i*2..i*2+2], 16).unwrap_or(0);
                                        }
                                        header[112..120].copy_from_slice(&target_val.to_be_bytes());
                                        
                                        {
                                            let mut job_write = job_ref_net.write().unwrap();
                                            *job_write = Some(MiningJob { job_id: jid, target: pool_t, header_base: header });
                                        }
                                        version_ref_net.fetch_add(1, Ordering::SeqCst);
                                        hashes_ref_net.store(0, Ordering::SeqCst);
                                        
                                        let _ = app_net.emit("miner-log", LogPayload { message: format!("New Task Received | Height: {}", jid) });
                                    }
                                } else if parsed["result"] == true {
                                    let _ = app_net.emit("miner-log", LogPayload { message: "✅ Share Accepted by Network!".to_string() });
                                } else if !parsed["error"].is_null() {
                                    let _ = app_net.emit("miner-log", LogPayload { message: format!("❌ Share Rejected: {}", parsed["error"]) });
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
            
            let mut last_hash_time = Instant::now();
            let mut last_hash_count = 0u64;
            let mut next_milestone = 100_000_000u64;
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
            
            while flag.load(Ordering::SeqCst) {
                tokio::select! {
                    Some((jid, nonce)) = rx_share.recv() => {
                        let _ = app.emit("miner-log", LogPayload { message: format!("Submitting Share... Nonce: {}", nonce) });
                        let sub_msg = serde_json::json!({
                            "id": 3,
                            "method": "mining.submit",
                            "params": [wallet_name.clone(), jid, nonce]
                        });
                        let _ = write_half.write_all(format!("{}\n", sub_msg).as_bytes()).await;
                    }
                    _ = ticker.tick() => {
                        let current = total_hashes.load(Ordering::Relaxed);
                        
                        if current < last_hash_count {
                            last_hash_count = 0;
                            next_milestone = 100_000_000;
                        }
                        
                        let diff = current.saturating_sub(last_hash_count);
                        let elapsed = last_hash_time.elapsed().as_secs_f64();
                        
                        if elapsed > 0.0 && diff > 0 {
                            let mhs = (diff as f64 / elapsed) / 1_000_000.0;
                            let _ = app.emit("hash-rate", HashRatePayload { mhs });
                            
                            while current >= next_milestone {
                                 let _ = app.emit("miner-log", LogPayload { 
                                     message: format!("⚡ {} million hashes calculated...", next_milestone / 1_000_000) 
                                 });
                                 next_milestone += 100_000_000;
                            }
                        }
                        
                        last_hash_count = current;
                        last_hash_time = Instant::now();
                    }
                }
            }
        } else {
            let _ = app.emit("miner-log", LogPayload { message: "Connection to Q-BTC Network failed.".into() });
        }
        flag.store(false, Ordering::SeqCst);
        let _ = app.emit("miner-log", LogPayload { message: "Mining engine halted.".into() });
    });
    
    Ok(true)
}

#[tauri::command]
fn stop_mining(state: State<'_, MinerState>) -> Result<bool, String> {
    state.is_mining.store(false, Ordering::SeqCst);
    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
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
                
                let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
                if !app_data_dir.exists() {
                    let _ = std::fs::create_dir_all(&app_data_dir);
                }

                let (_rx, child) = app.shell().sidecar("quantum-btc")
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
        .manage(MinerState {
            is_mining: Arc::new(AtomicBool::new(false)),
        })
        .manage(AuthState { token })
        .invoke_handler(tauri::generate_handler![start_mining, stop_mining, get_rpc_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}