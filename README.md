# vendor

Third-party crates FastTrackStudio patches, kept in one place so the fix
lives once instead of once per repo.

Every consumer repo (`architect`, `task`, `fasttrackstudio`) points a
`[patch.crates-io]` entry at this repo and pins its own rev.

**These are pinned to `*-rc.5`, matching the published versions on
crates.io. Do not casually rebase onto upstream `main`** — upstream has
drifted (see the notes below), and a rebase silently mixes API changes
into what is meant to be a minimal fix.

| crate | upstream | pinned to | why |
|---|---|---|---|
| `phon` | [bearcove/phon](https://github.com/bearcove/phon) | `0.2.0-rc.5` | `Def::Scalar` opaque shapes |
| `phon-jit` | [bearcove/phon](https://github.com/bearcove/phon) | `0.2.0-rc.5` | nightly probe breaks under nix |
| `styx-format` | [bearcove/styx](https://github.com/bearcove/styx) | `5.0.0-rc.5` | angle brackets in bare scalars |

## phon — `Def::Scalar` opaque shapes

**Symptom:** every `task-server` list RPC aborts the process.

vox wraps each RPC reply payload as a phon opaque field. Upstream's
derive has no branch for `Def::Scalar` opaque shapes (`Uuid`, `chrono`
types, `url::Url`) — `ref_of` falls through to
`DeriveError::Unsupported`. Because the encode thunk is `extern "C"`,
the panic crosses a non-unwinding boundary and aborts rather than
unwinding.

**Fix** (`src/derive.rs`, ~130 lines): a last-resort branch encoding
display/parse scalars as a `Primitive::Bytes` UTF-8 `Display` run,
decoded via the shape's parse vtable. All pre-existing branches keep
precedence, so previously-working shapes encode byte-identically.

**Upstream status:** not fixed. `derive.rs` on `main` has no
`Def::Scalar`, `has_display`, `has_parse`, or Display fallback; the
`ref_of` chain still ends in `Unsupported`.

**Rebase warning:** upstream `main` renamed `SchemaId(..)` to
`SchemaId::from_raw(..)` throughout. The total diff against `main` is
~221 lines, of which only ~18 are this fix.

## phon-jit — nightly probe breaks under nix

**Symptom:** the macOS desktop build fails to compile stencils.

Upstream's `build.rs` calls `nightly_available()`, which probes
`rustc +nightly`. The nix-pinned toolchain has no rustup nightly, and
the stable fallback cannot compile the stencils either because `become`
(explicit tail calls) is *parse*-gated, not feature-gated.

**Fix** (`build.rs` only, ~30 lines): use `PHON_JIT_NIGHTLY_RUSTC` when
set, otherwise set `RUSTC_BOOTSTRAP=1` and always compile the tail-call
stencils. Sources and stencils are untouched.

**Upstream status:** not fixed. `build.rs` on `main` still calls
`nightly_available()`, with no env override and no cfg gate.

**Note for a future re-vendor:** upstream `main` now gates the
arm64-macOS path on `DEP_WEAVY_JIT == "1"` and adds a `phon_jit_native`
cfg. That is not in rc.5 and does not help the aarch64-apple-darwin
build this fix exists for.

## styx-format — angle brackets in bare scalars

**Symptom:** a saved keybind config cannot be re-parsed.

`facet-styx` emits `keys <C-s>` as a bare scalar, and styx-parse then
rejects `>` because it is the attribute separator. The formatter
produces a file its own parser refuses.

**Fix** (`src/scalar.rs`, one line): add `'<' | '>'` to the set
`can_be_bare` rejects.

**Upstream status:** not fixed — `scalar.rs` is unchanged since
2026-01-16 and angle brackets are still absent from the reject set.
Upstream's own compliance corpus documents the bug
(`compliance/corpus/01-scalars/bare-angle-brackets.styx`: "`<` can
appear in bare scalars, but `>` cannot").

Strictly only `>` is fatal; quoting both is the safe superset.

## Upstreaming

All three are filable against the `bearcove` repos, which are active.
Doing so is a deliberate decision, not a chore to run unprompted.
