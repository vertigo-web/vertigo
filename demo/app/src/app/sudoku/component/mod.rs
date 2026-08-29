use vertigo::{DomNode, Value, bind, component, css, dom};

use super::state::{Cell, SudokuState, sudoku_square::SudokuSquare, tree_box::TreeBoxIndex};

pub mod render_cell_possible;
pub mod render_cell_value;

#[component]
pub fn Sudoku(state: SudokuState) {
    let wrapper_css = css! {"
        display: flex;
        margin: 10px;
    "};

    dom! {
        <div css={wrapper_css}>
            <ExamplesRender sudoku={&state} />
            <MainRender sudoku={state} />
        </div>
    }
}

#[component]
pub fn MainRender(sudoku: SudokuState) {
    let hints = &sudoku.hints;

    let (group_width, group_height, view1) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::First, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view2) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::First, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view3) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::First, TreeBoxIndex::Last),
        hints,
    );
    let (_, _, view4) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::Middle, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view5) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view6) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last),
        hints,
    );
    let (_, _, view7) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::Last, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view8) = render_group(
        sudoku
            .grid
            .get_from(TreeBoxIndex::Last, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view9) = render_group(
        sudoku.grid.get_from(TreeBoxIndex::Last, TreeBoxIndex::Last),
        hints,
    );

    let width = 3 * group_width + 2 * 2;
    let height = 3 * group_height + 2 * 2;

    let out_css = css! {"
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;

        width: {width}px;
        height: {height}px;

        border: 2px solid blue;
        user-select: none;
    "};

    let css_center = css! {"
        display: flex;
        justify-content: center;
    "};

    dom! {
        <div css={css_center}>
            <div css={out_css}>
                { view1 }
                { view2 }
                { view3 }
                { view4 }
                { view5 }
                { view6 }
                { view7 }
                { view8 }
                { view9 }
            </div>
        </div>
    }
}

fn render_group(group: &SudokuSquare<Cell>, hints: &Value<bool>) -> (u32, u32, DomNode) {
    let (cell_width, cell_height, view1) = render_cell(
        group.get_from(TreeBoxIndex::First, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view2) = render_cell(
        group.get_from(TreeBoxIndex::First, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view3) = render_cell(
        group.get_from(TreeBoxIndex::First, TreeBoxIndex::Last),
        hints,
    );
    let (_, _, view4) = render_cell(
        group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view5) = render_cell(
        group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view6) = render_cell(
        group.get_from(TreeBoxIndex::Middle, TreeBoxIndex::Last),
        hints,
    );
    let (_, _, view7) = render_cell(
        group.get_from(TreeBoxIndex::Last, TreeBoxIndex::First),
        hints,
    );
    let (_, _, view8) = render_cell(
        group.get_from(TreeBoxIndex::Last, TreeBoxIndex::Middle),
        hints,
    );
    let (_, _, view9) = render_cell(
        group.get_from(TreeBoxIndex::Last, TreeBoxIndex::Last),
        hints,
    );

    let border = 1;

    let width = 2 * border + 3 * cell_width;
    let height = 2 * border + 3 * cell_height;

    let out_css = css! {"
        border: {border}px solid black;

        width: {width}px;
        height: {height}px;

        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        flex-shrink: 0;
    "};

    let group = dom! {
        <div css={out_css}>
            { view1 }
            { view2 }
            { view3 }
            { view4 }
            { view5 }
            { view6 }
            { view7 }
            { view8 }
            { view9 }
        </div>
    };

    (width, height, group)
}

fn render_cell(item: &Cell, hints: &Value<bool>) -> (u32, u32, DomNode) {
    let item = item.clone();

    let small_item_width = 24;
    let small_item_height = 24;
    let border = 1;

    let cell_width = 2 * border + 3 * small_item_width;
    let cell_height = 2 * border + 3 * small_item_height;

    let value_view = item.number.render_value({
        let item = item.clone();
        let hints = hints.clone();
        move |value| {
            if let Some(value) = value {
                render_cell_value::render_cell_value(cell_height, value, &item)
            } else {
                render_cell_possible::render_cell_possible(cell_width, &item, &hints)
            }
        }
    });

    let css_wrapper = css! {"
        border: {border}px solid green;
        width: {cell_width}px;
        height: {cell_height}px;
    "};

    let dom = dom! {
        <div css={css_wrapper}>
            { value_view }
        </div>
    };

    (cell_width, cell_height, dom)
}

#[component]
fn ExamplesRender(sudoku: SudokuState) {
    let clear = bind!(sudoku, |_| {
        sudoku.clear();
    });

    let example1 = bind!(sudoku, |_| {
        sudoku.example1();
    });

    let example2 = bind!(sudoku, |_| {
        sudoku.example2();
    });

    let example3 = bind!(sudoku, |_| {
        sudoku.example3();
    });

    // A fixed width rather than one sized to its contents. The status line's text changes
    // with the board, and "Conflict: a digit repeats" is a good deal wider than
    // "36 of 81 filled" - so a panel left to grow pushes the board sideways for as long as
    // the message is up. Long messages wrap instead.
    let css_sudoku_example = css! {"
        border: 1px solid black;
        padding: 10px;
        width: 170px;
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        margin-right: 10px;
    "};

    let css_sudoku_example_button = css! {"
        margin: 5px;
        cursor: pointer;
    "};

    let status = sudoku.status.render_value(|status| {
        let colour = status.colour();

        let css_status = css! {"
            margin: 10px 5px 0 5px;
            color: {colour};
        "};

        dom! {
            <div css={css_status}>
                { status.message() }
            </div>
        }
    });

    dom! {
        <div css={css_sudoku_example}>
            <button css={&css_sudoku_example_button} on_click={clear}>"Clear"</button>
            <button css={&css_sudoku_example_button} on_click={example1}>"Easy"</button>
            <button css={&css_sudoku_example_button} on_click={example2}>"Medium"</button>
            <button css={css_sudoku_example_button} on_click={example3}>"Hard"</button>
            { status }
            <HintsSwitch hints={sudoku.hints} />
        </div>
    }
}

/// The hints toggle: a track with a knob that slides, and a label saying which way it is.
///
/// Built out of two divs rather than an `<input type="checkbox">` because vertigo gives
/// `value` the property treatment when it sets an attribute but nothing else, so a checkbox's
/// `checked` would only ever be its *initial* state and would drift as soon as it was clicked.
/// Two divs and a reactive `css` say what is meant and stay in step.
#[component]
fn HintsSwitch(hints: Value<bool>) {
    let on_click = bind!(hints, |_| {
        hints.change(|on| *on = !*on);
    });

    let css_track = hints.map(|on| {
        let colour = if on { "#3a3" } else { "#bbb" };

        css! {"
            width: 34px;
            height: 18px;
            border-radius: 9px;
            background-color: {colour};
            flex-shrink: 0;
            transition: background-color .15s;
        "}
    });

    let css_knob = hints.map(|on| {
        let offset = if on { "18px" } else { "2px" };

        // `margin-left` on its own rather than inside a `margin` shorthand: the `css!`
        // interpolation does not reach a placeholder buried in a multi-value shorthand, and
        // silently leaves it as literal text.
        css! {"
            width: 14px;
            height: 14px;
            margin-top: 2px;
            margin-left: {offset};
            border-radius: 50%;
            background-color: white;
            transition: margin-left .15s;
        "}
    });

    let label = hints.map(|on| if on { "Hints: on" } else { "Hints: off" });

    let css_row = css! {"
        display: flex;
        align-items: center;
        gap: 8px;
        margin: 10px 5px 0 5px;
        cursor: pointer;
        user-select: none;
    "};

    dom! {
        <div css={css_row} {on_click}>
            <div css={css_track}>
                <div css={css_knob} />
            </div>
            { label }
        </div>
    }
}
