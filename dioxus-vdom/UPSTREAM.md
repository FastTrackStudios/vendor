# dioxus-vdom (vendored)

From DioxusLabs/dioxus `packages/vdom` at rev
`430cfb563f1b70602ef387f0c80f171d84527b41` (0.8.0-alpha.3 line) — the rev
the fleet pins every other dioxus crate to. Upstream renamed the old
`dioxus-core` (the virtual DOM) to `dioxus-vdom` in #6021, and `dioxus-core`
is now the prelude crate built on it (#6024); this is the crate the old
`dioxus-core/` vendor dir carried. Patched in through a consumer's
`[patch."https://github.com/DioxusLabs/dioxus"] dioxus-vdom = …`.

The one change (`src/events.rs`, `Callback::__point_to`): when a component
re-renders, `rsx!`'s generated `memoize` points its old event handler at the
new one. Upstream unwraps that, and it fails whenever the old handler's
owner — the scope that created the closure — has already dropped while the
component still held it (an element built in one scope and rendered from
another: a popup's render function, a face remounted mid-update). The
result was `called Result::unwrap() on an Err value: Dropped(...)` on the
main thread, which on iOS aborts the app. The vendored copy logs a warning
and takes the new handler instead. Still an `unwrap()` upstream at
`430cfb56`.

`Cargo.toml` is the upstream one with the workspace-inherited dependencies
written out (dioxus siblings as git deps at the same rev; dev-deps, benches
and tests dropped). When the pin moves, re-vendor from the new rev and carry
the change across — or drop this copy once upstream stops unwrapping there.
