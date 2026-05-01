mod color;
mod trash;
mod item;
mod app;
mod ui;
mod scrap_yard;
mod scrap;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{ backend::CrosstermBackend, Terminal };
use std::io;
use std::time::Duration;
use crate::app::{App, InputMode};
use crate::ui::ui;
use crate::scrap_yard::check_trash_access;

fn main() -> io::Result<()> {
    if !check_trash_access() {
        eprintln!("scrap: no permission to access Trash.");
        eprintln!("Grant Full Disk Access to your terminal in:");
        eprintln!("  System Settings > Privacy & Security > Full Disk Access");
        eprintln!("Opening System Settings now...");
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_AllFiles")
            .spawn();
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Query => match key.code {
                            KeyCode::Esc => app.exit_query_mode(),
                            KeyCode::Enter => {
                                if app.is_query_valid() {
                                    app.run_query();
                                }
                            }
                            KeyCode::Backspace => app.query_pop(),
                            KeyCode::Char(c) => app.query_push(c),
                            _ => {}
                        },
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                            KeyCode::Char('/') => app.enter_query_mode(),
                            KeyCode::Tab | KeyCode::Right => app.next_tab(),
                            KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                            KeyCode::Down => app.next_item(),
                            KeyCode::Up => app.prev_item(),
                            _ => {}
                        },
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
