Source: DioxusLabs/dioxus, revision f717a8e184a522d078b70bb4b4d62a5f9a99ddfc,
packages/native-dom (0.8.0-alpha.0). Workspace dependency declarations are expanded
without changing their versions. License: MIT OR Apache-2.0.

Session's combined workstation DOM stress exposed stale ElementId-to-NodeId
mappings after subtree removal. Blitz reuses numeric slab IDs. assign_node_id's
cleanup of an old unparented mapping could therefore delete a fresh template
clone, then panic in node_at_path. Clear mappings for removed/replaced subtrees
before immediately dropping the removed subtree, using a reverse map to avoid scanning the whole document.
Queued mount events skip elements removed before delivery.

Remove this patch after upgrading to an upstream version with equivalent mapping
lifetime guarantees. Regression coverage lives in Session's DOM stress harness
and the native mutation writer unit tests.

FTS also adds `text_input_focused()` / `on_text_focus_change(f)`
(`src/text_focus.rs`): each document counts itself in while its focused node
is a text input, re-checked after every poll and UI event, so an app's
global hotkeys can stand aside while text is typed — in every window, plugin
editor and embedded panel. Not upstream; carry it across upgrades.

Also carried and not listed above: `set_focus` parks a focus change while the
document is borrowed (`PENDING_FOCUS`, `apply_pending_focus`), and
`get_scroll_offset`, `get_scroll_size` and `get_client_rect` use `try_borrow`,
so a handle that outlives a busy document reports an error instead of
panicking.
