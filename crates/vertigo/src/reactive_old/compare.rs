//! Wall-clock and work-count comparison: previous graph (`reactive_old`) vs current (`reactive`).
//!
//! Run with: `cargo test -p vertigo --lib reactive_old::compare -- --nocapture`

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

use crate::reactive::{self as new, Graph};
use crate::reactive_old as old;

const FANOUT: usize = 10_000;
const CHAIN: usize = 200;
const UPDATES: usize = 200;

/// Item count of the list widget scenario.
const ITEMS: usize = 500;
/// Number of monitored websites in the dashboard scenario.
const SITES: usize = 200;
/// Number of paragraphs in the text editor scenario.
const BLOCKS: usize = 300;

fn elapsed(f: impl FnOnce()) -> Duration {
    let start = Instant::now();
    f();
    start.elapsed()
}

fn report(name: &str, old_d: Duration, new_d: Duration) {
    let old_ms = old_d.as_secs_f64() * 1000.0;
    let new_ms = new_d.as_secs_f64() * 1000.0;
    let speedup = if new_ms <= 0.0 {
        f64::INFINITY
    } else {
        old_ms / new_ms
    };
    eprintln!("compare {name:>28}: old={old_ms:8.3}ms  new={new_ms:8.3}ms  speedup={speedup:6.2}x");
}

struct FanoutOld {
    source: old::Value<i32>,
    _children: Vec<old::Computed<(bool, usize)>>,
    _subs: Vec<crate::DropResource>,
    runs: Rc<Cell<usize>>,
}

fn setup_fanout_old() -> FanoutOld {
    let source = old::Value::new(1);
    let even = old::Computed::from({
        let source = source.clone();
        move |ctx| source.get(ctx) % 2 == 0
    });
    let runs = Rc::new(Cell::new(0));
    let children: Vec<_> = (0..FANOUT)
        .map(|i| {
            let even = even.clone();
            let runs = runs.clone();
            old::Computed::from(move |ctx| {
                runs.set(runs.get() + 1);
                (even.get(ctx), i)
            })
        })
        .collect();
    let subs = children
        .iter()
        .cloned()
        .map(|child| child.subscribe(|_| {}))
        .collect();
    FanoutOld {
        source,
        _children: children,
        _subs: subs,
        runs,
    }
}

struct FanoutNew {
    source: new::Value<i32>,
    _children: Vec<new::Computed<(bool, usize)>>,
    _subs: Vec<crate::DropResource>,
    runs: Rc<Cell<usize>>,
}

fn setup_fanout_new() -> FanoutNew {
    let g = Graph::new();
    let source = g.value(1);
    let even = g.computed({
        let source = source.clone();
        move |ctx| source.get(ctx) % 2 == 0
    });
    let runs = Rc::new(Cell::new(0));
    let children: Vec<_> = (0..FANOUT)
        .map(|i| {
            let even = even.clone();
            let runs = runs.clone();
            g.computed(move |ctx| {
                runs.set(runs.get() + 1);
                (even.get(ctx), i)
            })
        })
        .collect();
    let subs = children
        .iter()
        .cloned()
        .map(|child| child.subscribe(|_| {}))
        .collect();
    FanoutNew {
        source,
        _children: children,
        _subs: subs,
        runs,
    }
}

#[test]
fn cutoff_fanout_skips_unchanged_children() {
    let old_g = setup_fanout_old();
    let new_g = setup_fanout_new();

    old_g.runs.set(0);
    new_g.runs.set(0);

    let old_d = elapsed(|| old_g.source.set(3));
    let new_d = elapsed(|| new_g.source.set(3));

    report("cutoff 10k (odd→odd)", old_d, new_d);

    assert_eq!(
        new_g.runs.get(),
        0,
        "new graph must cut off when even/odd is unchanged"
    );
    assert_eq!(
        old_g.runs.get(),
        FANOUT,
        "old graph invalidates the whole fan-out"
    );
}

#[test]
fn fanout_runs_when_parity_changes() {
    let old_g = setup_fanout_old();
    let new_g = setup_fanout_new();

    old_g.runs.set(0);
    new_g.runs.set(0);

    let old_d = elapsed(|| old_g.source.set(2));
    let new_d = elapsed(|| new_g.source.set(2));

    report("fanout 10k (odd→even)", old_d, new_d);

    assert_eq!(old_g.runs.get(), FANOUT);
    assert_eq!(new_g.runs.get(), FANOUT);
}

fn setup_chain_old() -> (old::Value<i32>, Vec<crate::DropResource>) {
    let source = old::Value::new(0);
    let mut prev: old::Computed<i32> = source.to_computed();
    let mut subs = Vec::new();
    for _ in 0..CHAIN {
        let next = old::Computed::from({
            let prev = prev.clone();
            move |ctx| prev.get(ctx) + 1
        });
        subs.push(next.clone().subscribe(|_| {}));
        prev = next;
    }
    (source, subs)
}

fn setup_chain_new() -> (new::Value<i32>, Vec<crate::DropResource>) {
    let g = Graph::new();
    let source = g.value(0);
    let mut prev: new::Computed<i32> = source.to_computed();
    let mut subs = Vec::new();
    for _ in 0..CHAIN {
        let next = g.computed({
            let prev = prev.clone();
            move |ctx| prev.get(ctx) + 1
        });
        subs.push(next.clone().subscribe(|_| {}));
        prev = next;
    }
    (source, subs)
}

#[test]
fn deep_chain_updates() {
    let (old_src, _old_subs) = setup_chain_old();
    let (new_src, _new_subs) = setup_chain_new();

    // Warm the caches.
    old_src.set(1);
    new_src.set(1);

    let old_d = elapsed(|| {
        for i in 2..(2 + UPDATES as i32) {
            old_src.set(i);
        }
    });
    let new_d = elapsed(|| {
        for i in 2..(2 + UPDATES as i32) {
            new_src.set(i);
        }
    });

    report(&format!("chain {CHAIN} x {UPDATES} sets"), old_d, new_d);
}

fn setup_diamond_old() -> (old::Value<i32>, crate::DropResource) {
    let a = old::Value::new(1);
    let left = old::Computed::from({
        let a = a.clone();
        move |ctx| a.get(ctx) + 1
    });
    let right = old::Computed::from({
        let a = a.clone();
        move |ctx| a.get(ctx) * 10
    });
    let d = old::Computed::from({
        let left = left.clone();
        let right = right.clone();
        move |ctx| left.get(ctx) + right.get(ctx)
    });
    let sub = d.subscribe(|_| {});
    (a, sub)
}

fn setup_diamond_new() -> (new::Value<i32>, crate::DropResource) {
    let g = Graph::new();
    let a = g.value(1);
    let left = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) + 1
    });
    let right = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) * 10
    });
    let d = g.computed({
        let left = left.clone();
        let right = right.clone();
        move |ctx| left.get(ctx) + right.get(ctx)
    });
    let sub = d.subscribe(|_| {});
    (a, sub)
}

#[test]
fn diamond_repeated_updates() {
    let (old_a, _old_sub) = setup_diamond_old();
    let (new_a, _new_sub) = setup_diamond_new();

    old_a.set(2);
    new_a.set(2);

    let old_d = elapsed(|| {
        for i in 3..(3 + UPDATES as i32) {
            old_a.set(i);
        }
    });
    let new_d = elapsed(|| {
        for i in 3..(3 + UPDATES as i32) {
            new_a.set(i);
        }
    });

    report(&format!("diamond x {UPDATES} sets"), old_d, new_d);
}

// ---------------------------------------------------------------------------
// Scenarios shaped like real application state.
//
// The graphs above are microbenchmarks; the ones below are the shapes an app
// actually builds - a list of items, a dashboard of metrics, a text editor -
// where most writes are small and most of the derived state does not move.
// ---------------------------------------------------------------------------

/// Counts how many times a single node recomputed.
#[derive(Clone, Default)]
struct Counter(Rc<Cell<usize>>);

impl Counter {
    fn hit(&self) {
        self.0.set(self.0.get() + 1);
    }

    fn get(&self) -> usize {
        self.0.get()
    }
}

macro_rules! counters {
    ($name:ident { $($field:ident),* $(,)? }) => {
        #[derive(Clone, Default)]
        struct $name {
            $($field: Counter,)*
        }

        impl $name {
            /// Zero everything, so a test measures one write and not the setup.
            fn reset(&self) {
                $(self.$field.0.set(0);)*
            }
        }
    };
}

counters!(ListCounters {
    row,
    total,
    footer,
    selected_count,
    any_selected,
    toolbar,
});

counters!(DashboardCounters {
    error_rate,
    health,
    row,
    total_visits,
    traffic_label,
    unhealthy,
    alert,
    banner,
});

counters!(EditorCounters {
    rendered,
    words,
    total_words,
    status,
    active_block,
    toolbar,
});

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Health {
    Ok,
    Warn,
    Down,
}

/// Both graphs expose the same surface (`Value::new`, `Computed::from`, `get`,
/// `set`, `subscribe`, `transaction`), so each scenario is written once and
/// instantiated for both. That also makes the two topologies identical by
/// construction - a hand-written pair could drift.
macro_rules! scenarios {
    ($module:ident, $api:ident) => {
        mod $module {
            use super::*;
            use crate::DropResource;
            use crate::$api as api;

            // -- 1. a widget with a list of items, each with properties ------

            pub struct Item {
                pub label: api::Value<String>,
                pub price: api::Value<u64>,
                pub qty: api::Value<u64>,
                pub selected: api::Value<bool>,
            }

            pub struct ListWidget {
                pub items: Vec<Item>,
                pub counters: ListCounters,
                _subs: Vec<DropResource>,
            }

            /// A selectable list: every row renders from its own properties, and
            /// two independent aggregates (a total and a selection toolbar) span
            /// the whole list.
            pub fn setup_list() -> ListWidget {
                let counters = ListCounters::default();

                let items: Vec<Item> = (0..ITEMS)
                    .map(|i| Item {
                        label: api::Value::new(format!("Item {i}")),
                        price: api::Value::new(100 + i as u64),
                        qty: api::Value::new(1),
                        selected: api::Value::new(i == 0),
                    })
                    .collect();

                let mut subs = Vec::new();
                let mut line_totals = Vec::new();

                for item in &items {
                    let line_total = api::Computed::from({
                        let price = item.price.clone();
                        let qty = item.qty.clone();
                        move |ctx| price.get(ctx) * qty.get(ctx)
                    });

                    let row = api::Computed::from({
                        let label = item.label.clone();
                        let qty = item.qty.clone();
                        let selected = item.selected.clone();
                        let line_total = line_total.clone();
                        let runs = counters.row.clone();
                        move |ctx| {
                            runs.hit();
                            let mark = if selected.get(ctx) { "[x]" } else { "[ ]" };
                            format!(
                                "{mark} {} x{} = {}",
                                label.get(ctx),
                                qty.get(ctx),
                                line_total.get(ctx)
                            )
                        }
                    });

                    subs.push(row.subscribe(|_| {}));
                    line_totals.push(line_total);
                }

                let total = api::Computed::from({
                    let line_totals = line_totals.clone();
                    let runs = counters.total.clone();
                    move |ctx| {
                        runs.hit();
                        line_totals.iter().map(|line| line.get(ctx)).sum::<u64>()
                    }
                });

                let footer = api::Computed::from({
                    let total = total.clone();
                    let runs = counters.footer.clone();
                    move |ctx| {
                        runs.hit();
                        format!("Total: {}", total.get(ctx))
                    }
                });
                subs.push(footer.subscribe(|_| {}));

                let selected_flags: Vec<_> =
                    items.iter().map(|item| item.selected.clone()).collect();

                let selected_count = api::Computed::from({
                    let runs = counters.selected_count.clone();
                    move |ctx| {
                        runs.hit();
                        selected_flags.iter().filter(|flag| flag.get(ctx)).count()
                    }
                });

                let any_selected = api::Computed::from({
                    let selected_count = selected_count.clone();
                    let runs = counters.any_selected.clone();
                    move |ctx| {
                        runs.hit();
                        selected_count.get(ctx) > 0
                    }
                });

                let toolbar = api::Computed::from({
                    let any_selected = any_selected.clone();
                    let runs = counters.toolbar.clone();
                    move |ctx| {
                        runs.hit();
                        if any_selected.get(ctx) {
                            "Delete (enabled)".to_string()
                        } else {
                            "Delete (disabled)".to_string()
                        }
                    }
                });
                subs.push(toolbar.subscribe(|_| {}));

                ListWidget {
                    items,
                    counters,
                    _subs: subs,
                }
            }

            /// One transaction repricing the whole list, like a currency switch.
            pub fn reprice_all(widget: &ListWidget) {
                api::transaction(|ctx| {
                    for item in &widget.items {
                        let price = item.price.get(ctx);
                        item.price.set(price + 1);
                    }
                });
            }

            // -- 2. a dashboard with statistics over multiple websites -------

            pub struct Site {
                pub visits: api::Value<u64>,
                pub errors: api::Value<u64>,
                pub latency: api::Value<u64>,
            }

            pub struct Dashboard {
                pub sites: Vec<Site>,
                pub counters: DashboardCounters,
                pub banner_text: Rc<RefCell<String>>,
                _subs: Vec<DropResource>,
            }

            /// Per-site metrics roll up into a health status, and the statuses
            /// roll up into a single alert banner.
            pub fn setup_dashboard() -> Dashboard {
                let counters = DashboardCounters::default();

                let sites: Vec<Site> = (0..SITES)
                    .map(|i| Site {
                        visits: api::Value::new(10_000),
                        errors: api::Value::new(if i == 0 { 5 } else { 1 }),
                        latency: api::Value::new(100 + (i as u64 % 50)),
                    })
                    .collect();

                let mut subs = Vec::new();
                let mut healths = Vec::new();
                let mut visit_counts = Vec::new();

                for (i, site) in sites.iter().enumerate() {
                    // Errors per thousand requests.
                    let error_rate = api::Computed::from({
                        let visits = site.visits.clone();
                        let errors = site.errors.clone();
                        let runs = counters.error_rate.clone();
                        move |ctx| {
                            runs.hit();
                            errors.get(ctx) * 1000 / visits.get(ctx).max(1)
                        }
                    });

                    let health = api::Computed::from({
                        let error_rate = error_rate.clone();
                        let latency = site.latency.clone();
                        let runs = counters.health.clone();
                        move |ctx| {
                            runs.hit();
                            let rate = error_rate.get(ctx);
                            if rate > 100 {
                                Health::Down
                            } else if rate > 20 || latency.get(ctx) > 500 {
                                Health::Warn
                            } else {
                                Health::Ok
                            }
                        }
                    });

                    let row = api::Computed::from({
                        let health = health.clone();
                        let latency = site.latency.clone();
                        let runs = counters.row.clone();
                        move |ctx| {
                            runs.hit();
                            format!("site{i}: {:?} {}ms", health.get(ctx), latency.get(ctx))
                        }
                    });
                    subs.push(row.subscribe(|_| {}));

                    healths.push(health);
                    visit_counts.push(site.visits.clone());
                }

                let total_visits = api::Computed::from({
                    let runs = counters.total_visits.clone();
                    move |ctx| {
                        runs.hit();
                        visit_counts
                            .iter()
                            .map(|visits| visits.get(ctx))
                            .sum::<u64>()
                    }
                });

                // Displayed rounded, so single-request ticks never reach the DOM.
                let traffic_label = api::Computed::from({
                    let total_visits = total_visits.clone();
                    let runs = counters.traffic_label.clone();
                    move |ctx| {
                        runs.hit();
                        format!("{}k visits", total_visits.get(ctx) / 1000)
                    }
                });
                subs.push(traffic_label.subscribe(|_| {}));

                let unhealthy = api::Computed::from({
                    let healths = healths.clone();
                    let runs = counters.unhealthy.clone();
                    move |ctx| {
                        runs.hit();
                        healths
                            .iter()
                            .filter(|health| health.get(ctx) != Health::Ok)
                            .count()
                    }
                });

                let alert = api::Computed::from({
                    let unhealthy = unhealthy.clone();
                    let runs = counters.alert.clone();
                    move |ctx| {
                        runs.hit();
                        unhealthy.get(ctx) > 0
                    }
                });

                let banner = api::Computed::from({
                    let alert = alert.clone();
                    let runs = counters.banner.clone();
                    move |ctx| {
                        runs.hit();
                        if alert.get(ctx) {
                            "Degraded".to_string()
                        } else {
                            "All systems operational".to_string()
                        }
                    }
                });

                let banner_text = Rc::new(RefCell::new(String::new()));
                subs.push(banner.subscribe({
                    let banner_text = banner_text.clone();
                    move |text| *banner_text.borrow_mut() = text
                }));

                Dashboard {
                    sites,
                    counters,
                    banner_text,
                    _subs: subs,
                }
            }

            /// One poll cycle: fresh traffic everywhere, and one site breaking.
            pub fn poll_with_incident(dashboard: &Dashboard, broken: usize) {
                api::transaction(|ctx| {
                    for site in &dashboard.sites {
                        let visits = site.visits.get(ctx);
                        site.visits.set(visits + 100);
                    }
                    dashboard.sites[broken].errors.set(5_000);
                });
            }

            // -- 3. a rich text editor ---------------------------------------

            pub struct Block {
                pub text: api::Value<String>,
                pub bold: api::Value<bool>,
                pub italic: api::Value<bool>,
            }

            pub struct Editor {
                pub blocks: Vec<Block>,
                /// Caret as (block index, offset in block).
                pub caret: api::Value<(usize, usize)>,
                pub counters: EditorCounters,
                _subs: Vec<DropResource>,
            }

            /// Blocks render independently, a status bar counts words across the
            /// whole document, and the format toolbar follows the caret.
            pub fn setup_editor() -> Editor {
                let counters = EditorCounters::default();

                let blocks: Vec<Block> = (0..BLOCKS)
                    .map(|i| Block {
                        text: api::Value::new(
                            "The quick brown fox jumps over the lazy dog".to_string(),
                        ),
                        bold: api::Value::new(false),
                        italic: api::Value::new(i % 7 == 0),
                    })
                    .collect();

                let caret = api::Value::new((3usize, 5usize));

                let mut subs = Vec::new();
                let mut word_counts = Vec::new();

                for block in &blocks {
                    let rendered = api::Computed::from({
                        let text = block.text.clone();
                        let bold = block.bold.clone();
                        let italic = block.italic.clone();
                        let runs = counters.rendered.clone();
                        move |ctx| {
                            runs.hit();
                            let mut html = text.get(ctx);
                            if bold.get(ctx) {
                                html = format!("<b>{html}</b>");
                            }
                            if italic.get(ctx) {
                                html = format!("<i>{html}</i>");
                            }
                            html
                        }
                    });
                    subs.push(rendered.subscribe(|_| {}));

                    let words = api::Computed::from({
                        let text = block.text.clone();
                        let runs = counters.words.clone();
                        move |ctx| {
                            runs.hit();
                            text.get(ctx).split_whitespace().count()
                        }
                    });
                    word_counts.push(words);
                }

                let total_words = api::Computed::from({
                    let word_counts = word_counts.clone();
                    let runs = counters.total_words.clone();
                    move |ctx| {
                        runs.hit();
                        word_counts
                            .iter()
                            .map(|words| words.get(ctx))
                            .sum::<usize>()
                    }
                });

                let status = api::Computed::from({
                    let total_words = total_words.clone();
                    let runs = counters.status.clone();
                    move |ctx| {
                        runs.hit();
                        format!("{} words - {} blocks", total_words.get(ctx), BLOCKS)
                    }
                });
                subs.push(status.subscribe(|_| {}));

                // The caret carries an offset, but only the block index matters
                // to anything downstream.
                let active_block = api::Computed::from({
                    let caret = caret.clone();
                    let runs = counters.active_block.clone();
                    move |ctx| {
                        runs.hit();
                        caret.get(ctx).0
                    }
                });

                let formats: Vec<_> = blocks
                    .iter()
                    .map(|block| (block.bold.clone(), block.italic.clone()))
                    .collect();

                // Dynamic dependency: the toolbar reads the formatting of
                // whichever block the caret is in.
                let toolbar = api::Computed::from({
                    let active_block = active_block.clone();
                    let runs = counters.toolbar.clone();
                    move |ctx| {
                        runs.hit();
                        let (bold, italic) = &formats[active_block.get(ctx)];
                        format!("B={} I={}", bold.get(ctx), italic.get(ctx))
                    }
                });
                subs.push(toolbar.subscribe(|_| {}));

                Editor {
                    blocks,
                    caret,
                    counters,
                    _subs: subs,
                }
            }
        }
    };
}

scenarios!(scenario_old, reactive_old);
scenarios!(scenario_new, reactive);

#[test]
fn list_widget_edit_one_quantity() {
    let old_w = scenario_old::setup_list();
    let new_w = scenario_new::setup_list();

    old_w.counters.reset();
    new_w.counters.reset();

    let old_d = elapsed(|| old_w.items[7].qty.set(3));
    let new_d = elapsed(|| new_w.items[7].qty.set(3));

    report(&format!("list {ITEMS}: qty of one item"), old_d, new_d);

    // The edited row and the money total move; the selection side of the graph
    // never sees the write in either implementation.
    for counters in [&old_w.counters, &new_w.counters] {
        assert_eq!(counters.row.get(), 1, "only the edited row rerenders");
        assert_eq!(counters.total.get(), 1);
        assert_eq!(counters.footer.get(), 1);
        assert_eq!(counters.selected_count.get(), 0);
        assert_eq!(counters.toolbar.get(), 0);
    }
}

#[test]
fn list_widget_second_selection_keeps_toolbar() {
    let old_w = scenario_old::setup_list();
    let new_w = scenario_new::setup_list();

    // Item 0 is already selected, so the toolbar is enabled before and after.
    old_w.counters.reset();
    new_w.counters.reset();

    let old_d = elapsed(|| old_w.items[5].selected.set(true));
    let new_d = elapsed(|| new_w.items[5].selected.set(true));

    report(&format!("list {ITEMS}: select 2nd item"), old_d, new_d);

    for counters in [&old_w.counters, &new_w.counters] {
        assert_eq!(counters.row.get(), 1);
        assert_eq!(counters.selected_count.get(), 1);
        assert_eq!(counters.total.get(), 0, "prices did not change");
    }

    assert_eq!(
        new_w.counters.toolbar.get(),
        0,
        "1 -> 2 selected leaves `any_selected` true, so the toolbar must not rerender"
    );
    assert_eq!(
        old_w.counters.toolbar.get(),
        1,
        "old graph rebuilds the toolbar on any selection change"
    );
}

#[test]
fn list_widget_reprice_all_in_one_transaction() {
    let old_w = scenario_old::setup_list();
    let new_w = scenario_new::setup_list();

    old_w.counters.reset();
    new_w.counters.reset();

    let old_d = elapsed(|| scenario_old::reprice_all(&old_w));
    let new_d = elapsed(|| scenario_new::reprice_all(&new_w));

    report(&format!("list {ITEMS}: reprice all"), old_d, new_d);

    // A transaction is one wave: every row rerenders once, the total once.
    for counters in [&old_w.counters, &new_w.counters] {
        assert_eq!(counters.row.get(), ITEMS);
        assert_eq!(counters.total.get(), 1);
        assert_eq!(counters.footer.get(), 1);
        assert_eq!(counters.toolbar.get(), 0);
    }
}

#[test]
fn dashboard_metric_tick_is_absorbed() {
    let old_d_board = scenario_old::setup_dashboard();
    let new_d_board = scenario_new::setup_dashboard();

    old_d_board.counters.reset();
    new_d_board.counters.reset();

    // A single extra request on one site: the error rate per thousand and the
    // rounded traffic label both stay where they were.
    let old_d = elapsed(|| old_d_board.sites[3].visits.set(10_001));
    let new_d = elapsed(|| new_d_board.sites[3].visits.set(10_001));

    report(&format!("dashboard {SITES}: one tick"), old_d, new_d);

    for counters in [&old_d_board.counters, &new_d_board.counters] {
        assert_eq!(counters.error_rate.get(), 1);
        assert_eq!(counters.total_visits.get(), 1);
        assert_eq!(counters.traffic_label.get(), 1);
    }

    let new_c = &new_d_board.counters;
    assert_eq!(new_c.health.get(), 0, "the error rate did not move");
    assert_eq!(new_c.row.get(), 0);
    assert_eq!(new_c.unhealthy.get(), 0, "no status changed");
    assert_eq!(new_c.banner.get(), 0);

    let old_c = &old_d_board.counters;
    assert_eq!(old_c.health.get(), 1);
    assert_eq!(old_c.row.get(), 1);
    assert_eq!(
        old_c.unhealthy.get(),
        1,
        "old graph rescans every site status after one tick"
    );
    assert_eq!(old_c.banner.get(), 1);
}

#[test]
fn dashboard_incident_reaches_the_banner() {
    let old_board = scenario_old::setup_dashboard();
    let new_board = scenario_new::setup_dashboard();

    for banner_text in [&old_board.banner_text, &new_board.banner_text] {
        assert_eq!(*banner_text.borrow(), "All systems operational");
    }

    old_board.counters.reset();
    new_board.counters.reset();

    let old_d = elapsed(|| scenario_old::poll_with_incident(&old_board, 7));
    let new_d = elapsed(|| scenario_new::poll_with_incident(&new_board, 7));

    report(&format!("dashboard {SITES}: poll + incident"), old_d, new_d);

    for banner_text in [&old_board.banner_text, &new_board.banner_text] {
        assert_eq!(*banner_text.borrow(), "Degraded");
    }
    for counters in [&old_board.counters, &new_board.counters] {
        assert_eq!(counters.error_rate.get(), SITES);
        assert_eq!(counters.unhealthy.get(), 1);
        assert_eq!(counters.banner.get(), 1);
    }

    assert_eq!(
        new_board.counters.health.get(),
        1,
        "only the broken site changed its error rate"
    );
    assert_eq!(new_board.counters.row.get(), 1);
    assert_eq!(old_board.counters.health.get(), SITES);
    assert_eq!(old_board.counters.row.get(), SITES);
}

#[test]
fn editor_caret_move_inside_block() {
    let old_e = scenario_old::setup_editor();
    let new_e = scenario_new::setup_editor();

    old_e.counters.reset();
    new_e.counters.reset();

    // The most frequent event in an editor: the caret moves, still in block 3.
    let old_d = elapsed(|| old_e.caret.set((3, 12)));
    let new_d = elapsed(|| new_e.caret.set((3, 12)));

    report(&format!("editor {BLOCKS}: caret move"), old_d, new_d);

    for counters in [&old_e.counters, &new_e.counters] {
        assert_eq!(counters.active_block.get(), 1);
        assert_eq!(counters.rendered.get(), 0);
        assert_eq!(counters.status.get(), 0);
    }

    assert_eq!(
        new_e.counters.toolbar.get(),
        0,
        "the active block did not change, so the toolbar must not rerender"
    );
    assert_eq!(
        old_e.counters.toolbar.get(),
        1,
        "old graph rebuilds the toolbar on every caret move"
    );
}

#[test]
fn editor_typing_does_not_touch_the_status_bar() {
    let old_e = scenario_old::setup_editor();
    let new_e = scenario_new::setup_editor();

    old_e.counters.reset();
    new_e.counters.reset();

    // Typing inside a word: the block rerenders, the word count does not move.
    let old_d = elapsed(|| old_e.blocks[3].text.change(|text| text.push('!')));
    let new_d = elapsed(|| new_e.blocks[3].text.change(|text| text.push('!')));

    report(&format!("editor {BLOCKS}: one keystroke"), old_d, new_d);

    for counters in [&old_e.counters, &new_e.counters] {
        assert_eq!(counters.rendered.get(), 1, "one block rerenders");
        assert_eq!(counters.words.get(), 1);
        assert_eq!(counters.toolbar.get(), 0);
    }

    assert_eq!(
        new_e.counters.total_words.get(),
        0,
        "the block's word count is unchanged, so the document total is not resummed"
    );
    assert_eq!(new_e.counters.status.get(), 0);
    assert_eq!(
        old_e.counters.total_words.get(),
        1,
        "old graph resums all {BLOCKS} blocks per keystroke"
    );
    assert_eq!(old_e.counters.status.get(), 1);
}
