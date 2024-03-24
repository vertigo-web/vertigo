use vertigo::Value;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SudokuValue {
    Value1,
    Value2,
    Value3,
    Value4,
    Value5,
    Value6,
    Value7,
    Value8,
    Value9,
}

impl SudokuValue {
    pub fn as_u16(&self) -> u16 {
        match self {
            SudokuValue::Value1 => 1,
            SudokuValue::Value2 => 2,
            SudokuValue::Value3 => 3,
            SudokuValue::Value4 => 4,
            SudokuValue::Value5 => 5,
            SudokuValue::Value6 => 6,
            SudokuValue::Value7 => 7,
            SudokuValue::Value8 => 8,
            SudokuValue::Value9 => 9,
        }
    }

    pub fn variants() -> Vec<SudokuValue> {
        vec![
            SudokuValue::Value1,
            SudokuValue::Value2,
            SudokuValue::Value3,
            SudokuValue::Value4,
            SudokuValue::Value5,
            SudokuValue::Value6,
            SudokuValue::Value7,
            SudokuValue::Value8,
            SudokuValue::Value9,
        ]
    }
}

pub type NumberItem = Value<Option<SudokuValue>>;

// #[derive(Clone)]
// pub struct NumberItem {
//     pub x0: TreeBoxIndex,
//     pub y0: TreeBoxIndex,
//     pub x1: TreeBoxIndex,
//     pub y1: TreeBoxIndex,
//     pub value: Value<Option<SudokuValue>>,
// }

// impl NumberItem {
//     pub fn new(
//         x0: TreeBoxIndex,
//         y0: TreeBoxIndex,
//         x1: TreeBoxIndex,
//         y1: TreeBoxIndex,
//         value: Option<SudokuValue>,
//     ) -> NumberItem {
//         NumberItem {
//             x0,
//             y0,
//             x1,
//             y1,
//             value: Value::new(value),
//         }
//     }
// }
