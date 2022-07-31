use super::JavascriptPlayer;
use rfd::{AsyncFileDialog, FileHandle};
use ruffle_core::backend::ui::{
    DialogResultFuture, Error, FileDialogResult, FileFilter, LoaderError, MouseCursor, UiBackend,
};
use ruffle_web_common::JsResult;
use std::path::Path;
use web_sys::HtmlCanvasElement;

use chrono::{DateTime, Utc};

#[cfg(target_arch = "wasm32")]
use chrono::NaiveDateTime;

#[derive(Debug)]
struct FullScreenError {
    jsval: String,
}

impl std::fmt::Display for FullScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.jsval)
    }
}

impl std::error::Error for FullScreenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct WebFileDialogResult {
    handle: Option<FileHandle>,
    contents: Vec<u8>,
}

impl WebFileDialogResult {
    pub async fn new(handle: Option<FileHandle>) -> Self {
        let contents = if let Some(handle) = handle.as_ref() {
            handle.read().await
        } else {
            Vec::new()
        };

        Self { handle, contents }
    }
}

fn get_extension_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| ".".to_owned() + x)
}

impl FileDialogResult for WebFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.handle.is_none()
    }

    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn creation_time(&self) -> Option<DateTime<Utc>> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn creation_time(&self) -> Option<DateTime<Utc>> {
        // Creation time is not available in JS
        None
    }

    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        if let Some(handle) = &self.handle {
            Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(handle.inner().last_modified() as i64, 0),
                Utc,
            ))
        } else {
            None
        }
    }

    fn file_name(&self) -> Option<String> {
        self.handle.as_ref().map(|handle| handle.file_name())
    }

    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn size(&self) -> Option<u64> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn size(&self) -> Option<u64> {
        self.handle.as_ref().map(|x| x.inner().size() as u64)
    }

    fn file_type(&self) -> Option<String> {
        if let Some(handle) = &self.handle {
            get_extension_from_filename(&handle.file_name())
        } else {
            None
        }
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write(&self, _data: &[u8]) {
        //NOOP
    }

    fn refresh(&mut self) {}
}

/// An implementation of `UiBackend` utilizing `web_sys` bindings to input APIs.
pub struct WebUiBackend {
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    cursor: MouseCursor,
    /// Is a dialog currently open
    dialog_open: bool,
}

impl WebUiBackend {
    pub fn new(js_player: JavascriptPlayer, canvas: &HtmlCanvasElement) -> Self {
        Self {
            js_player,
            canvas: canvas.clone(),
            cursor_visible: true,
            cursor: MouseCursor::Arrow,
            dialog_open: false,
        }
    }

    fn update_mouse_cursor(&self) {
        let cursor = if self.cursor_visible {
            match self.cursor {
                MouseCursor::Arrow => "auto",
                MouseCursor::Hand => "pointer",
                MouseCursor::IBeam => "text",
                MouseCursor::Grab => "grab",
            }
        } else {
            "none"
        };
        self.canvas
            .style()
            .set_property("cursor", cursor)
            .warn_on_error();
    }
}

impl UiBackend for WebUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        self.update_mouse_cursor();
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
        self.update_mouse_cursor();
    }

    fn set_clipboard_content(&mut self, _content: String) {
        log::warn!("set clipboard not implemented");
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error> {
        match self.js_player.set_fullscreen(is_full) {
            Ok(_) => Ok(()),
            Err(jsval) => Err(Box::new(FullScreenError {
                jsval: jsval
                    .as_string()
                    .unwrap_or_else(|| "Failed to change full screen state".to_string()),
            })),
        }
    }

    fn display_unsupported_message(&self) {
        self.js_player.display_unsupported_message()
    }

    fn display_root_movie_download_failed_message(&self) {
        self.js_player.display_root_movie_download_failed_message()
    }

    fn message(&self, message: &str) {
        self.js_player.display_message(message);
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            let mut dialog = AsyncFileDialog::new();

            for filter in filters {
                if std::env::consts::OS == "macos" && filter.mac_type.is_some() {
                    let mac_type = filter.mac_type.unwrap();
                    let extensions: Vec<&str> = mac_type.split(';').collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                } else {
                    let extensions: Vec<&str> = filter
                        .extensions
                        .split(';')
                        .map(|x| x.trim_start_matches("*."))
                        .collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                }
            }

            let result: Result<Box<dyn FileDialogResult>, LoaderError> = Ok(Box::new(
                WebFileDialogResult::new(dialog.pick_file().await).await,
            ));
            result
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }

    fn display_file_save_dialog(
        &mut self,
        _file_name: String,
        _domain: String,
    ) -> Option<DialogResultFuture> {
        None
    }
}
