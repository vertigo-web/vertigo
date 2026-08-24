//! A statistics dashboard: N sites, each with a latency reading and an up/down flag, plus
//! an alert banner that appears only while something is down.

use std::rc::Rc;

use vertigo::{Computed, DomNode, Value, dom, transaction};

pub const SITES: u32 = 200;

pub struct SiteState {
    /// Pre-formatted and fixed width, so a tick never changes the string's size.
    pub latency: Value<String>,
    pub down: Value<bool>,
}

pub struct DashScene {
    pub sites: Vec<SiteState>,
    /// Deliberately depends on `down` and not on `latency`.
    ///
    /// An average latency would be more realistic, but then every tick would drag the
    /// header along with it and the per-site cost would hide behind an aggregate. Keeping
    /// the aggregate on the status flag is what makes `dash-tick-one` measure exactly one
    /// site's text update.
    pub down_count: Computed<u32>,
    /// Rendered through `render_value_option`, so a status change mounts and unmounts a real
    /// subtree rather than toggling an attribute.
    pub banner: Computed<Option<String>>,
    /// Latency readings cycled through by the tick workloads.
    pub latencies: Vec<String>,
}

pub fn build() -> Rc<DashScene> {
    let sites: Vec<SiteState> = (0..SITES)
        .map(|index| SiteState {
            latency: Value::new(format!("{:04}ms", 100 + index % 50)),
            down: Value::new(false),
        })
        .collect();

    let downs: Vec<Value<bool>> = sites.iter().map(|site| site.down.clone()).collect();

    let down_count =
        Computed::from(move |ctx| downs.iter().filter(|down| down.get(ctx)).count() as u32);

    let banner = down_count.map(|count| match count {
        0 => None,
        count => Some(format!("{count} site(s) degraded")),
    });

    Rc::new(DashScene {
        sites,
        down_count,
        banner,
        latencies: (0..8).map(|slot| format!("{:04}ms", 200 + slot)).collect(),
    })
}

pub fn render(scene: Rc<DashScene>) -> DomNode {
    let rows: Vec<DomNode> = scene
        .sites
        .iter()
        .enumerate()
        .map(|(index, site)| {
            let class = site
                .down
                .map(|down| if down { "site down" } else { "site" }.to_string());
            dom! {
                <div class={class}>
                    <span class="n">{index}</span>
                    <span class="ms">{site.latency.clone()}</span>
                </div>
            }
        })
        .collect();

    let banner = scene
        .banner
        .render_value_option(|text| text.map(|text| dom! { <div id="dash-banner">{text}</div> }));

    let total = scene.down_count.map(|count| format!("{count} down"));

    dom! {
        <div id="stage-dash">
            {banner}
            <div id="dash-total">{total}</div>
            <div id="dash-rows">{..rows}</div>
        </div>
    }
}

/// Read by the benchmark's checksum, so the whole scene stays observable.
pub fn total_len(scene: &DashScene) -> u64 {
    transaction(|ctx| {
        u64::from(scene.down_count.get(ctx)) * 1_000_000
            + scene
                .sites
                .iter()
                .map(|site| site.latency.get(ctx).len() as u64)
                .sum::<u64>()
    })
}
