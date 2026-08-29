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

    /// One ASCII digit as a value, for reading the example boards.
    ///
    /// Anything else reads as an empty cell.
    pub fn from_ascii(byte: u8) -> Option<SudokuValue> {
        match byte {
            b'1' => Some(SudokuValue::Value1),
            b'2' => Some(SudokuValue::Value2),
            b'3' => Some(SudokuValue::Value3),
            b'4' => Some(SudokuValue::Value4),
            b'5' => Some(SudokuValue::Value5),
            b'6' => Some(SudokuValue::Value6),
            b'7' => Some(SudokuValue::Value7),
            b'8' => Some(SudokuValue::Value8),
            b'9' => Some(SudokuValue::Value9),
            _ => None,
        }
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
