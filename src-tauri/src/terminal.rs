use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, State};
use std::path::Path;

pub struct TerminalSession {
    pub pty_master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
}

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

fn command_exists(cmd: &str) -> bool {
    if cfg!(target_os = "windows") {
        std::process::Command::new("where")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[tauri::command]
pub fn spawn_terminal(id: String, profile: String, app: AppHandle, state: State<'_, TerminalState>) -> Result<(), String> {
    let pty_system = native_pty_system();

    let mut cmd = if cfg!(target_os = "windows") {
            match profile.as_str() {
                "pwsh" => {
                    if command_exists("pwsh.exe") {
                        CommandBuilder::new("pwsh.exe")
                    } else {
                        CommandBuilder::new("powershell.exe")
                    }
                },
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
                custom_path => CommandBuilder::new(custom_path), 
            }
        } else {
            let mut c = CommandBuilder::new("bash");
            c.env("TERM", "xterm-256color");
            c
        };

    let pair = pty_system.openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    let session = TerminalSession {
        pty_master: pair.master,
        writer,
    };

    state.sessions.lock().unwrap().insert(id.clone(), session);

    let _child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    
    let thread_id = id.clone();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 { break; }
            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
            
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