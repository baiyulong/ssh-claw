use std::io::{Read, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

use crate::server::Server;

struct SendableMaster(Box<dyn MasterPty>);
unsafe impl Send for SendableMaster {}

pub struct SshSession {
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub writer: Box<dyn Write + Send>,
    pub exited: Arc<AtomicBool>,
    /// Kept alive so we can resize the PTY later
    master: Arc<Mutex<SendableMaster>>,
}

impl std::fmt::Debug for SshSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SshSession(exited={})", self.is_exited())
    }
}

impl SshSession {
    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }

    /// Notify the PTY and vt100 parser of a size change.
    pub fn resize(&self, rows: u16, cols: u16) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if let Ok(m) = self.master.lock() {
            let _ = m.0.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        self.parser.lock().unwrap().set_size(rows, cols);
    }
}

pub fn spawn_ssh(server: &Server, rows: u16, cols: u16) -> std::io::Result<SshSession> {
    let rows = rows.max(1);
    let cols = cols.max(1);

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(io_err)?;

    let mut cmd = CommandBuilder::new("ssh");
    for arg in server.ssh_args() {
        cmd.arg(arg);
    }

    let child = pair.slave.spawn_command(cmd).map_err(io_err)?;
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().map_err(io_err)?;
    let writer = pair.master.take_writer().map_err(io_err)?;
    let master = Arc::new(Mutex::new(SendableMaster(pair.master)));

    let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 1000)));
    let exited = Arc::new(AtomicBool::new(false));

    // Reader thread: PTY output → vt100 parser
    {
        let parser = parser.clone();
        let exited = exited.clone();
        let _master = master.clone(); // keep master alive until EOF
        thread::spawn(move || {
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
        master,
    })
}

fn io_err(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}
