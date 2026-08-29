use vertigo::{DomNode, bind, css, dom};

use crate::app::sudoku::state::{Cell, number_item::SudokuValue};

pub fn render_cell_value(item_height_size: u32, value: SudokuValue, cell: &Cell) -> DomNode {
    let cell = cell.clone();

    // A given is part of the puzzle, so it gets no delete affordance.
    let delete_component = cell.is_given.render_value_option({
        let cell = cell.clone();
        move |is_given| {
            if is_given {
                return None;
            }

            let on_click = bind!(cell, |_| {
                cell.number.set(None);
            });

            let css_delete = css! {"
                position: absolute;
                top: 3px;
                right: 3px;
                width: 20px;
                height: 20px;
                background-color: #ff000030;
                cursor: pointer;
                font-size: 12px;
                line-height: 12px;

                display: flex;
                align-items: center;
                justify-content: center;
            "};

            Some(dom! {
                <div css={css_delete} on_click={on_click}>
                    "X"
                </div>
            })
        }
    });

    // Givens in black, what the player entered in blue.
    let css_item = cell.is_given.map(move |is_given| {
        let colour = if is_given { "#111" } else { "blue" };
        let weight = if is_given { "bold" } else { "normal" };

        css! {"
            position: relative;
            text-align: center;
            font-size: 30px;
            color: {colour};
            font-weight: {weight};
            height: {item_height_size}px;
            line-height: {item_height_size}px;
        "}
    });

    dom! {
        <div css={css_item}>
            { value.as_u16() }
            {delete_component}
        </div>
    }
}
