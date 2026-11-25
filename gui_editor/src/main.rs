//! gui_editor: native SAD editor for Fedora on Wayland and X11 (GEDT-STORY-00019).
//! T-02 walking skeleton (GEDT-AR-00003, GEDT-AR-00004):
//! an egui window with an open-file dialog and an empty component list.

mod path_picker;

use eframe::egui;
use path_picker::PathPicker;

const APP_TITLE: &str = "gui_editor";

#[derive(Default)]
struct EditorApp {
    picker: PathPicker,
    // Stays empty until a later task loads a project file.
    components: Vec<String>,
}

impl eframe::App for EditorApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.picker.poll();
        if self.picker.is_open() {
            // The worker thread does not hold the egui context,
            // so keep polling while the dialog is open.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let open = ui.add_enabled(!self.picker.is_open(), egui::Button::new("Open"));
                if open.clicked() {
                    // The portal dialog runs in another process over the session D-Bus.
                    self.picker.start(|| rfd::FileDialog::new().pick_file());
                }
                match self.picker.chosen() {
                    Some(path) => ui.label(path.display().to_string()),
                    None => ui.label("No file chosen"),
                };
            });
        });
        egui::Panel::left("components").show(ui, |ui| {
            ui.heading("Components");
            if self.components.is_empty() {
                ui.label("No components");
            }
            for component in &self.components {
                ui.label(component);
            }
        });
        egui::CentralPanel::default().show(ui, |_ui| {});
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        APP_TITLE,
        eframe::NativeOptions::default(),
        Box::new(|_creation_context| Ok(Box::<EditorApp>::default())),
    )
}
