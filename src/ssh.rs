use std::process::Command;

use crate::server::Server;

/// Run SSH, returning the exit status.
/// The caller must restore the terminal before calling this.
pub fn run_ssh(server: &Server) -> std::io::Result<std::process::ExitStatus> {
    let args = server.ssh_args();
    Command::new("ssh").args(&args).status()
}
