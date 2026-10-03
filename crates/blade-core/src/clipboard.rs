use arboard::Clipboard;

pub struct SystemClipboard {
    clipboard: Option<Clipboard>,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
        }
    }

    pub fn get_text(&mut self) -> Option<String> {
        self.clipboard.as_mut()?.get_text().ok()
    }

    pub fn set_text(&mut self, text: String) {
        if let Some(cb) = self.clipboard.as_mut() {
            let _ = cb.set_text(text);
        }
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}
