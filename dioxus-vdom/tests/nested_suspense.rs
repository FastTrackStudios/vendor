//! Suspense boundaries nested under a boundary that is still suspended
//! (FastTrackStudios/task#410, the core half of DioxusLabs/dioxus#5668).
//!
//! A scope rendered while an ancestor boundary is suspended lives in the
//! background: it was created without a writer, so the renderer never saw its
//! nodes. In the wasm-split web build every route sits behind a
//! `SuspenseBoundary` (the lazy route chunk suspends it), and a nested
//! boundary inside the route resolves on its own before the chunk arrives.
//! Upstream at `430cfb56` decides "may this scope write?" from the nearest
//! boundary only, so the resolved inner boundary writes its fallback-to-
//! children swap to the real renderer against nodes that were never mounted
//! there: `cannot reclaim ElementId(N)` / a `ReplaceWith` on a dead node in the
//! browser, a blank page, the route chunk never requested.
//!
//! Every test drives a real event loop and applies every mutation batch to the
//! upstream renderer oracle, which panics on any edit that touches a node the
//! renderer does not have, and then checks the oracle's DOM against the
//! VirtualDom's own view of what is mounted.

use dioxus::prelude::*;
use dioxus_renderer_oracle::RendererOracle;
use std::time::Duration;

/// Drive `dom` until `deadline`, applying each batch to `oracle` and checking
/// the oracle against the VirtualDom after every batch.
async fn drive(dom: &mut VirtualDom, oracle: &mut RendererOracle, for_how_long: Duration) {
    let deadline = tokio::time::Instant::now() + for_how_long;
    loop {
        tokio::select! {
            _ = dom.wait_for_work() => {
                oracle.render(dom);
                oracle.assert_matches_vdom(dom);
            }
            _ = tokio::time::sleep_until(deadline) => break,
        }
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
}

/// Suspends its nearest boundary until `ms` have passed.
#[component]
fn SuspendFor(ms: u64, label: &'static str) -> Element {
    let mut resolved = use_signal(|| false);
    let task = use_hook(|| {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            resolved.set(true);
        })
    });
    if !resolved() {
        suspend(task)?;
    }
    rsx! { div { "{label}" } }
}

/// The wasm-split route shape: an explicit outer boundary (the router's
/// lazy-route boundary) suspended by a slow sibling (the chunk load), with an
/// inner boundary that resolves first.
#[test]
fn inner_boundary_resolves_while_outer_boundary_is_suspended() {
    fn app() -> Element {
        rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { p { "route loading" } },
                SuspendFor { ms: 300, label: "route" }
                SuspenseBoundary {
                    fallback: |_| rsx! { p { "inner loading" } },
                    SuspendFor { ms: 30, label: "inner" }
                }
            }
        }
    }

    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        let mut oracle = RendererOracle::new();
        oracle.rebuild(&mut dom);
        oracle.assert_matches_vdom(&dom);

        drive(&mut dom, &mut oracle, Duration::from_millis(600)).await;

        assert!(!dom.suspended_tasks_remaining(), "suspense never resolved");
        assert_eq!(dioxus_ssr::render(&dom), "<div>route</div><div>inner</div>");
    });
}

/// Resolves at 30ms, suspends again at 60ms, resolves for good at 90ms.
#[component]
fn Flip() -> Element {
    let mut phase = use_signal(|| 0u8);
    let first = use_hook(|| {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            phase.set(1);
            tokio::time::sleep(Duration::from_millis(30)).await;
            phase.set(2);
        })
    });
    let second = use_hook(|| {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(90)).await;
            phase.set(3);
        })
    });
    match phase() {
        0 => {
            suspend(first)?;
        }
        2 => {
            suspend(second)?;
        }
        _ => {}
    }
    rsx! { div { "flip {phase}" } }
}

/// An inner boundary that resolves and then suspends again while the outer
/// boundary is still suspended: a route whose nested data loader refetches
/// before the route chunk has arrived. The re-suspend swaps the inner
/// children for its fallback; with only the nearest boundary consulted, that
/// swap is written to the real renderer, placed against nodes that only exist
/// in the background.
#[test]
fn inner_boundary_resuspends_while_outer_boundary_is_suspended() {
    fn app() -> Element {
        rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { p { "route loading" } },
                SuspendFor { ms: 300, label: "route" }
                SuspenseBoundary {
                    fallback: |_| rsx! { p { "inner loading" } },
                    Flip {}
                }
            }
        }
    }

    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        let mut oracle = RendererOracle::new();
        oracle.rebuild(&mut dom);
        oracle.assert_matches_vdom(&dom);

        drive(&mut dom, &mut oracle, Duration::from_millis(600)).await;

        assert!(!dom.suspended_tasks_remaining(), "suspense never resolved");
        assert_eq!(
            dioxus_ssr::render(&dom),
            "<div>route</div><div>flip 3</div>"
        );
    });
}

/// Navigation: the outer boundary has rendered a page, then a route change
/// mounts a page that suspends it (the next lazy chunk) while that page's own
/// nested boundary resolves, suspends again and resolves.
#[test]
fn nested_boundary_flips_after_navigating_to_a_suspending_page() {
    static PAGE: GlobalSignal<u8> = Signal::global(|| 0);

    fn app() -> Element {
        use_hook(|| {
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                *PAGE.write() = 1;
            })
        });
        rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { p { "route loading" } },
                if PAGE() == 0 {
                    div { "home" }
                } else {
                    Page {}
                }
            }
        }
    }

    #[component]
    fn Page() -> Element {
        rsx! {
            SuspendFor { ms: 300, label: "page" }
            SuspenseBoundary {
                fallback: |_| rsx! { p { "inner loading" } },
                Flip {}
            }
        }
    }

    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        let mut oracle = RendererOracle::new();
        oracle.rebuild(&mut dom);
        oracle.assert_matches_vdom(&dom);

        drive(&mut dom, &mut oracle, Duration::from_millis(700)).await;

        assert!(!dom.suspended_tasks_remaining(), "suspense never resolved");
        assert_eq!(dioxus_ssr::render(&dom), "<div>page</div><div>flip 3</div>");
    });
}

/// The same with the implicit root boundary as the suspended ancestor
/// (#5668's `nested_boundary_resolves_under_suspended_root`).
#[test]
fn inner_boundary_resolves_while_root_is_suspended() {
    fn app() -> Element {
        rsx! {
            SuspendFor { ms: 300, label: "outer" }
            SuspenseBoundary {
                fallback: |_| rsx! { "inner loading" },
                SuspendFor { ms: 30, label: "inner" }
            }
        }
    }

    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        let mut oracle = RendererOracle::new();
        oracle.rebuild(&mut dom);
        oracle.assert_matches_vdom(&dom);

        drive(&mut dom, &mut oracle, Duration::from_millis(600)).await;

        assert!(!dom.suspended_tasks_remaining(), "suspense never resolved");
        assert_eq!(dioxus_ssr::render(&dom), "<div>outer</div><div>inner</div>");
    });
}

/// A boundary in the suspended background whose parent re-renders during the
/// outer suspension (memoized props replace its children), then everything
/// resolves and a later signal write re-renders the inner child
/// (#5668's `boundary_recreation_keeps_child_scopes`). If the outer resolve
/// rebuilt the inner boundary from its props instead of reusing its scopes,
/// the orphaned child scope would re-render against nodes that were never
/// written.
#[test]
fn boundary_recreation_keeps_child_scopes() {
    static SHARED: GlobalSignal<i32> = Signal::global(|| 0);

    fn app() -> Element {
        use_hook(|| {
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                *SHARED.write() = 1;
                tokio::time::sleep(Duration::from_millis(200)).await;
                *SHARED.write() = 2;
            })
        });
        rsx! {
            RootSuspender {}
            Section {}
        }
    }

    #[component]
    fn RootSuspender() -> Element {
        let mut resolved = use_signal(|| false);
        let task = use_hook(|| {
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                resolved.set(true);
            })
        });
        if !resolved() {
            suspend(task)?;
        }
        rsx! { header { "root done" } }
    }

    #[component]
    fn Section() -> Element {
        let shared = SHARED();
        rsx! {
            section {
                SuspenseBoundary {
                    fallback: |_| rsx! { div { "inner loading" } },
                    div { "gen {shared}" }
                    InnerChild {}
                }
            }
        }
    }

    #[component]
    fn InnerChild() -> Element {
        let shared = SHARED();
        let mut resolved = use_signal(|| false);
        let task = use_hook(|| {
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                resolved.set(true);
            })
        });
        if !resolved() {
            suspend(task)?;
        }
        if shared % 2 == 1 {
            rsx! { "inner {shared}" }
        } else {
            rsx! { span { "inner {shared}" } }
        }
    }

    runtime().block_on(async {
        let mut dom = VirtualDom::new(app);
        let mut oracle = RendererOracle::new();
        oracle.rebuild(&mut dom);
        oracle.assert_matches_vdom(&dom);

        drive(&mut dom, &mut oracle, Duration::from_millis(600)).await;

        assert!(!dom.suspended_tasks_remaining(), "suspense never resolved");
        assert_eq!(
            dioxus_ssr::render(&dom),
            "<header>root done</header><section><div>gen 2</div><span>inner 2</span></section>"
        );
    });
}
