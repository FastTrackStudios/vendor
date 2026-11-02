# dioxus-vdom (vendored)

From DioxusLabs/dioxus `packages/vdom` at rev
`430cfb563f1b70602ef387f0c80f171d84527b41` (0.8.0-alpha.3 line) — the rev
the fleet pins every other dioxus crate to. Upstream renamed the old
`dioxus-core` (the virtual DOM) to `dioxus-vdom` in #6021, and `dioxus-core`
is now the prelude crate built on it (#6024); this is the crate the old
`dioxus-core/` vendor dir carried. Patched in through a consumer's
`[patch."https://github.com/DioxusLabs/dioxus"] dioxus-vdom = …`.

Two changes.

**1. `src/events.rs`, `Callback::__point_to`:** when a component
re-renders, `rsx!`'s generated `memoize` points its old event handler at the
new one. Upstream unwraps that, and it fails whenever the old handler's
owner — the scope that created the closure — has already dropped while the
component still held it (an element built in one scope and rendered from
another: a popup's render function, a face remounted mid-update). The
result was `called Result::unwrap() on an Err value: Dropped(...)` on the
main thread, which on iOS aborts the app. The vendored copy logs a warning
and takes the new handler instead. Still an `unwrap()` upstream at
`430cfb56`.

**2. Nested suspense (FastTrackStudios/task#410, the core half of upstream
DioxusLabs/dioxus#5668, still open):** `Runtime::scope_should_render`
(`src/runtime.rs`) decides whether a scope may write to the renderer. Upstream
looks only at the scope's own suspense location, and a boundary's own
location names the boundary itself, so a `SuspenseBoundary` inside the
background of an outer boundary that is still suspended counted as visible.
When that inner boundary suspended again (its fallback swapped in), the swap
was written to the real renderer against nodes that only exist in the
background: a stray fallback in the page next to the outer fallback, then
edits on node ids the renderer never had. In a wasm-split web build every
route sits behind a boundary that suspends while its chunk loads, so a route
with its own nested loader hits this. The vendored copy records the location
each scope was created in (`Scope::enclosing_suspense_location`,
`src/scope_context.rs`, set in `src/scope_arena.rs`) and walks the whole
parent chain, refusing writes if any scope on it sits in a suspended
boundary's background. #5668's other two core changes are already in
`430cfb56` in another form (`SuspenseDriver::diff` gates its writer;
`SuspenseDriver::create` reuses the boundary's rendered output), and the
per-mount `RenderMode` covers the resolve case, so this is the only piece
ported. `tests/nested_suspense.rs` drives each shape through the upstream
renderer oracle; `inner_boundary_resuspends_while_outer_boundary_is_suspended`
and `nested_boundary_flips_after_navigating_to_a_suspending_page` fail
without the change. Drop it once upstream walks the chain.

`Cargo.toml` is the upstream one with the workspace-inherited dependencies
written out (dioxus siblings as git deps at the same rev; benches and upstream's
tests dropped). Its dev-dependencies and the `[patch]` to `.` exist only so
`cargo test` here runs the doctests and `tests/` against this copy. When
the pin moves, re-vendor from the new rev and carry both changes across — or
drop each once upstream has its own fix.
