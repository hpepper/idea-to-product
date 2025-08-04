use std::default;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListState, Paragraph, Tabs},
    Terminal,
};
use std::io::stdout;
use tui_textarea::{Input, Key, TextArea};
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // TODO understand what this is do
    let mut list_state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|f| {
            // Split into three vertical panes
            //  - Menu pane
            //  - Work pane
            //  - Status pane
            let pane = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(4),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let version = env!("CARGO_PKG_VERSION");

            // ------------------------------- Menu pane
            // let line = Line::from(Span::styled(
            //     "File Edit Options Help",
            //     Style::default().fg(Color::Black).bg(Color::Gray),
            // ));
            let line = Line::from(vec![
                Span::styled("F", Style::default().fg(Color::Red).bg(Color::Gray)),
                Span::styled("ile ", Style::default().fg(Color::Black).bg(Color::Gray)),
                Span::styled("E", Style::default().fg(Color::Red).bg(Color::Gray)),
                Span::styled("dit ", Style::default().fg(Color::Black).bg(Color::Gray)),
                Span::styled("O", Style::default().fg(Color::Red).bg(Color::Gray)),
                Span::styled("ptions ", Style::default().fg(Color::Black).bg(Color::Gray)),
                Span::styled("H", Style::default().fg(Color::Red).bg(Color::Gray)),
                Span::styled("elp", Style::default().fg(Color::Black).bg(Color::Gray)),
            ]);

            let menu_widget = Paragraph::new(line).block(
                Block::default()
                    .title("Chase Game : q - quit")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Gray)),
            );
            f.render_widget(menu_widget, pane[0]);

            // ------------------------------- tab pane
            // TODO add the tab bar here: Components | Viewpackets | Diagrams
            let tab_widget = 
                Tabs::new(vec!["1 Components", "2 Viewpackets", "3 Diagrams", "Tab4"])
                    //.block(Block::bordered().title("Tabs"))
                    .style(Style::default().bg(Color::Gray))
                    .highlight_style(Style::default().bg(Color::LightBlue))
                    .select(2)
                    .divider(symbols::DOT)
                    .padding("->", "<-");
            f.render_widget(tab_widget, pane[1]);

            // ------------------------------- work pane
            let work_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
                .split(pane[2]);

            // ......... Selector pane - left side
            let items = ["Item 1", "Item 2", "Item 3", "Item 4"];
            let list = List::new(items)
                .style(Color::White)
                .highlight_style(Modifier::REVERSED)
                .highlight_symbol("> ")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Blue)),
                );
            f.render_stateful_widget(list, work_layout[0], &mut list_state);
            // ......... information pane - right side
            // TODO it seems the text color is set per text line
            let static_lines = [
                "ID: 1",
                "Name: something",
                "Summary: just something I read the other day.",
                "Purpose: to test the editor",
            ];
            let mut textarea = TextArea::default();
            textarea.set_cursor_line_style(Style::default());
            textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
            //let editing = Paragraph::new("This is where you edit the text.")
            textarea.set_style(Style::default().fg(Color::LightRed));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Blue)),
            );
            // It seems the .fh sets the color of the border, so maybe I can use this to indicate which pane is active

            f.render_widget(&textarea, work_layout[1]);

            // ------------------------------- Status pane
            let line = Line::from(vec![
                Span::styled("F1", Style::default().fg(Color::Red).bg(Color::Gray)),
                Span::styled(" Help", Style::default().fg(Color::Black).bg(Color::Gray)),
            ]);

            let status_widget = Paragraph::new(line).block(
                Block::default()
                    .title_bottom(format!("v{}", version))
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Gray)),
            );
            f.render_widget(status_widget, pane[3]);
        })?;

        if let Event::Key(key) = event::read()? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match (key.modifiers, key.code) {
                        (_, KeyCode::Esc | KeyCode::Char('q'))
                        | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                            break;
                        }
                        // Add other key handlers here.
                        _ => {}
                    }
                }
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    ratatui::restore();
    Ok(())
}
