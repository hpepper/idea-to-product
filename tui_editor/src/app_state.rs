pub struct AppState {
    pub status_message: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status_message: String::new(),
        }
    }
}