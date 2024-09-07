use nix::libc::winsize;
use nix::libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, TIOCSCTTY};
use nix::pty::{openpty, OpenptyResult};
use nix::unistd::{close, dup2, execvp, fork, write, ForkResult};
use std::ffi::CString;
use std::os::unix::io::RawFd;

fn set_controlling_terminal(slave_fd: RawFd) -> nix::Result<()> {
    let res = unsafe { libc::ioctl(slave_fd, TIOCSCTTY as _, 0) };
    if res == -1 {
        Err(nix::errno::Errno::last())
    } else {
        Ok(())
    }
}

pub fn set_terminal_size(fd: RawFd, rows: u16, cols: u16) -> nix::Result<()> {
    println!("{} {}", rows, cols);
    let ws = winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let res = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ as _, &ws) };
    if res == -1 {
        Err(nix::errno::Errno::last())
    } else {
        Ok(())
    }
}

pub fn spawn_terminal() -> nix::Result<(RawFd, RawFd)> {
    let OpenptyResult { master, slave, .. } = openpty(None, None)?;
    let _ = set_terminal_size(slave, 24, 57);
    match unsafe { fork()? } {
        ForkResult::Child => {
            close(master)?;

            // Create a new session
            if nix::unistd::setsid().is_err() {
                eprintln!("Failed to setsid");
                return Err(nix::errno::Errno::ECHILD);
            }

            // Set the slave as the controlling terminal
            set_controlling_terminal(slave)?;

            // Redirect standard file descriptors to the slave
            dup2(slave, STDIN_FILENO)?;
            dup2(slave, STDOUT_FILENO)?;
            dup2(slave, STDERR_FILENO)?;

            close(slave)?;

            // Execute the shell
            let shell = CString::new("/bin/bash").unwrap();
            let args = &[shell.as_c_str()];

            execvp(args[0], args)?;

            eprintln!("execvp failed");
            std::process::exit(1);
        }
        ForkResult::Parent { child } => {
            // Close the slave side in the parent process
            close(slave)?;

            // Optional: Add a short sleep to ensure output is flushed
            std::thread::sleep(std::time::Duration::from_millis(100));

            Ok((master, slave))
        }
    }
}
