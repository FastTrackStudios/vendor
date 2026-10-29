//! FTS: whether a text input has the keyboard, process-wide.
//!
//! An app's global hotkeys (a held key that arranges panels, Space for the
//! transport) must stand aside while text is typed into a field. Every
//! focus change in a document passes through [`DioxusDocument::poll`] and
//! [`DioxusDocument::handle_ui_event`](crate::DioxusDocument): window
//! events, a task's `set_focus`, autofocus, a removed node. Each document
//! checks there whether its focused node is a text input (an `<input>` or
//! `<textarea>` Blitz edits text in) and counts itself in or out, so the
//! answer covers every window, plugin editor and embedded panel of the
//! process.
//!
//! [`text_input_focused`] reads it; [`on_text_focus_change`] registers one
//! function called whenever the process-wide answer flips (an app feeds
//! its own keyboard switch from it).
//!
//! [`DioxusDocument::poll`]: crate::DioxusDocument

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use blitz_dom::BaseDocument;

/// How many documents have a text input focused.
static FOCUSED: AtomicUsize = AtomicUsize::new(0);
/// Called when [`text_input_focused`] changes.
static ON_CHANGE: OnceLock<fn(bool)> = OnceLock::new();

/// True while a text input has the keyboard in any document of the
/// process.
#[must_use]
pub fn text_input_focused() -> bool {
    FOCUSED.load(Ordering::Relaxed) > 0
}

/// Registers `f`, called with the new value each time
/// [`text_input_focused`] changes. Set once, at startup; a second call is
/// ignored.
pub fn on_text_focus_change(f: fn(bool)) {
    let _ = ON_CHANGE.set(f);
}

/// One document's share of [`FOCUSED`].
#[derive(Debug, Default)]
pub(crate) struct TextFocus {
    held: bool,
}

impl TextFocus {
    /// Counts the document in or out by what has its focus now.
    pub(crate) fn sync(&mut self, doc: &BaseDocument) {
        // `get_focussed_node_id` falls back to the root element when nothing
        // is focused; the root is never a text input, so the fallback reads
        // as "not typing".
        let now = doc
            .get_focussed_node_id()
            .and_then(|id| doc.get_node(id))
            .and_then(|node| node.element_data())
            .is_some_and(|el| el.text_input_data().is_some());
        self.set(now);
    }

    fn set(&mut self, now: bool) {
        if now == self.held {
            return;
        }
        self.held = now;
        let before = if now {
            FOCUSED.fetch_add(1, Ordering::Relaxed)
        } else {
            FOCUSED.fetch_sub(1, Ordering::Relaxed)
        };
        // The process-wide answer flips when the count leaves or reaches 0.
        let flipped = if now { before == 0 } else { before == 1 };
        if flipped {
            if let Some(f) = ON_CHANGE.get() {
                f(now);
            }
        }
    }
}

impl Drop for TextFocus {
    /// A document closed while typing gives the keyboard back.
    fn drop(&mut self) {
        self.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::{TextFocus, text_input_focused};

    // One test: the count is process-wide.
    #[test]
    fn the_keyboard_is_held_until_every_typing_document_lets_go() {
        let (mut a, mut b) = (TextFocus::default(), TextFocus::default());
        assert!(!text_input_focused());
        a.set(true);
        a.set(true);
        b.set(true);
        assert!(text_input_focused());
        a.set(false);
        assert!(text_input_focused(), "b still types");
        drop(b);
        assert!(!text_input_focused(), "a closed document lets go");
    }
}
