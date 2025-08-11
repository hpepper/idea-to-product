use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListState, Paragraph, Tabs},
    Frame,
};
use rusqlite::Connection;
use sad_xml_sql::{
    get_component_by_name, get_vector_of_component_names_sorted,
    get_vector_of_viewpacket_titles_sorted, get_viewpacket_by_title,
};
use tui_textarea::{Input, Key, TextArea};

use super::app_state::AppState;

pub struct WorkState {
    pub list_state: ListState,
    pub active_pane: WorkPane,
    pub textarea: TextArea<'static>, // TODO delete this, I don't think we need it here
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
    Details,
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

    // Generate the list of items for the selector pane.
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

    let selected_index: usize;
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

    if work_state.active_pane == WorkPane::Selector {
        app_state.status_message = format!(
        "F4: Edit  - Tab: {}, Active Pane: {:?}, Selected Item: {} - {} - Total Items: {} - Cloned Objects: {}",
        work_state.selected_tab.as_index(),
        work_state.active_pane,
        selected_index,
        selected_item,
        work_state.current_selection_list.len(),
        work_state.number_of_cloned_objects
    );
    } else if work_state.active_pane == WorkPane::Editor {
        app_state.status_message = format!(
            "ESC: Cancel  F7: Save - Tab: {}, Active Pane: {:?}, Selected Item: {} - {}",
            work_state.selected_tab.as_index(),
            work_state.active_pane,
            selected_index,
            selected_item
        );
    } else {
        app_state.status_message = format!(
            "XXX - Tab: {}, Active Pane: {:?}, Selected Item: {} - {}",
            work_state.selected_tab.as_index(),
            work_state.active_pane,
            selected_index,
            selected_item
        );
    }

    // ......... Editor pane - right side
    let _editor_bg = if work_state.active_pane == WorkPane::Editor {
        Color::Blue
    } else {
        Color::DarkGray
    };

    match work_state.selected_tab {
        TabSubjects::Components => {
            if work_state.active_pane == WorkPane::Selector
                || work_state.active_pane == WorkPane::Details
            {
                render_component_details_pane(
                    frame,
                    work_state,
                    db_conn,
                    work_layout[1],
                    selected_item,
                );
            } else {
                render_component_editor_pane(
                    frame,
                    work_state,
                    db_conn,
                    work_layout[1],
                    selected_item,
                )
            }
        }
        TabSubjects::ViewPackets => render_viewpacket_details_pane(
            frame,
            work_state,
            db_conn,
            work_layout[1],
            selected_item,
        ),
        TabSubjects::Diagrams => {
            render_diagram_details_pane(frame, work_state, db_conn, work_layout[1], selected_item)
        }
    }
}

fn render_component_details_pane(
    frame: &mut Frame,
    _work_state: &mut WorkState,
    db_conn: &Connection,
    area: Rect,
    selected_item: String,
) {
    let component = get_component_by_name(db_conn, selected_item);

    let text = if let Ok(component) = component {
        let line_id = Line::from(vec![
            Span::styled(
                "id.....: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", component.id),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_name = Line::from(vec![
            Span::styled(
                "name...: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", component.name),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_purpose = Line::from(vec![
            Span::styled(
                "purpose: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", component.purpose),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_summary = Line::from(vec![
            Span::styled(
                "summary: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", component.summary),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        Text::from(vec![line_id, line_name, line_purpose, line_summary])
    } else {
        Text::from("no data available for this component")
    };

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Component")
            .style(Style::default().bg(Color::DarkGray)),
    );
    frame.render_widget(paragraph, area);
}

/*
 Since TextArea can't be combined with anything else, then the label and input needs to be split
 into separate panes.
 So split the editor pane into two panes: one for the label and one for the input.
 and then each input is in its own row.

 First split the pane vertically into as many parts/rows as there are fields.
 Then render each row, with a fixed left(label) and right(input) part
*/
// https://github.com/rhysd/tui-textarea
fn render_component_editor_pane(
    frame: &mut Frame,
    _work_state: &mut WorkState,
    db_conn: &Connection,
    area: Rect,
    selected_item: String,
) {
    let component = get_component_by_name(db_conn, selected_item);

    if let Ok(component) = component {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // id row
                Constraint::Length(3), // name row
                Constraint::Length(5), // purpose row
                Constraint::Length(5), // summary row
            ])
            .split(area);

        //render_component_detail_row(frame, rows[0], "id.....:", component.id);
        render_component_detail_row(frame, rows[1], "name...:", component.name);
        //render_component_detail_row(frame, rows[2], "purpose:", component.purpose);
        //render_component_detail_row(frame, rows[3], "summary:", component.summary);
    }
}

fn render_component_detail_row(frame: &mut Frame, area: Rect, label: &str, value: impl ToString) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // label column
            Constraint::Min(10),    // textarea column
        ])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(Color::Yellow));
    frame.render_widget(label_widget, chunks[0]);

    let textarea = TextArea::new(vec![value.to_string()]);
    frame.render_widget(&textarea, chunks[1]);
}

fn render_diagram_details_pane(
    frame: &mut Frame,
    work_state: &mut WorkState,
    _db_conn: &Connection,
    area: Rect,
    _selected_item: String,
) {
    work_state.textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Diagram")
            .style(Style::default().bg(Color::DarkGray)),
    );

    frame.render_widget(&work_state.textarea, area);
}

fn render_viewpacket_details_pane(
    frame: &mut Frame,
    _work_state: &mut WorkState,
    db_conn: &Connection,
    area: Rect,
    selected_item: String,
) {
    let viewpacket = get_viewpacket_by_title(db_conn, &selected_item);

    let text = if let Ok(viewpacket) = viewpacket {
        let line_id = Line::from(vec![
            Span::styled(
                "id.................: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.viewpacket_id),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_title = Line::from(vec![
            Span::styled(
                "title..............: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.title),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_introduction = Line::from(vec![
            Span::styled(
                "introduction.......: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.introduction),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_view_style = Line::from(vec![
            Span::styled(
                "view_style.........: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.view_style),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_view_type = Line::from(vec![
            Span::styled(
                "view_type..........: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.view_type),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_primary_display_key = Line::from(vec![
            Span::styled(
                "primary_display_key: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.primary_display_key),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        let line_context_model_key = Line::from(vec![
            Span::styled(
                "context_model_key..: ",
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
            Span::styled(
                format!("{} ", viewpacket.context_model_key),
                Style::default().fg(Color::Black).bg(Color::Gray),
            ),
        ]);
        Text::from(vec![
            line_id,
            line_title,
            line_introduction,
            line_view_type,
            line_view_style,
            line_primary_display_key,
            line_context_model_key,
        ])
    } else {
        Text::from("no data available for this viewpacket")
    };

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("View Packet")
            .style(Style::default().bg(Color::DarkGray)),
    );
    frame.render_widget(paragraph, area);
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

pub fn handle_work_input(
    work_state: &mut WorkState,
    event: crossterm::event::KeyEvent,
) -> bool {
    let max_elements = work_state.tab_counts.get_count(&work_state.selected_tab);
    match (event.modifiers, event.code) {
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
        // Ctrl+Left/Right to switch panes
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            work_state.active_pane = WorkPane::Selector;
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            work_state.active_pane = WorkPane::Details;
            true
        }
        // TODO F4: Implement edit functionality
        (_, KeyCode::F(4)) if work_state.active_pane == WorkPane::Selector => {
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
