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
    dump_db_to_xml, get_component_by_name, get_vector_of_component_names_sorted,
    get_vector_of_viewpacket_titles_sorted, get_viewpacket_by_title, update_component_by_id
};


use super::app_state::AppState;

mod work_state;
use work_state::{TabSubjects, WorkPane};
// This is to make it available in main.rs
pub use work_state::WorkState;

mod work_component_render;
use work_component_render::{
    render_component_details_pane,
    render_component_editor_pane,
};





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
                        dump_db_to_xml(&db_conn, "tui_out.xml");
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
