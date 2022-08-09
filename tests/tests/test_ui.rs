use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use ruffle_core::backend::ui::{
    DialogResultFuture, Error, FileDialogResult, FileFilter, LoaderError, MouseCursor, UiBackend,
};

#[derive(Default)]
pub struct TestFileDialogResult {
    canceled: bool,
    file_name: Option<String>,
}

impl TestFileDialogResult {
    fn new_canceled() -> Self {
        Self {
            canceled: true,
            file_name: None,
        }
    }

    fn new_success(file_name: String, file_size: u64) -> Self {
        Self {
            canceled: false,
            file_name: Some(file_name),
        }
    }
}

impl FileDialogResult for TestFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.canceled
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        /*
        let d: DateTime<Utc> = Utc.datetime_from_str("2022-08-07T21:32:58", "%Y-%m-%dT%H:%M:%S").unwrap();
        let d = d.with_timezone(&FixedOffset::east(60 * 60));

        (!self.is_cancelled()).then(|| d)*/
        None
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn file_name(&self) -> Option<String> {
        (!self.is_cancelled()).then(|| self.file_name.clone().unwrap())
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn file_type(&self) -> Option<String> {
        (!self.is_cancelled()).then(|| ".txt".to_string())
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &[]
    }

    fn write(&self, data: &[u8]) {}

    fn refresh(&mut self) {}
}

#[derive(Default)]
pub struct TestUiBackend;

impl UiBackend for TestUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error> {
        Ok(())
    }

    fn display_unsupported_message(&self) {}

    fn display_root_movie_download_failed_message(&self) {}

    fn message(&self, message: &str) {}

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If filters has the magic debug-select-success filter, then return a fake file for testing

            let result: Result<Box<dyn FileDialogResult>, LoaderError> = if filters
                .iter()
                .any(|f| f.description == "debug-select-success")
            {
                Ok(Box::new(TestFileDialogResult::new_success(
                    "test.txt".to_string(),
                    13,
                )))
            } else {
                Ok(Box::new(TestFileDialogResult::new_canceled()))
            };

            result
        }))
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        _title: String,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            // If file_name has the magic debug-success.txt value, then return a fake file for testing

            let result: Result<Box<dyn FileDialogResult>, LoaderError> =
                if file_name == "debug-success.txt" {
                    Ok(Box::new(TestFileDialogResult::new_success(file_name, 1256)))
                } else {
                    Ok(Box::new(TestFileDialogResult::new_canceled()))
                };

            result
        }))
    }

    fn close_file_dialog(&mut self) {}
}
