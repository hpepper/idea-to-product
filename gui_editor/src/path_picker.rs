//! Runs the open-file dialog on a worker thread and keeps the chosen path.
//! The portal dialog blocks until the user answers,
//! so running it on the UI thread would freeze the window (GEDT-STORY-00019).
//! T-02 only records the path; reading or parsing the file belongs to later tasks.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

#[derive(Default)]
pub struct PathPicker {
    pending: Option<Receiver<Option<PathBuf>>>,
    chosen: Option<PathBuf>,
}

impl PathPicker {
    /// Starts `pick` on a worker thread, unless a dialog is already open.
    /// Returns false when a dialog is already open, so one click opens one dialog.
    /// `pick` returns `None` when the user cancels.
    pub fn start(&mut self, pick: impl FnOnce() -> Option<PathBuf> + Send + 'static) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            // The receiver is gone only when the picker was dropped;
            // nobody needs the path then.
            let _ = sender.send(pick());
        });
        self.pending = Some(receiver);
        true
    }

    /// Takes the dialog result when it is ready, without blocking.
    /// A cancelled dialog keeps the previous path.
    /// A worker that ends without a result, for example by panicking, counts as a cancel.
    pub fn poll(&mut self) {
        let Some(receiver) = &self.pending else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                if let Some(path) = result {
                    self.chosen = Some(path);
                }
                self.pending = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => self.pending = None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }

    pub fn chosen(&self) -> Option<&Path> {
        self.chosen.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    /// Polls until the dialog closes, so the tests do not depend on thread timing.
    fn poll_until_closed(picker: &mut PathPicker) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while picker.is_open() {
            assert!(Instant::now() < deadline, "dialog worker did not finish");
            picker.poll();
            thread::yield_now();
        }
    }

    #[test]
    fn test_story_00019_new_picker_has_no_path_and_no_dialog() {
        let picker = PathPicker::default();
        assert!(!picker.is_open());
        assert_eq!(picker.chosen(), None);
    }

    #[test]
    fn test_story_00019_chosen_path_is_kept() {
        let mut picker = PathPicker::default();
        assert!(picker.start(|| Some(PathBuf::from("/home/user/sad.xml"))));
        poll_until_closed(&mut picker);
        assert_eq!(picker.chosen(), Some(Path::new("/home/user/sad.xml")));
    }

    #[test]
    fn test_story_00019_cancel_keeps_previous_path() {
        let mut picker = PathPicker::default();
        picker.start(|| Some(PathBuf::from("/a.xml")));
        poll_until_closed(&mut picker);
        picker.start(|| None);
        poll_until_closed(&mut picker);
        assert_eq!(picker.chosen(), Some(Path::new("/a.xml")));
    }

    #[test]
    fn test_story_00019_second_start_while_open_is_ignored() {
        let mut picker = PathPicker::default();
        let release = Arc::new(Barrier::new(2));
        let worker_release = Arc::clone(&release);
        assert!(picker.start(move || {
            worker_release.wait();
            Some(PathBuf::from("/first.xml"))
        }));
        picker.poll();
        assert!(picker.is_open(), "dialog closed before the user answered");
        assert!(!picker.start(|| Some(PathBuf::from("/second.xml"))));
        release.wait();
        poll_until_closed(&mut picker);
        assert_eq!(picker.chosen(), Some(Path::new("/first.xml")));
    }

    #[test]
    fn test_story_00019_panicking_dialog_counts_as_cancel() {
        let mut picker = PathPicker::default();
        picker.start(|| Some(PathBuf::from("/a.xml")));
        poll_until_closed(&mut picker);
        picker.start(|| panic!("portal failure"));
        poll_until_closed(&mut picker);
        assert_eq!(picker.chosen(), Some(Path::new("/a.xml")));
        assert!(picker.start(|| None), "picker stays usable after a failure");
    }
}
