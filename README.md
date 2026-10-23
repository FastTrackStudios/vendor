# vendor

Third-party crates FastTrackStudio patches, kept in one place so the fix
lives once instead of once per repo.

Every consumer repo (`architect`, `task`, `fasttrackstudio`) points a
`[patch.crates-io]` entry at this repo and pins its own rev.

**Each is the published crates.io release named in the table, verbatim,
plus the one fix. Do not casually rebase onto upstream `main`** — upstream
has drifted (see the notes below), and a rebase silently mixes API changes
into what is meant to be a minimal fix. To move to a newer release, start
from that release's published source and re-apply the fix; the fleet
(facet 0.50 / vox 0.10 / phon 0.2) is on **rc.7** as of 2026-09.

| crate | upstream | pinned to | why |
|---|---|---|---|
| `phon` | [bearcove/phon](https://github.com/bearcove/phon) | `0.2.0-rc.7` | `Def::Scalar` opaque shapes |
| `phon-jit` | [bearcove/phon](https://github.com/bearcove/phon) | `0.2.0-rc.7` | nightly probe breaks under nix |
| `styx-format` | [bearcove/styx](https://github.com/bearcove/styx) | `5.0.0-rc.5` | angle brackets in bare scalars |
| `facet-core` | [facet-rs/facet](https://github.com/facet-rs/facet) | `0.50.0-rc.7` | chrono display truncates sub-second precision |
| `vox-phon` | [bearcove/vox](https://github.com/bearcove/vox) | `0.10.0-rc.7` | bogus post-decode length check aborts every real schema exchange |
| `tao` | [tauri-apps/tao](https://github.com/tauri-apps/tao) | `0.35.3` | iOS release builds crash at launch on a released scene configuration |
| `anyrender_vello` | [dioxuslabs/anyrender](https://github.com/dioxuslabs/anyrender) | `0.12.0` | window-renderer fixes found at 2560x1440 + the FTS blitz compositor/rasterizer (studio) |
| `anyrender_vello_hybrid` | [dioxuslabs/anyrender](https://github.com/dioxuslabs/anyrender) | `0.8.0` | `set_size` resizes a wrong-sized surface (studio) |
| `baseview` | [RustAudio/baseview](https://github.com/RustAudio/baseview) | `0.3.3` | macOS frame timer in common run-loop modes, so live resize doesn't freeze (studio) |
| `dioxus-core` | [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) | rev `f717a8e1` | `Callback::__point_to` no longer unwraps a dropped owner (studio) |
| `dioxus-native` | [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) | rev `f717a8e1` | iOS simulator takes `vello-hybrid` (studio) |
| `dioxus-native-dom` | [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) | rev `f717a8e1` | stale ElementId→NodeId mappings after subtree removal (studio) |
| `raw-window-metal` | [rust-windowing/raw-window-metal](https://github.com/rust-windowing/raw-window-metal) | `1.1.0` | ObjC class name collides across plugin bundles (studio) |
| `wgpu-hal` | [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu) | `29.0.4` | iOS Simulator has indirect draw/dispatch (studio) |
| `winit-uikit` | [rust-windowing/winit](https://github.com/rust-windowing/winit) | `0.31.0-beta.2` | proxy wake-up drains events queued during it (studio) |
| `world-sys` (`world/`) | [mmorise/World](https://github.com/mmorise/World) | — | the WORLD vocoder, BSD-3-Clause (studio) |

Each of the crates marked (studio) carries its own `UPSTREAM.md` or
`FTS-PATCH.md` with the full story. They moved here from studio's
in-tree `vendor/` (2026-10) so a studio CI run caches them as compiled
dependencies instead of building and testing them as workspace members;
studio patches them in from a tag of this repo.

## tao — the scene configuration it returns is already released

**Symptom:** a release iOS build of a Dioxus app crashes on every launch,
before any app code: `EXC_BAD_ACCESS` in `objc_retain`, under
`-[UIApplication _connectUISceneFromFBSScene:transitionContext:]`. A debug
build of the same code runs. iOS 27 requires the scene lifecycle, so every
app with `UIApplicationSupportsMultipleScenes` takes this path.

tao's `application:configurationForConnectingSceneSession:options:`
returned `Retained::as_ptr(&config)` and then dropped `config` — the only
reference — so the caller received a pointer to an object it had just
released. In debug an autorelease pool happened to hold it; in release the
objc runtime's autorelease elision does not, and UIKit retains freed
memory.

**Fix** (`src/platform_impl/ios/view.rs`, one line): return
`Retained::autorelease_return(config)`, the +0 autoreleased reference an
Objective-C method returning an object owes its caller.

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

**Upstream status:** not fixed as of the published `0.2.0-rc.7`:
`derive.rs` has no `Def::Scalar`, `has_display`, `has_parse`, or Display
fallback; the `ref_of` chain still ends in `Unsupported`.

**Forward-ported to rc.7** (2026-09): the published rc.7 crate verbatim
plus the same four hunks in `src/derive.rs` (129 added lines, nothing
removed). The only adaptation is the `ref_of` hunk's neighbours, which
rc.7 spells `SchemaId::from_raw(..)` where rc.5 had `SchemaId(..)`.
Covered by `tests/display_scalar.rs` (its own test target, with its own
dev-deps, because the publish strips the upstream tests' dev-deps); it
fails on unpatched rc.7 with the lowering error.

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

rc.5 is still the newest published styx-format, and `facet-styx`
5.0.0-rc.7 depends on `^5.0.0-rc.5`, so this entry needed no forward-port
for the rc.7 fleet.

## vox-phon — bogus post-decode length check aborts every real schema exchange

**Symptom:** any peer-to-peer schema exchange over vox fails with
`"schema decode consumed N of M bytes"` — a `daw-proto` batch type as
small as one enum variant already hits it (`"consumed 159 of 361
bytes"`), so this is not an edge case, it is the common case.

`parse_schema_bytes` (`src/schema.rs`) decodes each schema in the
closure with `phon_schema::schema_from_bytes`, then rejects the result
unless its second tuple element equals the encoded slice length. That
element is not a buffer position — `phon_schema` 0.2.0-rc.6 changed
`schema_from_bytes` to return `(Schema, usize)` where the `usize` is the
owned-allocation byte count charged against the new `DecodeLimits`
budget (string/Vec bytes it had to copy for `DecodeLimits` accounting).
It has no relationship to the serialized length: struct/enum tags,
field-name strings, and list-length headers all add encoded bytes
without charging the allocation budget, so the two numbers diverge for
every schema with any structure. `schema_from_bytes` already guarantees
full-slice consumption itself — it rejects trailing bytes internally
before returning `Ok` — so the check in `vox-phon` was both wrong and
redundant.

**Fix** (`src/schema.rs`, one line removed from `parse_schema_bytes`):
drop the `if consumed != slice.len() { ... }` block; keep the
`schema_from_bytes` call, ignoring its second element.

**Upstream status:** not filed yet. This surfaced from bumping
`architect` (which pulls `vox`) in the `daw` repo; not yet checked
against `bearcove/vox` `main`.

## Upstreaming

All are filable against the `bearcove`/`facet-rs` repos, which are
active. Doing so is a deliberate decision, not a chore to run
unprompted.

## facet-core — chrono display truncates sub-second precision

**Symptom:** every timestamp on every vox RPC arrives rounded to the
second, and an activity feed ordered by time puts a checkpoint before
the root creation it followed.

`chrono`'s types are registered as `Def::Scalar` / `UserType::Opaque`,
so their only encode path is the `display` fn in each vtable — there is
no structural representation for a format to fall back on. Upstream
0.50.0-rc.5 writes the `DateTime` types with
`to_rfc3339_opts(SecondsFormat::Secs, true)` and the naive types with
`%Y-%m-%dT%H:%M:%S` / `%H:%M:%S`. Still so in the published rc.7. All four drop the fraction, so every
facet format inherits the loss: facet-json, facet-toml, and vox.

It is silent and one-directional. The value round-trips without error
and comes back rounded, which is why it survived: nothing fails, and
two events in the same second simply stop being orderable.

**Fix:** `SecondsFormat::AutoSi` for the RFC3339 types, `%.f` for the
naive ones. Both write nothing when the fraction is zero, so a
whole-second value formats exactly as it did before and only the
previously-lossy case changes — existing files keep parsing, and the
diff on stored data is empty unless precision was actually being lost.
`NaiveDateTime`'s parse format gains `%.f` for the same reason (its
display now emits one; `NaiveTime`'s parse already accepted it).

**Forward-ported to rc.7** (2026-09): the published rc.7 crate verbatim
(its `Cargo.toml.orig` kept as `Cargo.toml.upstream`) plus the same
nine-line `src/impls/crates/chrono.rs` change and test target.

Covered by `facet-core/tests/chrono_precision.rs`, which is its own
test target: the crate's `main` target pulls in `type_name.rs`, whose
`facet-testhelpers` dev-dependency the publish strips, so `main` cannot
build from the published source at all.
