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
    pub fn copy(_text: &str) -> Result<(), String> {
        Err("Clipboard support not enabled".to_string())
    }

    pub fn paste() -> Result<String, String> {
        Err("Clipboard support not enabled".to_string())
    }
}

pub use inner::*;
