use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, List, ListState, Tabs},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

pub struct WorkState {
    pub list_state: ListState,
    pub active_pane: WorkPane,
    pub textarea: TextArea<'static>,
}

#[derive(PartialEq)]
pub enum WorkPane {
    Selector,
    Editor,
}

impl WorkState {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
        textarea.set_style(Style::default().fg(Color::LightRed));
        
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            active_pane: WorkPane::Selector,
            textarea,
        }
    }
}

pub fn render_work(frame: &mut Frame, tab_pane: Rect, work_pane: Rect, work_state: &mut WorkState) {
    let work_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(work_pane);

    let tab_widget = 
                Tabs::new(vec!["1 Components", "2 Viewpackets", "3 Diagrams", "Tab4"])
                    //.block(Block::bordered().title("Tabs"))
                    .style(Style::default().bg(Color::Gray))
                    .highlight_style(Style::default().bg(Color::LightBlue))
                    .select(2)
                    .divider(symbols::DOT)
                    .padding("->", "<-");
            frame.render_widget(tab_widget, tab_pane);

    // ......... Selector pane - left side
    let items = ["Item 1", "Item 2", "Item 3", "Item 4"];
    let selector_bg = if work_state.active_pane == WorkPane::Selector {
        Color::Blue
    } else {
        Color::DarkGray
    };
    
    let list = List::new(items)
        .style(Style::default().fg(Color::White))
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Items")
                .style(Style::default().bg(selector_bg)),
        );
    frame.render_stateful_widget(list, work_layout[0], &mut work_state.list_state);

    // ......... Editor pane - right side
    let editor_bg = if work_state.active_pane == WorkPane::Editor {
        Color::Blue
    } else {
        Color::DarkGray
    };

    work_state.textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Editor")
            .style(Style::default().bg(editor_bg)),
    );

    frame.render_widget(&work_state.textarea, work_layout[1]);
}

pub fn handle_work_key(
    work_state: &mut WorkState,
    modifiers: KeyModifiers,
    key_code: KeyCode,
) -> bool {
    match (modifiers, key_code) {
        // Tab to switch between panes
        (_, KeyCode::Tab) => {
            work_state.active_pane = match work_state.active_pane {
                WorkPane::Selector => WorkPane::Editor,
                WorkPane::Editor => WorkPane::Selector,
            };
            true
        }
        // Ctrl+Left/Right to switch panes
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            work_state.active_pane = WorkPane::Selector;
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            work_state.active_pane = WorkPane::Editor;
            true
        }
        // Handle selector pane navigation
        (_, KeyCode::Up) if work_state.active_pane == WorkPane::Selector => {
            let selected = work_state.list_state.selected().unwrap_or(0);
            let new_selected = if selected == 0 { 3 } else { selected - 1 };
            work_state.list_state.select(Some(new_selected));
            true
        }
        (_, KeyCode::Down) if work_state.active_pane == WorkPane::Selector => {
            let selected = work_state.list_state.selected().unwrap_or(0);
            let new_selected = (selected + 1) % 4;
            work_state.list_state.select(Some(new_selected));
            true
        }
        // Handle textarea input when editor is active
        // _ if work_state.active_pane == WorkPane::Editor => {
        //     let input = crossterm::event::KeyEvent::new(key_code, modifiers);
        //     work_state.textarea.input(tui_textarea::Input::from(input));
        //     true
        // }
        _ => false, // Key not handled by work pane
    }
}