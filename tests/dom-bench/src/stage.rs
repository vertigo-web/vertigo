//! The mount point every scenario's DOM lives in.

use std::rc::Rc;

use vertigo::{DomNode, Value, dom, store};

use crate::scenes::{dash::DashScene, editor::EditorScene, list::ListScene, probe::ProbeScene};

/// What is currently mounted under `#bench-stage`.
///
/// Each variant is an `Rc` so switching scenes is a pointer write rather than a deep clone
/// of several hundred `Value`s.
#[derive(Clone)]
pub enum Scene {
    Empty,
    Probe(Rc<ProbeScene>),
    List(Rc<ListScene>),
    Editor(Rc<EditorScene>),
    Dash(Rc<DashScene>),
}

/// Identity, not structure.
///
/// A derived `PartialEq` would work - `Value` already compares by graph node id - but it
/// would walk several hundred rows on every stage write, and it would force `PartialEq`
/// onto every field of every scene. `Rc::ptr_eq` answers the only question the stage has:
/// is this the same scene object I am already showing.
impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Scene::Empty, Scene::Empty) => true,
            (Scene::Probe(left), Scene::Probe(right)) => Rc::ptr_eq(left, right),
            (Scene::List(left), Scene::List(right)) => Rc::ptr_eq(left, right),
            (Scene::Editor(left), Scene::Editor(right)) => Rc::ptr_eq(left, right),
            (Scene::Dash(left), Scene::Dash(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}

/// The one stage cell.
///
/// A singleton because `Workload::make` takes no arguments and has to reach it. `#[store]`
/// is vertigo's own thread-local memo; wasm is single-threaded, so this is a process global.
#[store]
pub fn stage() -> Value<Scene> {
    Value::new(Scene::Empty)
}

/// Mounted as a **sibling** of the progress UI, never inside it: the progress
/// `render_value` rebuilds its whole subtree on every update, and the stage has to survive
/// that untouched.
pub fn stage_node() -> DomNode {
    dom! {
        <div id="bench-stage">
            {stage().render_value(render_scene)}
        </div>
    }
}

fn render_scene(scene: Scene) -> DomNode {
    match scene {
        Scene::Empty => dom! { <div id="stage-empty" /> },
        Scene::Probe(scene) => crate::scenes::probe::render(scene),
        Scene::List(scene) => crate::scenes::list::render(scene),
        Scene::Editor(scene) => crate::scenes::editor::render(scene),
        Scene::Dash(scene) => crate::scenes::dash::render(scene),
    }
}

/// Keeps a scene mounted for as long as the benchmark that built it is alive.
///
/// The ordering is the point. `Value::set` on the default graph runs the propagation, then
/// the `on_after_transaction` hook flushes the command buffer, then the JS side applies
/// every command inline - no rAF, no microtask batching. So the scene is already in the
/// real document by the time `mount` returns, and already gone by the time `drop` returns.
/// Nothing needs awaiting, and since the guard is built in `make()` and dropped at the end
/// of `run_one`, no mount or teardown cost can land inside a timed batch.
pub struct StageGuard;

impl StageGuard {
    pub fn mount(scene: Scene) -> StageGuard {
        // Empty first. Two scenes of the same variant are different `Rc`s and would compare
        // unequal anyway, but going through `Empty` makes that a property of this function
        // rather than of `Scene`'s `PartialEq` - and it stops the two trees from ever
        // coexisting, since `render_value` builds the new subtree before dropping the old.
        stage().set(Scene::Empty);
        stage().set(scene);
        StageGuard
    }
}

impl Drop for StageGuard {
    fn drop(&mut self) {
        stage().set(Scene::Empty);
    }
}
