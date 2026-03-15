mod app;
mod input;
mod server;
mod ssh;
mod ui;

use std::io;

use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

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
        // Best-effort terminal restore on panic
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
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            // Ignore key release events on Windows
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            input::handle_key(&mut app, key);
        }

        // Check if we need to launch SSH
        if let Some(idx) = app.should_ssh.take() {
            if let Some(server) = app.servers.get(idx).cloned() {
                // Restore terminal for SSH
                restore_terminal(&mut terminal)?;

                eprintln!("Connecting to {} ...\n", server.display_connection());
                let status = ssh::run_ssh(&server);

                match status {
                    Ok(s) if s.success() => {
                        eprintln!("\nSSH session ended successfully.");
                    }
                    Ok(s) => {
                        eprintln!("\nSSH exited with: {}", s);
                    }
                    Err(e) => {
                        eprintln!("\nFailed to launch ssh: {}", e);
                    }
                }

                eprintln!("Press any key to return to the manager...");
                // Wait for a keypress before re-entering TUI
                enable_raw_mode()?;
                let _ = event::read();
                disable_raw_mode()?;

                // Re-setup terminal
                terminal = setup_terminal()?;
            }
        }

        if app.should_quit {
            restore_terminal(&mut terminal)?;
            break;
        }
    }

    Ok(())
}
