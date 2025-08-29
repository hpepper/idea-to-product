use ratatui::{style::{Color, Style}, widgets::ListState};
use crate::work::work_component_render::ComponentTextAreas;
use crate::work::work_viewpacket_render::ViewPacketEditorFieldData;
use tui_textarea::{Input, Key, TextArea};

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