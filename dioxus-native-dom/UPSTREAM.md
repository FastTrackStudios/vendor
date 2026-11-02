Source: DioxusLabs/dioxus, revision 430cfb563f1b70602ef387f0c80f171d84527b41,
packages/native-dom (0.8.0-alpha.3). Workspace dependency declarations are
expanded without changing their versions. License: MIT OR Apache-2.0.

FTS changes, carried across upgrades:

- `text_input_focused()` / `on_text_focus_change(f)` (`src/text_focus.rs`):
  each document counts itself in while its focused node is a text input,
  re-checked after every poll and UI event, so an app's global hotkeys can
  stand aside while text is typed — in every window, plugin editor and
  embedded panel. Not upstream.
- `set_focus` parks a focus change while the document is borrowed
  (`PENDING_FOCUS`, `apply_pending_focus`), applied at the next poll or UI
  event.
- `get_scroll_offset`, `get_scroll_size` and `get_client_rect` use
  `try_borrow`, so a handle used while the document is busy reports an error
  instead of panicking.
- IME composition (`composition_event`, `NativeCompositionData`): Blitz's
  `Ime` event reaches Dioxus as `compositionupdate` (the preedit; empty when
  cleared) and `compositionend` (the committed text) instead of being
  dropped, so a focusable element that does its own editing (with the FTS
  Blitz fork's `inputmode` IME opt-in) can compose. Not upstream.
- Queued mounted events skip an element removed before delivery
  (`try_element_to_node_id`): upstream now clears a removed element's
  mapping, so its `element_to_node_id` would panic there.

Dropped at this rev: the ElementId→NodeId mapping cleanup for removed
subtrees (`forget_subtree`, `node_to_element`). Upstream rewrote the mutation
writer as a stack machine over buffered mutations (#5911, #5943): there is no
`assign_node_id` cleanup any more to delete a fresh template clone that
reuses a removed node's slab id, which is what the fix guarded.
