use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, List, ListState, Tabs},
    Frame,
};
use rusqlite::Connection;
use sad_xml_sql::{
    get_component_by_id, get_vector_of_component_names_sorted,
    get_vector_of_viewpacket_titles_sorted,
};
use tui_textarea::{Input, Key, TextArea};

use super::app_state::AppState;

pub struct WorkState {
    pub list_state: ListState,
    pub active_pane: WorkPane,
    pub textarea: TextArea<'static>,
    pub tab_counts: TabCounts,
    selected_tab: TabSubjects,
    pub current_selection_list: Vec<String>,
    pub number_of_cloned_objects: usize,
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
            tab_counts: TabCounts::new(),
            selected_tab: TabSubjects::Components,
            current_selection_list: vec![],
            number_of_cloned_objects: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WorkPane {
    Selector,
    Editor,
}

pub struct TabCounts {
    pub components: usize,
    pub view_packets: usize,
    pub diagrams: usize,
}

impl TabCounts {
    pub fn new() -> Self {
        Self {
            components: 0,
            view_packets: 0,
            diagrams: 0,
        }
    }

    pub fn get_count(&self, tab: &TabSubjects) -> usize {
        match tab {
            TabSubjects::Components => self.components,
            TabSubjects::ViewPackets => self.view_packets,
            TabSubjects::Diagrams => self.diagrams,
        }
    }

    pub fn set_count(&mut self, tab: &TabSubjects, length: usize) {
        match tab {
            TabSubjects::Components => self.components = length,
            TabSubjects::ViewPackets => self.view_packets = length,
            TabSubjects::Diagrams => self.diagrams = length,
        }
    }
}
enum TabSubjects {
    Components,
    ViewPackets,
    Diagrams,
}

impl TabSubjects {
    fn as_index(&self) -> usize {
        match self {
            TabSubjects::Components => 0,
            TabSubjects::ViewPackets => 1,
            TabSubjects::Diagrams => 2,
        }
    }
}

pub fn render_work(
    db_conn: &Connection,
    frame: &mut Frame,
    tab_pane: Rect,
    work_pane: Rect,
    work_state: &mut WorkState,
    app_state: &mut AppState,
) {
    let work_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(work_pane);

    let tab_widget = Tabs::new(vec!["1 Components", "2 Viewpackets", "3 Diagrams", "Tab4"])
        //.block(Block::bordered().title("Tabs"))
        .style(Style::default().bg(Color::Gray))
        .highlight_style(Style::default().bg(Color::LightBlue))
        .select(work_state.selected_tab.as_index())
        .divider(symbols::DOT)
        .padding("->", "<-");
    frame.render_widget(tab_widget, tab_pane);

    // ......... Selector pane - left side
    let items = match work_state.selected_tab {
        TabSubjects::Components => get_list_of_component_names(work_state, db_conn),
        TabSubjects::ViewPackets => get_list_of_viewpacket_names(work_state, db_conn),
        TabSubjects::Diagrams => vec!["Diagram 1", "Diagram 2", "Diagram 3", "Diagram 4"]
            .into_iter()
            .map(String::from)
            .collect(),
    };
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

    let mut selected_index: usize = 0;
    let selected_item: String;
    if work_state.list_state.selected().is_some() {
        selected_index = work_state.list_state.selected().unwrap();
        if work_state.current_selection_list.len() > selected_index {
            selected_item = work_state.current_selection_list[selected_index].clone();
        } else {
            selected_item = "Selection out of index".to_string();
        }
    } else {
        selected_index = 0;
        selected_item = "NAN".to_string();
    } // TODO move this a more appropriate place?
    app_state.status_message = format!(
        "Tab: {}, Active Pane: {:?}, Selected Item: {} - {} - Total Items: {} - Cloned Objects: {}",
        work_state.selected_tab.as_index(),
        work_state.active_pane,
        selected_index,
        selected_item,
        work_state.current_selection_list.len(),
        work_state.number_of_cloned_objects
    );

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
    let max_elements = work_state.tab_counts.get_count(&work_state.selected_tab);
    match (modifiers, key_code) {
        // Alt+1, Alt+2, Alt+3, Alt+4 to switch tabs
        (KeyModifiers::ALT, KeyCode::Char('1')) => {
            work_state.selected_tab = TabSubjects::Components;
            true
        }
        (KeyModifiers::ALT, KeyCode::Char('2')) => {
            work_state.selected_tab = TabSubjects::ViewPackets;
            true
        }
        (KeyModifiers::ALT, KeyCode::Char('3')) => {
            work_state.selected_tab = TabSubjects::Diagrams;
            true
        }
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

            let new_selected = if selected == 0 {
                max_elements - 1
            } else {
                selected - 1
            };
            work_state.list_state.select(Some(new_selected));
            true
        }
        (_, KeyCode::Down) if work_state.active_pane == WorkPane::Selector => {
            let selected = work_state.list_state.selected().unwrap_or(0);
            let new_selected = (selected + 1) % max_elements;
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
    // TODO maybe this handles the tabs entries and the switch between the work panes.
    // TODO then call handle_key function for the correct pane.
}

fn get_list_of_component_names(work_state: &mut WorkState, db_conn: &Connection) -> Vec<String> {
    // This function should return the actual list of component names from your data source.
    match get_vector_of_component_names_sorted(db_conn) {
        Ok(names) => {
            work_state.current_selection_list = names.clone();
            work_state.number_of_cloned_objects = work_state.current_selection_list.len();
            work_state
                .tab_counts
                .set_count(&TabSubjects::Components, names.len());
            names
        }
        Err(_) => vec![],
    }
}

fn get_list_of_viewpacket_names(work_state: &mut WorkState, db_conn: &Connection) -> Vec<String> {
    // This function should return the actual list of component names from your data source.
    match get_vector_of_viewpacket_titles_sorted(db_conn) {
        Ok(names) => {
            work_state.current_selection_list = names.clone();
            work_state.number_of_cloned_objects = work_state.current_selection_list.len();
            work_state
                .tab_counts
                .set_count(&TabSubjects::ViewPackets, names.len());
            names
        }
        Err(_) => vec![],
    }
}
