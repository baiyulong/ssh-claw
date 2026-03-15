mod app;
mod input;
mod server;
mod ssh;
mod ui;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Screen};

#[derive(Parser)]
#[command(name = "term-ssh-manager", about = "TUI SSH Connection Manager")]
struct Cli {
    /// Path to a custom servers.json config file
    #[arg(short, long)]
    config: Option<String>,
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

fn main() -> io::Result<()> {
    install_panic_hook();

    let cli = Cli::parse();
    let config_path = server::config_path(cli.config.as_deref());

    let mut terminal = setup_terminal()?;
    let mut app = App::new(config_path);

    loop {
        // Check SSH exit first — before drawing or handling new events
        let ssh_ended = match &app.screen {
            Screen::SshSession(s) => s.is_exited(),
            _ => false,
        };
        if ssh_ended {
            app.screen = Screen::Dashboard;
            app.status_msg = "SSH session ended. Press any key.".to_string();
        }

        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll with a short timeout so we detect SSH exit promptly
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        input::handle_key(&mut app, key);
                    }
                }
                Event::Resize(w, h) => {
                    if let Screen::SshSession(ref session) = app.screen {
                        let (pty_rows, pty_cols) =
                            ui::ssh_inner_size(ratatui::layout::Size { width: w, height: h });
                        session.resize(pty_rows, pty_cols);
                    }
                }
                _ => {}
            }
        }

        // Spawn an embedded SSH session sized to the middle pane
        if let Some(idx) = app.should_ssh.take() {
            if let Some(server) = app.servers.get(idx).cloned() {
                let total = terminal.size()?;
                let (pty_rows, pty_cols) = ui::ssh_inner_size(total);
                match ssh::spawn_ssh(&server, pty_rows, pty_cols) {
                    Ok(session) => {
                        app.status_msg = server.display_connection();
                        app.screen = Screen::SshSession(session);
                    }
                    Err(e) => {
                        app.status_msg = format!("SSH error: {}", e);
                    }
                }
            }
        }

        if app.should_quit {
            restore_terminal(&mut terminal)?;
            break;
        }
    }

    Ok(())
}

