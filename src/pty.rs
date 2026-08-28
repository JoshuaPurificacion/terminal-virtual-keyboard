use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
    parser: Arc<Mutex<vt100::Parser>>,
}

impl PtySession {
    pub fn spawn(
        shell_cmd: Option<String>,
        rows: u16,
        cols: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell_str = shell_cmd.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        });

        let mut parts = shell_str.split_whitespace();
        let program = parts.next().unwrap_or("/bin/bash");
        let mut cmd = CommandBuilder::new(program);
        for arg in parts {
            cmd.arg(arg);
        }
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd)?;
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let master = pair.master;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 2000)));
        let parser_clone = Arc::clone(&parser);

        // Spawn non-blocking reader loop
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut p) = parser_clone.lock() {
                            p.process(&buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master,
            writer,
            _child: child,
            parser,
        })
    }

    pub fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        if let Ok(mut p) = self.parser.lock() {
            p.set_size(rows, cols);
        }
        Ok(())
    }

    pub fn with_screen<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&vt100::Screen) -> R,
    {
        let p = self.parser.lock().unwrap();
        f(p.screen())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_pty_echo_roundtrip() {
        let mut pty = PtySession::spawn(Some("/bin/sh".to_string()), 24, 80)
            .expect("Failed to spawn PTY session");

        // Write a test command
        pty.write(b"echo CYBERDECK_M7_SUCCESS\n")
            .expect("Failed to write to PTY");

        let start = Instant::now();
        let mut found = false;

        // Poll screen contents for the output
        while start.elapsed() < Duration::from_secs(3) {
            pty.with_screen(|screen| {
                let rows: Vec<String> = screen.rows(0, 80).collect();
                let full_text = rows.join("\n");
                if full_text.contains("CYBERDECK_M7_SUCCESS") {
                    found = true;
                }
            });

            if found {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        assert!(found, "PTY did not output expected echo string");
    }
}
