# dioxus-native (vendored)

From DioxusLabs/dioxus `packages/native` at rev
`430cfb563f1b70602ef387f0c80f171d84527b41` (0.8.0-alpha.3 line) — the rev
the fleet pins every other dioxus crate to. Patched in through a consumer's
`[patch."https://github.com/DioxusLabs/dioxus"]`.

Two changes:

- `src/dioxus_renderer.rs`: the iOS simulator takes the `vello-hybrid`
  renderer even when `vello` is enabled. Full Vello needs indirect compute
  dispatch, which the simulator's Metal lacks (a wgpu validation panic at
  the first frame) unless the vendored `wgpu-hal` is patched in too; real
  iPhones and desktops keep Vello.
- `Cargo.toml`: `winit` is `=0.31.0-beta.4`, not upstream's `=beta.3`, to
  match Blitz `main` (blitz-shell re-exports winit's types; two winits in
  one graph do not type-check). Drop when Dioxus moves to beta.4.

`Cargo.toml` is otherwise the upstream one with the workspace-inherited
dependencies written out (dioxus siblings as git deps at the same rev). When
the pin moves, re-vendor from the new rev and carry the changes across.
