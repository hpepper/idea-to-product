use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use rusqlite::Connection;
use tui_textarea::TextArea;

use super::work_state::WorkState;
use sad_xml_sql::{get_viewpacket_by_title, ViewPacket};

pub fn render_viewpacket_details_pane(
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

/*
  prompt to gpt-4.1 for generating the code:

  as a experienced rust and ratatui developer please show how to create an editor pan for viewpackets
where title and introduction are editable textareas
and id, primary_display_key and context_model_key are static text
view_type is a drop-down containing: Module, ViewPacketsAndConnectors and Allocation
please use tab and shift+tab to move between the non static fields
 */

// TODO how do I handle when the field is a drop down, then it has to be handled differently, should I have a control function for viewpacket that will handle sending the input to the right field and do the right thimmg like send to textarea or browse in list or pop-up editing a context diagram?

pub fn render_viewpacket_editor_pane(frame: &mut Frame, work_state: &mut WorkState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // id
            Constraint::Length(3), // title
            Constraint::Length(5), // introduction
            Constraint::Length(3), // view_type
            Constraint::Length(3), // view style
            Constraint::Length(2), // primary_display_key
            Constraint::Length(2), // context_model_key
        ])
        .split(area);

    // Static fields
    render_detail_row_static_text(
        frame,
        chunks[0],
        20, // label column width
        "ViewPacket ID:",
        &work_state.viewpacket_editor_fields.viewpacket_id,
    );
    render_row_detail_textarea(
        frame,
        chunks[1],
        20,
        "Title:",
        &mut work_state.viewpacket_editor_fields.title,
    );
    render_row_detail_textarea(
        frame,
        chunks[2],
        20,
        "Introduction:",
        &mut work_state.viewpacket_editor_fields.introduction,
    );
    render_detail_row_static_text(
        frame,
        chunks[5],
        20, // label column width
        "Primary display key:",
        &work_state.viewpacket_editor_fields.primary_display_key,
    );
    render_detail_row_static_text(
        frame,
        chunks[6],
        20, // label column width
        "Context Model Key:",
        &work_state.viewpacket_editor_fields.context_model_key,
    );

    if work_state.viewpacket_editor_fields.field_in_focus != ViewPacketEditorFocusField::ViewStyleDropdown {
        render_detail_row_static_text(
            frame,
            chunks[3],
            20, // label column width
            "View Type:",
            work_state.viewpacket_editor_fields.view_type.as_str(),
        );
    } else {
        // Render dropdown for view type
        let view_types: Vec<ListItem> = ViewType::all()
            .iter()
            .map(|vt| ListItem::new(vt.as_str()))
            .collect();
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(work_state.viewpacket_editor_fields.view_type_selected));
        frame.render_stateful_widget(
            List::new(view_types).block(
                Block::default()
                    .title("View Type")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            ),
            chunks[3],
            &mut list_state,
        );
    }    // ViewType dropdown
    let view_types: Vec<ListItem> = ViewType::all()
        .iter()
        .map(|vt| ListItem::new(vt.as_str()))
        .collect();

    let view_type_block = Block::default()
        .title("View Type")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // TODO implement this for ViewPackets
    // if state.view_type_dropdown_open {
    //     let mut list_state = ratatui::widgets::ListState::default();
    //     list_state.select(Some(work_state.viewpacket_editor_fields.view_type_selected));
    //     frame.render_stateful_widget(
    //         List::new(view_types).block(view_type_block),
    //         chunks[3],
    //         &mut list_state,
    //     );
    // } else {
    //     frame.render_widget(
    //         Paragraph::new(Span::raw(format!(
    //             "View Type: {}",
    //             state.view_type.as_str()
    //         )))
    //         .block(view_type_block),
    //         chunks[3],
    //     );
    // }

    // // Editable textareas
    // frame.render_widget(state.title.widget(), chunks[4]);
    // frame.render_widget(state.introduction.widget(), chunks[5]);
}

fn render_detail_row_static_text(
    frame: &mut Frame,
    area: Rect,
    label_column_width: u16,
    label: &str,
    static_text: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(label_column_width), // label column
            Constraint::Min(10),                    // textarea column
        ])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(Color::Yellow));
    frame.render_widget(label_widget, chunks[0]);

    frame.render_widget(static_text, chunks[1]);
}

fn render_row_detail_textarea(
    frame: &mut Frame,
    area: Rect,
    label_column_width: u16,
    label: &str,
    textarea: &TextArea<'_>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(label_column_width), // label column
            Constraint::Min(10),    // textarea column
        ])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(Color::Yellow));
    frame.render_widget(label_widget, chunks[0]);

    frame.render_widget(textarea, chunks[1]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewType {
    Module,
    ViewPacketsAndConnectors,
    Allocation,
}

impl ViewType {
    pub fn all() -> &'static [ViewType] {
        &[
            ViewType::Module,
            ViewType::ViewPacketsAndConnectors,
            ViewType::Allocation,
        ]
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewType::Module => "Module",
            ViewType::ViewPacketsAndConnectors => "ViewPacketsAndConnectors",
            ViewType::Allocation => "Allocation",
        }
    }
}

/**
 * Handle switching/traversing between field in the right order, with tab/shift+tab
 * Holds the label for each field.
 *
 * @see ViewPacketEditorFieldData for the actual data manipulation.
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewPacketEditorFocusField {
    Title,
    Introduction,
    ViewTypeDropdown,
    ViewStyleDropdown,
    SortOrder,
    PrimaryDisplayKey,
    ContextModelKey,
}

impl ViewPacketEditorFocusField {
    // Get the next field in the order listed. (for use with tab for next)
    pub fn next(self) -> Self {
        match self {
            ViewPacketEditorFocusField::Title => ViewPacketEditorFocusField::Introduction,
            ViewPacketEditorFocusField::Introduction => {
                ViewPacketEditorFocusField::ViewTypeDropdown
            }
            ViewPacketEditorFocusField::ViewTypeDropdown => {
                ViewPacketEditorFocusField::ViewStyleDropdown
            }
            ViewPacketEditorFocusField::ViewStyleDropdown => ViewPacketEditorFocusField::SortOrder,
            ViewPacketEditorFocusField::SortOrder => ViewPacketEditorFocusField::PrimaryDisplayKey,
            ViewPacketEditorFocusField::PrimaryDisplayKey => {
                ViewPacketEditorFocusField::ContextModelKey
            }
            ViewPacketEditorFocusField::ContextModelKey => ViewPacketEditorFocusField::Title,
        }
    }

    // Get the previous field in the order listed. (for use with shift + tab for previous)
    pub fn previous(self) -> Self {
        match self {
            ViewPacketEditorFocusField::Title => ViewPacketEditorFocusField::ContextModelKey,
            ViewPacketEditorFocusField::ContextModelKey => {
                ViewPacketEditorFocusField::PrimaryDisplayKey
            }
            ViewPacketEditorFocusField::PrimaryDisplayKey => ViewPacketEditorFocusField::SortOrder,
            ViewPacketEditorFocusField::SortOrder => ViewPacketEditorFocusField::ViewStyleDropdown,
            ViewPacketEditorFocusField::ViewStyleDropdown => {
                ViewPacketEditorFocusField::ViewTypeDropdown
            }
            ViewPacketEditorFocusField::ViewTypeDropdown => {
                ViewPacketEditorFocusField::Introduction
            }
            ViewPacketEditorFocusField::Introduction => ViewPacketEditorFocusField::Title,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewPacketEditorFocusField::Title => "Title:",
            ViewPacketEditorFocusField::Introduction => "Introduction:",
            ViewPacketEditorFocusField::ViewTypeDropdown => "View Type:",
            ViewPacketEditorFocusField::ViewStyleDropdown => "View Style:",
            ViewPacketEditorFocusField::SortOrder => "Sort Order:",
            ViewPacketEditorFocusField::PrimaryDisplayKey => "Primary Display Key:",
            ViewPacketEditorFocusField::ContextModelKey => "Context Model Key:",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ViewPacketEditorFieldData {
    pub viewpacket_id: String,
    pub title: TextArea<'static>,
    pub introduction: TextArea<'static>,
    pub primary_display_key: String,
    pub context_model_key: String,
    pub view_type: ViewType,
    pub view_type_selected: usize,
    pub view_type_dropdown_open: bool,
    pub field_in_focus: ViewPacketEditorFocusField,
    // TODO add component id, is the edit also going to be a drop down list?
}

impl ViewPacketEditorFieldData {
    pub fn new() -> Self {
        let mut title = TextArea::default();
        title.set_block(Block::default().title("Title").borders(Borders::ALL));
        title.set_style(Style::default().fg(Color::White));
        let mut introduction = TextArea::default();
        introduction.set_block(Block::default().title("Introduction").borders(Borders::ALL));
        introduction.set_style(Style::default().fg(Color::White));
        Self {
            title,
            introduction,
            viewpacket_id: String::new(),
            primary_display_key: String::new(),
            context_model_key: String::new(),
            view_type: ViewType::Module,
            view_type_selected: 0,
            view_type_dropdown_open: false,
            field_in_focus: ViewPacketEditorFocusField::Title,
        }
    }

    // TODO I need to change this to a Result, since the field in focus might not be a a text area
    pub fn get_field_in_focus(&mut self) -> Option<&mut TextArea<'static>> {
        self.get_textarea_field(self.field_in_focus)
    }

    pub fn get_textarea_field(
        &mut self,
        field: ViewPacketEditorFocusField,
    ) -> Option<&mut TextArea<'static>> {
        match field {
            ViewPacketEditorFocusField::Title => Some(&mut self.title),
            ViewPacketEditorFocusField::Introduction => Some(&mut self.introduction),
            _ => None,
        }
    }

    pub fn field_in_focus(&self) -> ViewPacketEditorFocusField {
        self.field_in_focus
    }

    pub fn next_field(&mut self) {
        self.field_in_focus = self.field_in_focus.next();
        // Skip ID field in forward direction (it's read-only)
    }

    pub fn previous_field(&mut self) {
        self.field_in_focus = self.field_in_focus.previous();
    }

    pub fn set_field(&mut self, field: ViewPacketEditorFocusField) {
        // Don't allow setting focus to read-only ID field
        self.field_in_focus = field;
    }

    // Load ViewPacket data into text areas
    pub fn load_viewpacket(&mut self, viewpacket: &ViewPacket) {
        // self.id.delete_line_by_head();
        // self.id.insert_str(viewpacket.id.to_string());
        self.viewpacket_id = viewpacket.viewpacket_id.to_string();

        // TODO refactor? what does delete_line_by_head() do?
        self.title.delete_line_by_head();
        self.title.insert_str(&viewpacket.title);

        self.introduction.delete_line_by_head();
        self.introduction.insert_str(&viewpacket.introduction);

        self.primary_display_key = viewpacket.primary_display_key.clone();
        self.context_model_key = viewpacket.context_model_key.clone();

        // TODO handle the dropdowns and the keys as well
    }

    // Extract ViewPacket data from text areas
    pub fn extract_viewpacket(&self) -> ViewPacket {
        ViewPacket {
            viewpacket_id: self.viewpacket_id.parse::<u64>().unwrap_or(0), // TODO panic if fail?
            title: self.title.lines().join("\n"),
            introduction: self.introduction.lines().join("\n"),
            // TODO handle the dropdowns and the keys as well
            primary_display_key: self.primary_display_key.clone(),
            context_model_key: self.context_model_key.clone(),
            view_type: self.view_type.as_str().to_string(),
            sort_order: 0, // TODO handle sort order
            component_id: 0,
            view_style: "TODO IMPLEMENT".to_string(),
            file_id: 0, // TODO why is this here???
            team_id: 0, // TODO why is this here?
        }
    }

    // Handle input for the active text area
    // pub fn handle_input(&mut self, input: tui_textarea::Input) -> bool {
    //     // Don't allow editing the ID field
    //     if self.field_in_focus == ViewPacketEditorFocusField::Id {
    //         return false;
    //     }

    //     let field_modified = self.get_active_textarea().input(input); // TODO get the text area if Some then handle the input.
    //     field_modified
    // }

    // Update visual styling based on active field
    // TODO change to handle ViewPackets
    // pub fn update_styling(&mut self) {
    //     // Reset all to inactive style
    //     let inactive_style = Style::default().fg(Color::White);
    //     let active_style = Style::default()
    //         .fg(Color::Yellow)
    //         .add_modifier(Modifier::BOLD);
    //     let readonly_style = Style::default().fg(Color::Gray);

    //     self.id.set_style(readonly_style);
    //     self.name
    //         .set_style(if self.field_in_focus == ViewPacketEditorFocusField::Name {
    //             active_style
    //         } else {
    //             inactive_style
    //         });
    //     self.purpose.set_style(
    //         if self.field_in_focus == ViewPacketEditorFocusField::Purpose {
    //             active_style
    //         } else {
    //             inactive_style
    //         },
    //     );
    //     self.summary.set_style(
    //         if self.field_in_focus == ViewPacketEditorFocusField::Summary {
    //             active_style
    //         } else {
    //             inactive_style
    //         },
    //     );

    //     // Update block borders
    //     let active_block = Block::default()
    //         .borders(Borders::ALL)
    //         .border_style(Style::default().fg(Color::Blue));
    //     let inactive_block = Block::default()
    //         .borders(Borders::ALL)
    //         .border_style(Style::default().fg(Color::Gray));
    //     let readonly_block = Block::default()
    //         .borders(Borders::ALL)
    //         .border_style(Style::default().fg(Color::DarkGray));

    //     self.id
    //         .set_block(readonly_block.clone().title("ID (read-only)"));
    //     self.name
    //         .set_block(if self.field_in_focus == ViewPacketEditorFocusField::Name {
    //             active_block.clone().title("Name")
    //         } else {
    //             inactive_block.clone().title("Name")
    //         });
    //     self.purpose.set_block(
    //         if self.field_in_focus == ViewPacketEditorFocusField::Purpose {
    //             active_block.clone().title("Purpose")
    //         } else {
    //             inactive_block.clone().title("Purpose")
    //         },
    //     );
    //     self.summary.set_block(
    //         if self.field_in_focus == ViewPacketEditorFocusField::Summary {
    //             active_block.clone().title("Summary")
    //         } else {
    //             inactive_block.clone().title("Summary")
    //         },
    //     );
    // }
}
