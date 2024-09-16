use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener};
use tauri_plugin_shell::{process::CommandEvent, ShellExt};

pub async fn call_utility(
    handle: AppHandle,
    utility: String,
    args: Vec<String>,
    is_sidecar: bool,
) -> bool {
    let (mut rx, child) = if is_sidecar {
        handle
            .shell()
            .sidecar(utility)
            .expect("Sidecard not provided")
            .args(args)
            .spawn()
            .expect("Failed to spawn sidecar process")
    } else {
        handle
            .shell()
            .command(utility)
            .args(args)
            .spawn()
            .expect("Failed to spawn process")
    };

    let child = Arc::new(Mutex::new(Some(child)));
    let child_clone = Arc::clone(&child);

    handle.listen("kill-utility", move |_| {
        if let Ok(mut child_guard) = child_clone.lock() {
            if let Some(child) = child_guard.take() {
                child.kill().expect("Failed to kill process");
                println!("Utility process killed by event");
            }
        }
    });

    let mut is_success = false;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(data) => {
                println!("{}", String::from_utf8_lossy(&data));
                handle
                    .emit("utility-stdout", String::from_utf8_lossy(&data))
                    .unwrap();
            }
            CommandEvent::Stderr(data) => {
                println!("{}", String::from_utf8_lossy(&data));
                handle
                    .emit("utility-stderr", String::from_utf8_lossy(&data))
                    .unwrap();
            }
            CommandEvent::Error(error) => {
                println!("{}", error);
                handle.emit("utility-error", error).unwrap();
            }
            CommandEvent::Terminated(status) => {
                if let Some(code) = status.code {
                    handle.emit("utility-terminated", code.to_string()).unwrap();
                    if code == 0 {
                        is_success = true;
                    }
                }
                if let Some(_signal) = status.signal {}
                break;
            }
            _ => {}
        }
    }

    is_success
}

pub async fn call_utility2(
    handle: AppHandle,
    utility: String,
    args: Vec<String>,
    is_sidecar: bool,
) -> Result<String, String> {
    let (mut rx, child) = if is_sidecar {
        handle
            .shell()
            .sidecar(utility)
            .expect("Sidecard not provided")
            .args(args)
            .spawn()
            .expect("Failed to spawn sidecar process")
    } else {
        handle
            .shell()
            .command(utility)
            .args(args)
            .spawn()
            .expect("Failed to spawn process")
    };

    let child = Arc::new(Mutex::new(Some(child)));
    let child_clone = Arc::clone(&child);

    handle.listen("kill-utility", move |_| {
        if let Ok(mut child_guard) = child_clone.lock() {
            if let Some(child) = child_guard.take() {
                child.kill().expect("Failed to kill process");
                println!("Utility process killed by event");
            }
        }
    });

    let mut stdout_buffer = String::new();
    let mut stderr_buffer = String::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(data) => {
                let output = String::from_utf8_lossy(&data);
                println!("{}", output);
                handle.emit("utility-stdout", output.to_string()).unwrap();
                stdout_buffer.push_str(&output);
            }
            CommandEvent::Stderr(data) => {
                let output = String::from_utf8_lossy(&data);
                println!("{}", output);
                handle.emit("utility-stderr", output.to_string()).unwrap();
                stderr_buffer.push_str(&output);
            }
            CommandEvent::Error(error) => {
                println!("{}", error);
                handle.emit("utility-error", error.to_string()).unwrap();
                return Err(error.to_string());
            }
            CommandEvent::Terminated(status) => {
                if let Some(code) = status.code {
                    handle.emit("utility-terminated", code.to_string()).unwrap();
                    if code != 0 {
                        return Err(format!("Process exited with code {}", code));
                    }
                }
                break;
            }
            _ => {}
        }
    }

    Ok(stdout_buffer)
}
