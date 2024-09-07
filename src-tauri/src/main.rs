use nix::unistd::write;
use std::sync::Arc;
use std::thread;
use tauri::{http::header::CONTENT_SECURITY_POLICY, Manager, State, Window};
mod terminal_spawn;

struct TerminalFds {
    master_fd: i32,
    slave_fd: i32,
}

#[tauri::command]
fn terminal_input(key: &str, fds: State<'_, Arc<TerminalFds>>) {
    write(fds.master_fd, key.as_bytes()).expect("Failed to write to terminal");
}

#[tauri::command]
fn terminal_resize(rows: u16, cols: u16, fds: State<'_, Arc<TerminalFds>>) {
    terminal_spawn::set_terminal_size(fds.slave_fd, rows, cols);
}

fn handle_terminal_output(master_fd: i32, window: Window) {
    let mut buffer = [0u8; 1024];
    let mut output_buffer = String::new();
    loop {
        match nix::unistd::read(master_fd, &mut buffer) {
            Ok(0) => break, // End of input
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                output_buffer.push_str(&chunk);
                println!("{}", output_buffer);
                window
                    .emit("terminal_output", output_buffer.clone())
                    .unwrap();
                output_buffer.clear();
            }
            Err(_) => break, // Error occurred
        }
    }
}

fn main() {
    let fds = terminal_spawn::spawn_terminal().expect("Failed to spawn terminal");
    let terminal_fds = Arc::new(TerminalFds {
        master_fd: fds.0,
        slave_fd: fds.1,
    });

    let terminal_fds_clone = Arc::clone(&terminal_fds);

    tauri::Builder::default()
        .setup(move |app| {
            let window = app.get_window("main").unwrap();

            // Spawn a thread to continuously handle terminal output
            thread::spawn(move || {
                handle_terminal_output(terminal_fds_clone.master_fd, window);
            });

            Ok(())
        })
        .manage(terminal_fds) // Manage the TerminalFds state
        .invoke_handler(tauri::generate_handler![terminal_input, terminal_resize])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
