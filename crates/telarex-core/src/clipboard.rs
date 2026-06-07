/// System clipboard integration.
///
/// Provides `copy` and `paste` operations backed by `arboard` when the
/// `clipboard` feature is enabled.  Falls back to stub functions that
/// return an error otherwise.
#[cfg(feature = "clipboard")]
mod inner {
    use std::sync::Mutex;
    use std::sync::OnceLock;

    struct ClipboardState {
        inner: Mutex<Option<arboard::Clipboard>>,
    }

    fn state() -> &'static ClipboardState {
        static STATE: OnceLock<ClipboardState> = OnceLock::new();
        STATE.get_or_init(|| {
            ClipboardState {
                inner: Mutex::new(arboard::Clipboard::new().ok()),
            }
        })
    }

    /// Copy text to the system clipboard.
    pub fn copy(text: &str) -> Result<(), String> {
        let s = state();
        let mut guard = s.inner.lock().map_err(|e| e.to_string())?;
        match &mut *guard {
            Some(cb) => cb.set_text(text).map_err(|e| e.to_string()),
            None => Err("Clipboard unavailable".to_string()),
        }
    }

    pub fn paste() -> Result<String, String> {
        let s = state();
        let mut guard = s.inner.lock().map_err(|e| e.to_string())?;
        match &mut *guard {
            Some(cb) => cb.get_text().map_err(|e| e.to_string()),
            None => Err("Clipboard unavailable".to_string()),
        }
    }
}

#[cfg(not(feature = "clipboard"))]
mod inner {
    /// Copy text to the system clipboard.
    pub fn copy(_text: &str) -> Result<(), String> {
        Err("Clipboard support not enabled".to_string())
    }

    /// Paste text from the system clipboard.
    pub fn paste() -> Result<String, String> {
        Err("Clipboard support not enabled".to_string())
    }
}

pub use inner::*;
