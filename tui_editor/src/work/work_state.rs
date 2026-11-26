use ratatui::{style::{Color, Style}, widgets::ListState};
use crate::work::work_component_render::ComponentTextAreas;
use crate::work::work_viewpacket_render::ViewPacketEditorFieldData;
use ratatui_textarea::{Input, Key, TextArea};

pub struct WorkState {
    pub list_state: ListState,
    pub active_pane: WorkPane,
    pub previous_active_pane: WorkPane,
    pub textarea: TextArea<'static>, // TODO delete this, I don't think we need it here
    pub tab_counts: TabCounts,
    pub selected_tab: TabSubjects,
    pub current_selection_list: Vec<String>,
    pub number_of_cloned_objects: usize,
    // Text area collections for each tab type
    pub component_text_areas: ComponentTextAreas,
    pub viewpacket_editor_fields: ViewPacketEditorFieldData,
    // pub diagram_text_areas: DiagramTextAreas,       // Future
    /// The file F7 saves the project to, relative to the working directory.
    /// Only tests change the default, to keep their save out of the crate root.
    pub save_path: String,
    /// The error of the last failed F7 save,
    /// shown in the status line until a save succeeds.
    /// The edits stay in the in-memory database,
    /// so the next F7 saves them (GEDT-FR-00048).
    pub save_error: Option<String>,
    /// Whether the session database holds edits that no save has written yet.
    /// A failed save leaves the flag on (GEDT-FR-00048).
    pub project_modified: bool,
    /// Whether the editor pane holds text changes that F7 has not applied yet.
    /// Esc discards those changes, so Esc clears this flag but not `project_modified`.
    pub pending_edit: bool,
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
            viewpacket_editor_fields: ViewPacketEditorFieldData::new(),
            save_path: "tui_out.xml".to_string(),
            save_error: None,
            project_modified: false,
            pending_edit: false,
        }
    }

    /// Whether the project has changes that no save has written,
    /// in the session database or in the editor pane.
    pub fn is_modified(&self) -> bool {
        self.project_modified || self.pending_edit
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
pub enum TabSubjects {
    Components,
    ViewPackets,
    Diagrams,
}

impl TabSubjects {
    pub fn as_index(&self) -> usize {
        match self {
            TabSubjects::Components => 0,
            TabSubjects::ViewPackets => 1,
            TabSubjects::Diagrams => 2,
        }
    }
}