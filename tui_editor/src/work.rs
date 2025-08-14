use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListState, Paragraph, Tabs},
    Frame,
};
use rusqlite::Connection;
use tui_textarea::{Input, Key, TextArea};

use sad_xml_sql::{
    get_component_by_name, get_vector_of_component_names_sorted,
    get_vector_of_viewpacket_titles_sorted, get_viewpacket_by_title, update_component_by_id
};

use sad_xml_sql::Component;

use super::app_state::AppState;

pub struct WorkState {
    pub list_state: ListState,
    pub active_pane: WorkPane,
    pub previous_active_pane: WorkPane,
    pub textarea: TextArea<'static>, // TODO delete this, I don't think we need it here
    pub tab_counts: TabCounts,
    selected_tab: TabSubjects,
    pub current_selection_list: Vec<String>,
    pub number_of_cloned_objects: usize,
    // Text area collections for each tab type
    pub component_text_areas: ComponentTextAreas,
    // pub viewpacket_text_areas: ViewPacketTextAreas, // Future
    // pub diagram_text_areas: DiagramTextAreas,       // Future
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
            previous_active_pane: WorkPane::Selector,
            textarea,
            tab_counts: TabCounts::new(),
            selected_tab: TabSubjects::Components,
            current_selection_list: vec![],
            number_of_cloned_objects: 0,
            component_text_areas: ComponentTextAreas::new(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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

/**
 * The work pane is horizontally split into a selector pane and a details/edit pane.
 */
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

    // Update the status message based on the active pane and selected item.
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
            "ESC: Cancel  F7: Save - Tab: {}, Active Pane: {:?}, active field: {:?} - {}",
            work_state.selected_tab.as_index(),
            work_state.active_pane,
            work_state.component_text_areas.active_field,
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
    work_state: &mut WorkState,
    db_conn: &Connection,
    area: Rect,
    selected_item: String,
) {
    // TODO fix this, it overwrites the

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // id row
            Constraint::Length(3), // name row
            Constraint::Length(5), // purpose row
            Constraint::Length(5), // summary row
        ])
        .split(area);

    render_component_detail_row(
        frame,
        rows[0],
        ComponentField::Id.label(),
        &work_state.component_text_areas.id,
    );
    render_component_detail_row(
        frame,
        rows[1],
        ComponentField::Name.label(),
        &work_state.component_text_areas.name,
    );
    render_component_detail_row(
        frame,
        rows[2],
        ComponentField::Purpose.label(),
        &work_state.component_text_areas.purpose,
    );
    render_component_detail_row(
        frame,
        rows[3],
        ComponentField::Summary.label(),
        &work_state.component_text_areas.summary,
    );
}

fn render_component_detail_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    textarea: &TextArea<'_>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // label column
            Constraint::Min(10),    // textarea column
        ])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(Color::Yellow));
    frame.render_widget(label_widget, chunks[0]);

    frame.render_widget(textarea, chunks[1]);
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
    db_conn: &Connection,
) -> bool {
    let max_elements = work_state.tab_counts.get_count(&work_state.selected_tab);

    // TODO if there is an active TextArea, then handle that first
    /* when in edit mode
       tab | shift+tab - move between fields.
       esc - cancel change
       f7 - save and exit edit mode.
       F1 - help(for editing)
    */

    // TODO make this into a function common with the same call in the render_work()
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

    let event_handled = if work_state.active_pane == WorkPane::Editor {
        match (event.modifiers, event.code) {
            (KeyModifiers::NONE, KeyCode::Tab) => {
                match work_state.selected_tab {
                    TabSubjects::Components => {
                        work_state.component_text_areas.next_field();
                        work_state.component_text_areas.update_styling();
                    }
                    // Handle other tabs...
                    _ => {}
                }
                true
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                match work_state.selected_tab {
                    TabSubjects::Components => {
                        work_state.component_text_areas.previous_field();
                        work_state.component_text_areas.update_styling();
                    }
                    // Handle other tabs...
                    _ => {}
                }
                true
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                work_state.active_pane = work_state.previous_active_pane;
                true
            }
            (KeyModifiers::NONE, KeyCode::F(7)) => {
                // TODO have a var that stores the previous active_pane, so we can return to that instead
                work_state.active_pane = work_state.previous_active_pane;
                match work_state.selected_tab {
                    TabSubjects::Components => {
                        let component = work_state
                            .component_text_areas
                            .extract_component();
                        update_component_by_id(
                            db_conn,
                            &component,
                        );
                        true
                    }
                    _ => false, // No edit functionality for other tabs yet
                }
            }
            _ => {
                // Pass input to active text area
                match work_state.selected_tab {
                    TabSubjects::Components => {
                        work_state
                            .component_text_areas
                            .handle_input(keyevent_to_input(event))
                        //work_state.component_text_areas.handle_input(Input::from(event))
                    }
                    // Handle other tabs...
                    _ => false,
                }
            }
        }
    } else {
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
            // TODO also handle if the active pane is the WorkPane::Details.
            (_, KeyCode::F(4)) => {
                work_state.previous_active_pane = work_state.active_pane;
                work_state.active_pane = WorkPane::Editor;
                match work_state.selected_tab {
                    TabSubjects::Components => {
                        // Load the selected component into the text areas
                        let selected_index = work_state.list_state.selected().unwrap_or(0);
                        if selected_index < work_state.current_selection_list.len() {
                            let selected_item = &work_state.current_selection_list[selected_index];
                            let component =
                                get_component_by_name(db_conn, selected_item.to_string());
                            if let Ok(component) = component {
                                work_state.component_text_areas.load_component(&component);
                            }
                        }
                        true
                    }
                    _ => false, // No edit functionality for other tabs yet
                }
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
    };
    // TODO maybe this handles the tabs entries and the switch between the work panes.
    // TODO then call handle_key function for the correct pane.
    event_handled
}

// To convert the crossterm keycode to tui_textarea::Input(via ratatui Key)
fn keyevent_to_input(key_event: KeyEvent) -> Input {
    let key = match key_event.code {
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Tab => Key::Tab,
        // TODO Fix this KeyCode::BackTab => Key::BackTab,
        KeyCode::Delete => Key::Delete,
        // TODO Fix this KeyCode::Insert => Key::Insert,
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Esc => Key::Esc,
        KeyCode::F(n) => Key::F(n),
        _ => Key::Null, // fallback for unmapped codes
    };

    Input {
        key,
        ctrl: key_event.modifiers.contains(KeyModifiers::CONTROL),
        alt: key_event.modifiers.contains(KeyModifiers::ALT),
        shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
    }
}

// TODO I think I need to create all at the beginning, and then set and retrieve the values from it.
// And I think I need to create a set for each tab, component, viewpacket etc.

// ----------------------------------------------------------------- Component text areas
// Originally generated by chatgpt
#[derive(Debug, Clone)]
pub struct ComponentTextAreas {
    pub id: TextArea<'static>,
    pub name: TextArea<'static>,
    pub purpose: TextArea<'static>,
    pub summary: TextArea<'static>,
    active_field: ComponentField,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentField {
    Id,
    Name,
    Purpose,
    Summary,
}

impl ComponentField {
    pub fn next(self) -> Self {
        match self {
            ComponentField::Id => ComponentField::Name,
            ComponentField::Name => ComponentField::Purpose,
            ComponentField::Purpose => ComponentField::Summary,
            ComponentField::Summary => ComponentField::Id,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            ComponentField::Id => ComponentField::Summary,
            ComponentField::Name => ComponentField::Id,
            ComponentField::Purpose => ComponentField::Name,
            ComponentField::Summary => ComponentField::Purpose,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ComponentField::Id => "ID:",
            ComponentField::Name => "Name:",
            ComponentField::Purpose => "Purpose:",
            ComponentField::Summary => "Summary:",
        }
    }
}

impl ComponentTextAreas {
    pub fn new() -> Self {
        let mut id = TextArea::default();
        id.set_placeholder_text("Component ID (read-only)");
        id.set_style(Style::default().fg(Color::Gray)); // Read-only styling

        let mut name = TextArea::default();
        name.set_placeholder_text("Enter component name");
        name.set_style(Style::default().fg(Color::White));

        let mut purpose = TextArea::default();
        purpose.set_placeholder_text("Enter component purpose");
        purpose.set_style(Style::default().fg(Color::White));

        let mut summary = TextArea::default();
        summary.set_placeholder_text("Enter component summary");
        summary.set_style(Style::default().fg(Color::White));

        Self {
            id,
            name,
            purpose,
            summary,
            active_field: ComponentField::Name, // Start with name (skip read-only ID)
        }
    }

    pub fn get_active_textarea(&mut self) -> &mut TextArea<'static> {
        match self.active_field {
            ComponentField::Id => &mut self.id,
            ComponentField::Name => &mut self.name,
            ComponentField::Purpose => &mut self.purpose,
            ComponentField::Summary => &mut self.summary,
        }
    }

    pub fn get_textarea(&mut self, field: ComponentField) -> &mut TextArea<'static> {
        match field {
            ComponentField::Id => &mut self.id,
            ComponentField::Name => &mut self.name,
            ComponentField::Purpose => &mut self.purpose,
            ComponentField::Summary => &mut self.summary,
        }
    }

    pub fn active_field(&self) -> ComponentField {
        self.active_field
    }

    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next();
        // Skip ID field in forward direction (it's read-only)
        if self.active_field == ComponentField::Id {
            self.active_field = ComponentField::Name;
        }
    }

    pub fn previous_field(&mut self) {
        self.active_field = self.active_field.previous();
        // Skip ID field in backward direction (it's read-only)
        if self.active_field == ComponentField::Id {
            self.active_field = ComponentField::Summary;
        }
    }

    pub fn set_field(&mut self, field: ComponentField) {
        // Don't allow setting focus to read-only ID field
        if field != ComponentField::Id {
            self.active_field = field;
        }
    }

    // Load component data into text areas
    pub fn load_component(&mut self, component: &Component) {
        self.id.delete_line_by_head();
        self.id.insert_str(component.id.to_string());

        self.name.delete_line_by_head();
        self.name.insert_str(&component.name);

        self.purpose.delete_line_by_head();
        self.purpose.insert_str(&component.purpose);

        self.summary.delete_line_by_head();
        self.summary.insert_str(&component.summary);
    }

    // Extract component data from text areas
    pub fn extract_component(&self) -> Component {
        Component {
            id: self.id.lines().join("").parse::<i32>().unwrap_or(0), // TODO panic if fail?
            name: self.name.lines().join("\n"),
            purpose: self.purpose.lines().join("\n"),
            summary: self.summary.lines().join("\n"),
        }
    }

    // Handle input for the active text area
    pub fn handle_input(&mut self, input: tui_textarea::Input) -> bool {
        // Don't allow editing the ID field
        if self.active_field == ComponentField::Id {
            return false;
        }

        let field_modified = self.get_active_textarea().input(input);
        field_modified
    }

    // Update visual styling based on active field
    pub fn update_styling(&mut self) {
        // Reset all to inactive style
        let inactive_style = Style::default().fg(Color::White);
        let active_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let readonly_style = Style::default().fg(Color::Gray);

        self.id.set_style(readonly_style);
        self.name
            .set_style(if self.active_field == ComponentField::Name {
                active_style
            } else {
                inactive_style
            });
        self.purpose
            .set_style(if self.active_field == ComponentField::Purpose {
                active_style
            } else {
                inactive_style
            });
        self.summary
            .set_style(if self.active_field == ComponentField::Summary {
                active_style
            } else {
                inactive_style
            });

        // Update block borders
        let active_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        let inactive_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));
        let readonly_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        self.id
            .set_block(readonly_block.clone().title("ID (read-only)"));
        self.name
            .set_block(if self.active_field == ComponentField::Name {
                active_block.clone().title("Name")
            } else {
                inactive_block.clone().title("Name")
            });
        self.purpose
            .set_block(if self.active_field == ComponentField::Purpose {
                active_block.clone().title("Purpose")
            } else {
                inactive_block.clone().title("Purpose")
            });
        self.summary
            .set_block(if self.active_field == ComponentField::Summary {
                active_block.clone().title("Summary")
            } else {
                inactive_block.clone().title("Summary")
            });
    }
}
