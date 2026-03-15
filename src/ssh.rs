use std::io::{Read, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use crate::server::Server;

/// Wraps `Box<dyn MasterPty>` to make it Send.
/// Safety: all concrete MasterPty implementations (Unix/Windows) are Send.
struct SendableMaster(Box<dyn portable_pty::MasterPty>);
unsafe impl Send for SendableMaster {}

/// Live SSH session running inside an in-process PTY.
pub struct SshSession {
    /// Shared vt100 terminal state updated by the reader thread.
    pub parser: Arc<Mutex<vt100::Parser>>,
    /// Write end of the PTY — keyboard bytes go here.
    pub writer: Box<dyn Write + Send>,
    /// Set to true when the PTY reader thread reaches EOF (SSH exited).
    pub exited: Arc<AtomicBool>,
}

impl SshSession {
    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }
}

impl std::fmt::Debug for SshSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SshSession(exited={})", self.is_exited())
    }
}

/// Spawn `ssh` inside a PTY sized to the current terminal.
/// Returns immediately; I/O is handled on background threads.
pub fn spawn_ssh(server: &Server, rows: u16, cols: u16) -> std::io::Result<SshSession> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(io_err)?;

    // Build the ssh command
    let mut cmd = CommandBuilder::new("ssh");
    for arg in server.ssh_args() {
        cmd.arg(arg);
    }

    // Spawn inside the PTY slave
    let child = pair.slave.spawn_command(cmd).map_err(io_err)?;
    // Close slave side in parent — the child keeps its own copy
    drop(pair.slave);

    // Destructure so we can move master into the reader thread
    let reader = pair.master.try_clone_reader().map_err(io_err)?;
    let writer = pair.master.take_writer().map_err(io_err)?;
    let master = SendableMaster(pair.master);

    let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));
    let exited = Arc::new(AtomicBool::new(false));

    // Reader thread: PTY output → vt100 parser
    {
        let parser = parser.clone();
        let exited = exited.clone();
        thread::spawn(move || {
            let _master = master; // keep PTY master alive until EOF
            let mut reader = reader;
            let mut buf = vec![0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        parser.lock().unwrap().process(&buf[..n]);
                    }
                }
            }
            exited.store(true, Ordering::Relaxed);
        });
    }

    // Reaper thread: wait for child to avoid zombie
    thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });

    Ok(SshSession {
        parser,
        writer,
        exited,
    })
}

fn io_err(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}
