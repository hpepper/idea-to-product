use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use std::io;

use sad_xml_sql::db_create_in_mem_db;
use sad_xml_sql::db_populate_from_xml;

use rusqlite::Connection;

mod app_state;
use app_state::AppState;

mod menu;
use menu::{handle_menu_input, render_menu, MenuState};

mod work;
use work::{handle_work_input, render_work, WorkState};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let stdout = io::stdout(); // Get a handle to standard output
    let mut stdout = stdout.lock(); // Lock stdout for exclusive access (needed for terminal UI)
    enable_raw_mode()?; // Enable raw mode so we can read input directly from the terminal
    crossterm::execute!(stdout, EnterAlternateScreen)?; // Switch to the alternate screen
                                                        // TODO later add EnableMouseCapture
    let backend = CrosstermBackend::new(stdout); // Create a backend for ratatui using Crossterm
    let mut terminal = Terminal::new(backend)?; // Create a Terminal object to manage drawing

    // TODO understand what this is do
    //let mut list_state = ListState::default().with_selected(Some(0));
    let mut app_state = AppState::new();
    let mut menu_state = MenuState::new();
    let mut work_state = WorkState::new();

    let filename: String = "../sad_xml_sql/tests/test_sad.xml".to_string();
    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    db_create_in_mem_db(&db_conn);

    db_populate_from_xml(&db_conn, &filename);

    loop {
        terminal.draw(|f| {
            let panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(4),
                    Constraint::Length(3),
                ])
                .split(f.area());
            let menu_pane = panes[0];
            let tab_pane = panes[1];
            let work_pane = panes[2];
            let status_pane = panes[3];

            let version = env!("CARGO_PKG_VERSION");

            // ------------------------------- Menu pane
            render_menu(f, menu_pane, &menu_state);

            // ------------------------------- Menu pane
            render_work(
                &db_conn,
                f,
                tab_pane,
                work_pane,
                &mut work_state,
                &mut app_state,
            );

            // ------------------------------- Status pane
            let status_line = Line::from(vec![Span::styled(
                &app_state.status_message,
                Style::default().fg(Color::Red).bg(Color::Gray),
            )]);

            let status_widget = Paragraph::new(status_line).block(
                Block::default()
                    .title_bottom(format!("v{}", version))
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Gray)),
            );
            f.render_widget(status_widget, status_pane);
        })?;

        match crossterm::event::read()? {
            // input => {
            //     let modified = self.textarea.input(input);
            //     modified.then(|| self.textarea.lines()[0].as_str())
            // }
            Event::Key(event) => {
                // Put the work handling first since it might have an active textarea.
                if handle_work_input(&mut work_state, event, &db_conn) {
                    continue; // Work pane handled the key, continue to next iteration
                }
                if handle_menu_input(&mut menu_state, event) {
                    continue; // Menu handled the key, continue to next iteration
                }
                // TODO convert from input to key.
                match (event.modifiers, event.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('q') | KeyCode::Char('Q')) => {
                        break;
                    } //     // Add other key handlers here.
                    _ => {}
                }
            }
            Event::FocusGained => println!("FocusGained"),
            Event::FocusLost => println!("FocusLost"),
            Event::Mouse(event) => println!("{:?}", event),
            // TODO what does this do? #[cfg(feature = "bracketed-paste")]
            Event::Paste(data) => println!("{:?}", data),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
    } // end loop
    disable_raw_mode()?; // Restore terminal to normal mode
    crossterm::execute!(
        terminal.backend_mut(), // Get the backend for cleanup
        LeaveAlternateScreen    // Leave the alternate screen
    )?;
    terminal.show_cursor()?; // Show the cursor again
    Ok(())
}
