use std::default;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::stdout;

use sad_xml_sql::db_population::{populate_db};
use sad_xml_sql::db_retrieval;
use sad_xml_sql::models;

use rusqlite::Connection;

mod app_state;
use app_state::AppState;

mod menu;
use menu::{handle_menu_key, render_menu, MenuState};

mod work;
use work::{handle_work_key, render_work, WorkState};



fn main() -> color_eyre::Result<()> {
    // color_eyre::install()?;
    // let backend = CrosstermBackend::new(stdout());
    // let mut terminal = Terminal::new(backend)?;
    let mut terminal = ratatui::init();

    // TODO understand what this is do
    //let mut list_state = ListState::default().with_selected(Some(0));
    let mut app_state = AppState::new();
    let mut menu_state = MenuState::new();
    let mut work_state = WorkState::new();

    let filename: String = "../sad_xml_sql/test/test_sad.xml".to_string();
    let db_conn = Connection::open_in_memory().expect("Connecting to the SQLite database failed.");

    populate_db(&db_conn, &filename);

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
            render_work(&db_conn, f, tab_pane,work_pane, &mut work_state, &mut app_state);

            // ------------------------------- Status pane
            let status_line = Line::from(vec![
                Span::styled(&app_state.status_message, Style::default().fg(Color::Red).bg(Color::Gray)),
            ]);

            let status_widget = Paragraph::new(status_line).block(
                Block::default()
                    .title_bottom(format!("v{}", version))
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Gray)),
            );
            f.render_widget(status_widget, status_pane);
        })?;

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if handle_menu_key(&mut menu_state, key.modifiers, key.code) {
                    continue; // Menu handled the key, continue to next iteration
                }
                // Try work pane key handling
                if handle_work_key(&mut work_state, key.modifiers, key.code) {
                    continue; // Work pane handled the key, continue to next iteration
                }
                match (key.modifiers, key.code) {
                    (_, KeyCode::Esc | KeyCode::Char('q'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                        break;
                    }
                    // Add other key handlers here.
                    _ => {}
                }
            }
            Event::Mouse(_) => {
                // Handle mouse events
            }
            Event::Resize(_, _) => {
                // Handle resize events
            }
            _ => {}
        }
    } // end loop
    ratatui::restore();
    Ok(())
}
