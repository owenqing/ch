use anyhow::Result;
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use std::io::stdout;
use clap::{Arg, Command};
use std::path::PathBuf;

mod config;
mod app;
mod ui;
mod command;

use config::load_config;
use app::AppState;
use ui::render_ui;
use crate::command::execute_command;
use crossterm::terminal;

fn main() -> Result<()> {
    let matches = Command::new("connection-manager")
        .version("0.1.0")
        .about("a commands manager")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("config path，default ~/.ch/config.toml")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .get_matches();

    let config_path = if let Some(path) = matches.get_one::<String>("config") {
        PathBuf::from(path)
    } else {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("can't get home dir"))?;
        home.join(".ch/config.toml")
    };

    let _ = env_logger::try_init();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let config = match load_config(config_path.to_str().unwrap()) {
        Ok(cfg) => cfg,
        Err(e) => {
            terminal::disable_raw_mode()?;
            eprintln!("load config file failed: {}\npath: {}", e, config_path.display());
            std::process::exit(1);
        }
    };
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;
    let mut app = AppState::new();
    let res = run_app(&mut terminal, &config, &mut app);
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    match res {
        Ok(Some(command)) => {
            if let Err(e) = execute_command(command) {
                eprintln!("Error executing command: {}", e);
            }
        },
        Ok(None) => {},
        Err(e) => {
            eprintln!("Application error: {}", e);
        }
    }
    Ok(())
}

fn run_app<B: tui::backend::Backend>(
    terminal: &mut tui::Terminal<B>,
    config: &config::Config,
    app: &mut AppState,
) -> Result<Option<String>> {
    loop {
        render_ui(terminal, config, app)?;
        if let Some(key_event) = poll_key_event()? {
            use crossterm::event::{KeyCode, KeyModifiers};
            match key_event.code {
                KeyCode::Char('q') if !app.search_mode => break,
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Left => app.handle_left(),
                KeyCode::Right => app.handle_right(),
                KeyCode::Down => {
                    let group_names: Vec<_> = config.groups.keys().cloned().collect();
                    let filtered_connections = ui::get_filtered_connections(config, &group_names, app);
                    app.move_down(group_names.len(), filtered_connections.len());
                },
                KeyCode::Up => {
                    let group_names: Vec<_> = config.groups.keys().cloned().collect();
                    let filtered_connections = ui::get_filtered_connections(config, &group_names, app);
                    app.move_up(group_names.len(), filtered_connections.len());
                },
                KeyCode::Enter => {
                    if app.focus == 1 {
                        let group_names: Vec<_> = config.groups.keys().cloned().collect();
                        let filtered_connections = ui::get_filtered_connections(config, &group_names, app);
                        if !filtered_connections.is_empty() {
                            if let Some((group_name, command)) = filtered_connections.get(app.current_selection).cloned() {
                                if let Some(_group) = config.groups.get(&group_name) {
                                    let description = _group.connections.get(command.as_str()).unwrap_or(&command);
                                    println!("execute command: {}", description);
                                    return Ok(Some(description.to_string()));
                                }
                            }
                        }
                    }
                },
                KeyCode::Char('/') => app.enter_search(),
                KeyCode::Esc => app.reset_search(),
                KeyCode::Char(c) => {
                    if app.search_mode {
                        app.search_query.push(c);
                        app.current_selection = 0;
                    }
                },
                KeyCode::Backspace => {
                    if app.search_mode && !app.search_query.is_empty() {
                        app.search_query.pop();
                        app.current_selection = 0;
                    }
                },
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(None)
}

fn poll_key_event() -> Result<Option<crossterm::event::KeyEvent>> {
    use crossterm::event::{self, Event};
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}
