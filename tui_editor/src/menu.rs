use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct MenuState {
    pub active_menu: Option<usize>,
}

impl MenuState {
    pub fn new() -> Self {
        Self { active_menu: None }
    }
}

pub fn render_menu(frame: &mut Frame, area: Rect, menu_state: &MenuState) {
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
            .title("Software Architecture Documentation Editor : q - quit")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Gray)),
    );
    frame.render_widget(menu_widget, area);
}

pub fn handle_menu_key(
    menu_state: &mut MenuState,
    modifiers: KeyModifiers,
    key_code: KeyCode,
) -> bool {
    match (modifiers, key_code) {
        (KeyModifiers::ALT, KeyCode::Char('f') | KeyCode::Char('F')) => {
            menu_state.active_menu = Some(0); // File menu
            true
        }
        (KeyModifiers::ALT, KeyCode::Char('e') | KeyCode::Char('E')) => {
            menu_state.active_menu = Some(1); // Edit menu
            true
        }
        (KeyModifiers::ALT, KeyCode::Char('o') | KeyCode::Char('O')) => {
            menu_state.active_menu = Some(2); // Options menu
            true
        }
        (KeyModifiers::ALT, KeyCode::Char('h') | KeyCode::Char('H')) => {
            menu_state.active_menu = Some(3); // Help menu
            true
        }
        (_, KeyCode::Esc) => {
            menu_state.active_menu = None; // Close any open menu
            true
        }
        _ => false, // Key not handled by menu
    }
}