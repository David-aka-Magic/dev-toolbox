use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use std::path::Path;

// A struct to hold a single terminal session
pub struct TerminalSession {
    pub pty_master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
}

// The global state now holds a Map of ID -> Session
pub struct TerminalState {
    pub sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn spawn_terminal(id: String, profile: String, app: AppHandle, state: State<'_, TerminalState>) -> Result<(), String> {
    let pty_system = native_pty_system();

    // 1. SELECT THE SHELL BASED ON PROFILE
    let mut cmd = if cfg!(target_os = "windows") {
            match profile.as_str() {
                "pwsh" => CommandBuilder::new("pwsh.exe"),
                "cmd" => CommandBuilder::new("cmd.exe"),
                "git-bash" => {
                    let path = "C:\\Program Files\\Git\\bin\\bash.exe";
                    if std::path::Path::new(path).exists() {
                         CommandBuilder::new(path)
                    } else {
                        CommandBuilder::new("powershell.exe")
                    }
                },
                "wsl" => CommandBuilder::new("wsl.exe"),
                
                // CHANGE: If the profile string doesn't match a preset, 
                // assume it is a raw path provided by the user settings.
                custom_path => CommandBuilder::new(custom_path), 
            }
        } else {
            // Linux/Mac logic...
            let mut c = CommandBuilder::new("bash");
            c.env("TERM", "xterm-256color");
            c
        };

    // 2. Create the PTY pair
    let pair = pty_system.openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;

    // 3. Clone the reader for the thread
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    
    // 4. Take the writer for input handling
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    // 5. Save to HashMap using the unique ID
    let session = TerminalSession {
        pty_master: pair.master,
        writer,
    };

    state.sessions.lock().unwrap().insert(id.clone(), session);

    // 6. Spawn background thread to read output
    let _child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    
    // We move 'id' into the thread so it knows which channel to emit to
    let thread_id = id.clone();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 { break; }
            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
            
            // EMIT to a specific channel: "terminal-output-<UUID>"
            let event_name = format!("terminal-output-{}", thread_id);
            let _ = app.emit(&event_name, data);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn write_to_terminal(id: String, data: String, state: State<'_, TerminalState>) {
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(&id) {
            let _ = write!(session.writer, "{}", data);
        }
    }
}

#[tauri::command]
pub fn resize_terminal(id: String, rows: u16, cols: u16, state: State<'_, TerminalState>) {
    if let Ok(mut sessions) = state.sessions.lock() {
        if let Some(session) = sessions.get_mut(&id) {
            let _ = session.pty_master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }
}