//! A rich text editor: N blocks, each with text and a bold flag, plus a caret and a
//! toolbar that follows it.

use std::{collections::HashMap, rc::Rc};

use vertigo::{Computed, DomNode, DomText, EmbedDom, Value, dom, render::render_list, transaction};

pub const BLOCKS: u32 = 300;
/// Length of one formatting run. The caret sweeps inside a single run, so the block it is
/// in never changes and the toolbar must not re-render.
pub const RUN_LEN: u32 = 4_096;

/// How a block's text reaches its DOM text node. The entire difference between
/// `editor-keystroke-embed` and `editor-keystroke-patch`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextMode {
    /// The ordinary `{value}` interpolation in `dom!`.
    Embed,
    /// [`DomText::new_computed`].
    Patch,
}

pub struct BlockState {
    pub text: Value<String>,
    pub bold: Value<bool>,
}

pub struct EditorScene {
    pub order: Value<Vec<u32>>,
    pub blocks: Rc<HashMap<u32, BlockState>>,
    pub keys: Vec<u32>,
    /// A block that starts outside the document, already built, so the insert workload only
    /// mutates `order` inside the timed loop.
    pub spare: u32,
    pub mode: TextMode,
    /// Character offset of the caret, within block zero.
    pub caret: Value<u32>,
    /// What the toolbar shows: the bold flag of whichever block the caret is in.
    ///
    /// The chain is `caret` -> `active_block` -> here. A caret move really does propagate,
    /// but `active_block` is `offset / RUN_LEN`, so while the caret stays inside one run it
    /// recomputes to the same value and the equality cutoff stops there. That is what
    /// `editor-caret-move` measures: a live reactive dependency that must still cost zero
    /// DOM commands.
    pub bold_at_caret: Computed<bool>,
    /// Two texts of equal length - a growing string would make late iterations cost more
    /// than early ones.
    pub texts: [String; 2],
}

pub fn build(mode: TextMode) -> Rc<EditorScene> {
    let spare = BLOCKS;
    let mut blocks = HashMap::new();

    for key in 0..=spare {
        blocks.insert(
            key,
            BlockState {
                text: Value::new(format!(
                    "The quick brown fox jumps over the lazy dog, paragraph {key:03}"
                )),
                bold: Value::new(key % 7 == 0),
            },
        );
    }
    let blocks = Rc::new(blocks);

    let caret = Value::new(0u32);
    let active_block = caret.map(|offset| offset / RUN_LEN);

    let bold_at_caret = Computed::from({
        let blocks = blocks.clone();
        let active_block = active_block.clone();
        move |ctx| {
            let index = active_block.get(ctx);
            blocks
                .get(&index)
                .map(|block| block.bold.get(ctx))
                .unwrap_or(false)
        }
    });

    Rc::new(EditorScene {
        order: Value::new((0..BLOCKS).collect()),
        blocks,
        keys: (0..BLOCKS).collect(),
        spare,
        mode,
        caret,
        bold_at_caret,
        texts: [
            "The quick brown fox jumps over the lazy dog, paragraph aaa".to_string(),
            "The quick brown fox jumps over the lazy dog, paragraph bbb".to_string(),
        ],
    })
}

/// The one line the whole A/B exists for.
///
/// `Embed` goes through `EmbedDom for Value<T>` and `render_value`, which throws the old
/// text node away and builds a new one every time the value changes - `CreateText`,
/// `InsertBefore`, `RemoveText` - plus one permanent marker comment per block, created at
/// mount. `Patch` subscribes the same value straight to the existing node: one `UpdateText`,
/// no marker, no node churn, no id allocation.
///
/// `DomText::new_computed` is public and used nowhere in the repo. This pair is the argument
/// for either promoting it or teaching `EmbedDom` to take the same path.
fn block_text(text: &Value<String>, mode: TextMode) -> DomNode {
    match mode {
        TextMode::Embed => text.clone().embed(),
        TextMode::Patch => DomText::new_computed(text.clone()).into(),
    }
}

pub fn render(scene: Rc<EditorScene>) -> DomNode {
    let blocks = scene.blocks.clone();
    let mode = scene.mode;

    let body = render_list(
        &scene.order,
        |key| *key,
        move |key| {
            let key = transaction(|ctx| key.get(ctx));
            let Some(block) = blocks.get(&key) else {
                return dom! { <p class="missing" /> };
            };
            // `AttrValue` has no `From<Value<bool>>`, so the flag is mapped to a class.
            let class = block
                .bold
                .map(|bold| if bold { "p b" } else { "p" }.to_string());
            dom! {
                <p class={class}>{block_text(&block.text, mode)}</p>
            }
        },
    );

    let toolbar_class = scene
        .bold_at_caret
        .map(|bold| if bold { "tb on" } else { "tb" }.to_string());

    dom! {
        <div id="stage-editor">
            <div id="editor-toolbar" class={toolbar_class} />
            <div id="editor-body">{body}</div>
        </div>
    }
}
