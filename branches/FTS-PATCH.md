# branches 0.4.6 — the stable path on a nightly compiler

**Upstream:** [fereidani/branches](https://github.com/fereidani/branches),
the published 0.4.6, verbatim except `build.rs`. MIT (LICENSE).

**Symptom:** on a current nightly (nightly-2026-10-20, rustc 1.101) the crate
does not compile:

```
error[E0425]: cannot find function `abort` in module `core::intrinsics`
error[E0308]: mismatched types: expected `!`, found `()`  (src/lib.rs:41)
```

`build.rs` emits `branches_nightly` whenever rustc is a nightly, and that
cfg switches every hint to `core::intrinsics`, which has since dropped
`abort` (and keeps moving: it is an internal, unstable module). turso_core
(0.8.3, 0.9.0-pre.1) requires `branches ^0.4.3`, so the 0.5 line is out of
reach for a `[patch]`.

**Fix** (`build.rs`): emit `branches_stable` on every channel. On rustc
1.95+ the stable path uses `core::hint::cold_path`, the same hint the
intrinsics gave; nothing in the crate's API changes.

**Drop it** when turso_core moves to a `branches` that builds on nightly.
