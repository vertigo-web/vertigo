//! The DOM workloads.
//!
//! Constants match the graph suites (`tests/reactive-bench`, and `compare.rs` in the vertigo
//! crate) so the shapes correspond, but these run on the **default graph**: the DOM flush is
//! wired to `on_after_transaction` on that graph, so a write anywhere else renders nothing.
//!
//! ## Steady state
//!
//! `run_one` repeats an operation thousands of times, so the scene must cost the same on the
//! last iteration as on the first. Every workload declares which pattern keeps it that way:
//!
//! - **P**air - the operation is a mutation and its inverse (mount/unmount, append/remove).
//!   The reported figure is the round trip; the tree size never drifts.
//! - **A**lternating - one write that flips between two values of *equal size*.
//! - **S**elf-inverse - one write whose effect undoes itself (a reverse).
//! - **C**utoff - one write that propagates but is stopped before it reaches the DOM.
//!
//! Two traps that silently make a DOM benchmark measure nothing, both guarded below:
//! `Value::set` short-circuits when the new value equals the old, and a string that grows
//! makes late iterations cost more than early ones. The command-count pass is the detector
//! for the first: a workload that writes an equal value reports zero commands.
//!
//! ## Baseline
//!
//! Recorded 2026-08-24 on Linux x86_64 (6.18.12-amd64), Chrome 145.0.7632.109 via
//! ChromeDriver 145, release build with `wasm-opt -Os`.
//!
//! ```text
//! workload                        per op      cmds
//! list-mount-unmount            27350 us     17000
//! list-append-remove              285 us        34
//! list-append-remove-small         82 us        34
//! list-middle-remove-reinsert     277 us        34
//! list-reverse                   1317 us      1000
//! list-reverse-layout            4843 us      1000
//! list-update-text                6.6 us         3
//! list-toggle-class               5.1 us         1
//! editor-keystroke-embed          7.0 us         3
//! editor-keystroke-patch          4.5 us         1
//! editor-toggle-bold              5.3 us         1
//! editor-caret-move               0.4 us         0
//! editor-block-insert-delete      163 us        13
//! dash-tick-all                   644 us       600
//! dash-tick-one                   6.9 us         3
//! dash-status-change               35 us        15
//! flush-min                       5.2 us         1
//! ```
//!
//! Read the table against two subtrahends. **`flush-min` is 5.2us**: one flush carrying one
//! command, which every operation pays and almost none of which is DOM work - it is the
//! wasm to JS round trip. So `editor-toggle-bold` at 5.3us is essentially free, and the
//! interesting figures are the ones well above the floor. The graph suite's
//! `clock-roundtrip` measures the same crossing at ~4.1us, which corroborates it.
//!
//! What the numbers say:
//!
//! - **Patching a text node beats replacing one.** `editor-keystroke-patch` (4.5us, one
//!   `UpdateText`) against `editor-keystroke-embed` (7.0us, three commands and a node swap):
//!   36% less time and a third of the commands, for what an app author writes as the same
//!   thing. `DomText::new_computed` is public and used nowhere; ordinary `{value}`
//!   interpolation takes the slower path. The wall-clock gap is modest only because the
//!   round trip dominates a one-command flush.
//! - **The equality cutoff reaches all the way to the DOM.** `editor-caret-move` is 0.4us
//!   and zero commands - thirteen times cheaper than the cheapest operation that does touch
//!   the DOM.
//! - **Layout is the larger half of a reorder.** `list-reverse` mutates in 1317us;
//!   forcing the browser to settle the layout it invalidated costs 4843us. Vertigo's share
//!   of a full 500-row reorder is about a quarter of what the user actually waits for.
//! - **One-row list edits are dominated by reconciliation, not by the DOM.** Appending one
//!   row emits the same 34 commands whether the list holds 50 rows or 500, yet costs 82us
//!   against 285us. The DOM work is constant; the reconciler's per-update walk is not.
//! - **Batching saves flushes, not commands.** `dash-tick-all` emits exactly 200x what
//!   `dash-tick-one` emits, but at 644us against 6.9us x 200 = 1380us it is more than twice
//!   as fast, because it is one transaction and therefore one flush.
//!
//! Iteration counts are tuned so every batch lands in 100-300ms on that machine. Re-tune if
//! a batch drifts far outside that band; `?scale=` shortens a run without changing the
//! per-operation figures.

use std::{cell::Cell, rc::Rc};

use vertigo::transaction;
use vertigo_bench_support::{Bench, Workload};

use crate::{
    scenes::{
        dash::{self, DashScene, SITES},
        editor::{self, BLOCKS, EditorScene, RUN_LEN, TextMode},
        list::{self, ITEMS, ITEMS_SMALL, ListScene},
        probe::{self, ProbeScene},
    },
    stage::{Scene, StageGuard},
};

pub const WORKLOADS: &[Workload] = &[
    Workload {
        slug: "list-mount-unmount",
        title: "mount 500 rows, then unmount them",
        iters: 10,
        make: make_list_mount_unmount,
    },
    Workload {
        slug: "list-append-remove",
        title: "append one row to 500, then remove it",
        iters: 800,
        make: make_list_append_remove,
    },
    Workload {
        slug: "list-append-remove-small",
        title: "append one row to 50, then remove it",
        iters: 2_000,
        make: make_list_append_remove_small,
    },
    Workload {
        slug: "list-middle-remove-reinsert",
        title: "remove a middle row, then put it back",
        iters: 800,
        make: make_list_middle,
    },
    Workload {
        slug: "list-reverse",
        title: "reverse the order of 500 rows",
        iters: 100,
        make: make_list_reverse,
    },
    Workload {
        slug: "list-reverse-layout",
        title: "reverse 500 rows, then force layout",
        iters: 40,
        make: make_list_reverse_layout,
    },
    Workload {
        slug: "list-update-text",
        title: "change one row's label",
        iters: 25_000,
        make: make_list_update_text,
    },
    Workload {
        slug: "list-toggle-class",
        title: "toggle one row's class",
        iters: 40_000,
        make: make_list_toggle_class,
    },
    Workload {
        slug: "editor-keystroke-embed",
        title: "one keystroke, text via {value} interpolation",
        iters: 25_000,
        make: make_keystroke_embed,
    },
    Workload {
        slug: "editor-keystroke-patch",
        title: "one keystroke, text via DomText::new_computed",
        iters: 35_000,
        make: make_keystroke_patch,
    },
    Workload {
        slug: "editor-toggle-bold",
        title: "toggle bold on one block",
        iters: 30_000,
        make: make_editor_toggle_bold,
    },
    Workload {
        slug: "editor-caret-move",
        title: "move the caret inside one formatting run",
        iters: 500_000,
        make: make_editor_caret_move,
    },
    Workload {
        slug: "editor-block-insert-delete",
        title: "insert a paragraph mid-document, then delete it",
        iters: 800,
        make: make_editor_block_insert_delete,
    },
    Workload {
        slug: "dash-tick-all",
        title: "one transaction, fresh latency on all 200 sites",
        iters: 250,
        make: make_dash_tick_all,
    },
    Workload {
        slug: "dash-tick-one",
        title: "fresh latency on one site",
        iters: 25_000,
        make: make_dash_tick_one,
    },
    Workload {
        slug: "dash-status-change",
        title: "one site goes down (banner mounts), then recovers",
        iters: 6_000,
        make: make_dash_status_change,
    },
    Workload {
        slug: "flush-min",
        title: "diagnostic: one flush carrying one command",
        iters: 24_000,
        make: make_flush_min,
    },
];

/// Make the browser settle the layout the previous operation invalidated.
///
/// `DriverDom.update` applies every command inline, so a reorder measures a thousand
/// `insertBefore` calls and none of the reflow they imply - the browser defers that until
/// something asks. Reading `offsetHeight` asks. The gap between the two reverse workloads,
/// less one clock round trip, is the browser's layout share of a full reorder.
fn force_layout() {
    let _unused = vertigo::js! { window.document.body.offsetHeight };
}

// -- 1. list widget ----------------------------------------------------------

/// Shared by every list workload: the scene, its stage guard, and the two counters.
struct ListBench {
    scene: Rc<ListScene>,
    _stage: StageGuard,
    step: Cell<u32>,
    writes: Cell<u64>,
}

impl ListBench {
    fn build(count: u32) -> ListBench {
        let scene = list::build(count);
        let stage = StageGuard::mount(Scene::List(scene.clone()));
        ListBench {
            scene,
            _stage: stage,
            step: Cell::new(0),
            writes: Cell::new(0),
        }
    }

    fn next_step(&self) -> u32 {
        let step = self.step.get();
        self.step.set(step.wrapping_add(1));
        step
    }

    fn wrote(&self, count: u64) {
        self.writes.set(self.writes.get() + count);
    }

    /// Reads every label, so nothing in the pipeline can decide the rows are unobserved.
    fn checksum(&self) -> u64 {
        transaction(|ctx| {
            self.scene.order.get(ctx).len() as u64 * 1_000_000
                + self
                    .scene
                    .rows
                    .values()
                    .map(|row| row.label.get(ctx).len() as u64)
                    .sum::<u64>()
        })
    }
}

/// Pattern P. One operation mounts 500 rows and unmounts them again, which is the only way
/// to measure creation repeatedly without the tree growing without bound.
struct ListMountUnmount(ListBench);

fn make_list_mount_unmount() -> Box<dyn Bench> {
    let bench = ListBench::build(ITEMS);
    // Start empty: the scene builder mounts the full list, and the operation below begins
    // by filling it.
    bench.scene.order.set(Vec::new());
    Box::new(ListMountUnmount(bench))
}

impl Bench for ListMountUnmount {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            self.0.scene.order.set(self.0.scene.keys.clone());
            self.0.scene.order.set(Vec::new());
            self.0.wrote(2);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern P. Append at the end, then remove it: the reconciler's longest-common-prefix
/// phase should absorb every surviving row, leaving O(1) work regardless of list length.
struct ListAppendRemove(ListBench);

fn make_list_append_remove() -> Box<dyn Bench> {
    Box::new(ListAppendRemove(ListBench::build(ITEMS)))
}

fn make_list_append_remove_small() -> Box<dyn Bench> {
    Box::new(ListAppendRemove(ListBench::build(ITEMS_SMALL)))
}

impl Bench for ListAppendRemove {
    fn run(&self, iters: u32) {
        let mut appended = self.0.scene.keys.clone();
        appended.push(self.0.scene.spare);

        for _ in 0..iters {
            self.0.scene.order.set(appended.clone());
            self.0.scene.order.set(self.0.scene.keys.clone());
            self.0.wrote(2);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern P. The same one-row churn as `list-append-remove`, but in the middle, where the
/// prefix and suffix phases both stop short and the keyed middle map does the work. The
/// command count should be identical - only the anchor differs.
struct ListMiddle(ListBench);

fn make_list_middle() -> Box<dyn Bench> {
    Box::new(ListMiddle(ListBench::build(ITEMS)))
}

impl Bench for ListMiddle {
    fn run(&self, iters: u32) {
        let middle = (ITEMS / 2) as usize;
        let mut without = self.0.scene.keys.clone();
        without.remove(middle);

        for _ in 0..iters {
            self.0.scene.order.set(without.clone());
            self.0.scene.order.set(self.0.scene.keys.clone());
            self.0.wrote(2);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern S. A reverse is its own inverse, so repeating it is naturally steady. It is also
/// the reconciler's worst case: prefix and suffix match nothing, so every row goes through
/// the middle map and is re-inserted.
struct ListReverse {
    bench: ListBench,
    force_layout: bool,
}

fn make_list_reverse() -> Box<dyn Bench> {
    Box::new(ListReverse {
        bench: ListBench::build(ITEMS),
        force_layout: false,
    })
}

fn make_list_reverse_layout() -> Box<dyn Bench> {
    Box::new(ListReverse {
        bench: ListBench::build(ITEMS),
        force_layout: true,
    })
}

impl Bench for ListReverse {
    fn run(&self, iters: u32) {
        let forward = self.bench.scene.keys.clone();
        let mut backward = forward.clone();
        backward.reverse();

        for _ in 0..iters {
            let step = self.bench.next_step();
            let order = if step.is_multiple_of(2) {
                &backward
            } else {
                &forward
            };
            self.bench.scene.order.set(order.clone());
            if self.force_layout {
                force_layout();
            }
            self.bench.wrote(1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.bench.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.bench.checksum()
    }
}

/// Pattern A. One fixed row's label, alternating between two labels of equal length.
///
/// One fixed row, not a sweep: sweeping the 500 rows while alternating on `step % 2` would
/// hand every row the same string it already holds - the stride is even - and `Value::set`
/// would swallow every write. The command count would then read zero, which is how that
/// mistake gets caught rather than shipped.
struct ListUpdateText(ListBench);

fn make_list_update_text() -> Box<dyn Bench> {
    Box::new(ListUpdateText(ListBench::build(ITEMS)))
}

impl Bench for ListUpdateText {
    fn run(&self, iters: u32) {
        let Some(row) = self.0.scene.rows.get(&(ITEMS / 2)) else {
            return;
        };
        for _ in 0..iters {
            let step = self.0.next_step();
            row.label
                .set(self.0.scene.labels[(step % 2) as usize].clone());
            self.0.wrote(1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern A. The cheapest possible per-row update: one attribute, no node churn.
struct ListToggleClass(ListBench);

fn make_list_toggle_class() -> Box<dyn Bench> {
    Box::new(ListToggleClass(ListBench::build(ITEMS)))
}

impl Bench for ListToggleClass {
    fn run(&self, iters: u32) {
        let Some(row) = self.0.scene.rows.get(&(ITEMS / 2)) else {
            return;
        };
        for _ in 0..iters {
            let step = self.0.next_step();
            row.class
                .set(self.0.scene.classes[(step % 2) as usize].clone());
            self.0.wrote(1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

// -- 2. text editor ----------------------------------------------------------

struct EditorBench {
    scene: Rc<EditorScene>,
    _stage: StageGuard,
    step: Cell<u32>,
    writes: Cell<u64>,
}

impl EditorBench {
    fn build(mode: TextMode) -> EditorBench {
        let scene = editor::build(mode);
        let stage = StageGuard::mount(Scene::Editor(scene.clone()));
        EditorBench {
            scene,
            _stage: stage,
            step: Cell::new(0),
            writes: Cell::new(0),
        }
    }

    fn next_step(&self) -> u32 {
        let step = self.step.get();
        self.step.set(step.wrapping_add(1));
        step
    }

    /// Reads the toolbar as well as the text, so the caret workload - whose write never
    /// reaches the DOM - still has an observable result.
    fn checksum(&self) -> u64 {
        transaction(|ctx| {
            let bold = u64::from(self.scene.bold_at_caret.get(ctx));
            let text: u64 = self
                .scene
                .blocks
                .values()
                .map(|block| block.text.get(ctx).len() as u64)
                .sum();
            bold * 1_000_000 + text
        })
    }
}

/// Pattern A. One keystroke on one block: the hottest path in any editor.
///
/// The two constructors below differ *only* in `TextMode`. Everything else - the scene, the
/// block count, the strings, the row markup - is identical, or the comparison would be
/// worthless.
struct Keystroke(EditorBench);

fn make_keystroke_embed() -> Box<dyn Bench> {
    Box::new(Keystroke(EditorBench::build(TextMode::Embed)))
}

fn make_keystroke_patch() -> Box<dyn Bench> {
    Box::new(Keystroke(EditorBench::build(TextMode::Patch)))
}

impl Bench for Keystroke {
    fn run(&self, iters: u32) {
        let Some(block) = self.0.scene.blocks.get(&(BLOCKS / 2)) else {
            return;
        };
        for _ in 0..iters {
            let step = self.0.next_step();
            block
                .text
                .set(self.0.scene.texts[(step % 2) as usize].clone());
            self.0.writes.set(self.0.writes.get() + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern A. A formatting change on one block - one class attribute, no node churn.
struct EditorToggleBold(EditorBench);

fn make_editor_toggle_bold() -> Box<dyn Bench> {
    Box::new(EditorToggleBold(EditorBench::build(TextMode::Embed)))
}

impl Bench for EditorToggleBold {
    fn run(&self, iters: u32) {
        let Some(block) = self.0.scene.blocks.get(&(BLOCKS / 2)) else {
            return;
        };
        for _ in 0..iters {
            let step = self.0.next_step();
            block.bold.set(step.is_multiple_of(2));
            self.0.writes.set(self.0.writes.get() + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern C. The most frequent event in an editor, and it must reach the DOM not at all.
///
/// The caret sweeps inside one formatting run, so `active_block` recomputes to the same
/// value and the equality cutoff stops there. Read the zero command count together with the
/// write count: "cut off before the DOM" and "optimised away entirely" look identical
/// otherwise.
struct EditorCaretMove(EditorBench);

fn make_editor_caret_move() -> Box<dyn Bench> {
    Box::new(EditorCaretMove(EditorBench::build(TextMode::Embed)))
}

impl Bench for EditorCaretMove {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.0.next_step();
            // Never leaves run zero, and never writes the same offset twice in a row, so
            // `Value::set`'s equality check cannot swallow the write.
            self.0.scene.caret.set(step % RUN_LEN);
            self.0.writes.set(self.0.writes.get() + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

/// Pattern P. Structural editing: a paragraph appears mid-document, then goes away again.
struct EditorBlockInsertDelete(EditorBench);

fn make_editor_block_insert_delete() -> Box<dyn Bench> {
    Box::new(EditorBlockInsertDelete(EditorBench::build(TextMode::Embed)))
}

impl Bench for EditorBlockInsertDelete {
    fn run(&self, iters: u32) {
        let mut with_extra = self.0.scene.keys.clone();
        with_extra.insert((BLOCKS / 2) as usize, self.0.scene.spare);

        for _ in 0..iters {
            self.0.scene.order.set(with_extra.clone());
            self.0.scene.order.set(self.0.scene.keys.clone());
            self.0.writes.set(self.0.writes.get() + 2);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        self.0.checksum()
    }
}

// -- 3. dashboard ------------------------------------------------------------

struct DashBench {
    scene: Rc<DashScene>,
    _stage: StageGuard,
    step: Cell<u32>,
    writes: Cell<u64>,
}

impl DashBench {
    fn build() -> DashBench {
        let scene = dash::build();
        let stage = StageGuard::mount(Scene::Dash(scene.clone()));
        DashBench {
            scene,
            _stage: stage,
            step: Cell::new(0),
            writes: Cell::new(0),
        }
    }

    fn next_step(&self) -> u32 {
        let step = self.step.get();
        self.step.set(step.wrapping_add(1));
        step
    }
}

/// Pattern A. A poll cycle: every site gets a fresh reading, inside one transaction, so the
/// whole dashboard lands in a single propagation wave and a single bulk DOM update.
struct DashTickAll(DashBench);

fn make_dash_tick_all() -> Box<dyn Bench> {
    Box::new(DashTickAll(DashBench::build()))
}

impl Bench for DashTickAll {
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.0.next_step() as usize;
            // Nested `set`s do not flush; only the outermost transaction does.
            transaction(|_ctx| {
                for (index, site) in self.0.scene.sites.iter().enumerate() {
                    let slot = (step + index) % self.0.scene.latencies.len();
                    site.latency.set(self.0.scene.latencies[slot].clone());
                }
            });
            self.0.writes.set(self.0.writes.get() + u64::from(SITES));
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        dash::total_len(&self.0.scene)
    }
}

/// Pattern A. One site's reading. Because the aggregate depends on the status flag rather
/// than the latency, this is exactly one row's text update and nothing else.
struct DashTickOne(DashBench);

fn make_dash_tick_one() -> Box<dyn Bench> {
    Box::new(DashTickOne(DashBench::build()))
}

impl Bench for DashTickOne {
    fn run(&self, iters: u32) {
        let Some(site) = self.0.scene.sites.get((SITES / 2) as usize) else {
            return;
        };
        for _ in 0..iters {
            let step = self.0.next_step() as usize;
            site.latency
                .set(self.0.scene.latencies[step % self.0.scene.latencies.len()].clone());
            self.0.writes.set(self.0.writes.get() + 1);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        dash::total_len(&self.0.scene)
    }
}

/// Pattern P rather than A: the two halves are not symmetric. Going down mounts the banner
/// subtree, coming back up unmounts it, so a single-`set` operation would report whichever
/// half the counting pass happened to catch.
struct DashStatusChange(DashBench);

fn make_dash_status_change() -> Box<dyn Bench> {
    Box::new(DashStatusChange(DashBench::build()))
}

impl Bench for DashStatusChange {
    fn run(&self, iters: u32) {
        let Some(site) = self.0.scene.sites.get((SITES / 2) as usize) else {
            return;
        };
        for _ in 0..iters {
            site.down.set(true);
            site.down.set(false);
            self.0.writes.set(self.0.writes.get() + 2);
        }
    }

    fn take_runs(&self) -> u64 {
        self.0.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        dash::total_len(&self.0.scene)
    }
}

// -- diagnostic: the floor every other workload stands on --------------------

struct FlushMin {
    scene: Rc<ProbeScene>,
    _stage: StageGuard,
    step: Cell<u32>,
    writes: Cell<u64>,
}

fn make_flush_min() -> Box<dyn Bench> {
    let scene = probe::build();
    let stage = StageGuard::mount(Scene::Probe(scene.clone()));
    Box::new(FlushMin {
        scene,
        _stage: stage,
        step: Cell::new(0),
        writes: Cell::new(0),
    })
}

impl Bench for FlushMin {
    /// Pattern A. One `set` of one attribute on one element: the least DOM work a flush can
    /// carry, so what this costs is overhead every other workload also pays.
    fn run(&self, iters: u32) {
        for _ in 0..iters {
            let step = self.step.get();
            self.step.set(step.wrapping_add(1));

            self.scene
                .class
                .set(self.scene.classes[(step % 2) as usize].clone());
            self.writes.set(self.writes.get() + 1);
        }
    }

    /// Reinterpreted for this suite: `Value::set` calls performed, not compute-closure runs.
    /// It is what makes "one operation is a pair" checkable from the report, and it
    /// distinguishes "cut off before the DOM" from "optimised away entirely".
    fn take_runs(&self) -> u64 {
        self.writes.replace(0)
    }

    fn checksum(&self) -> u64 {
        vertigo::transaction(|ctx| self.scene.class.get(ctx).len() as u64)
    }
}
