use virtualdom::computed::Value::Value;
use virtualdom::computed::Dependencies::Dependencies;

//export type SudokuValue = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;

//export const getIteratorByAllSudokuValue = (): Array<SudokuValue> => [1, 2, 3, 4, 5, 6, 7, 8, 9];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
    pub fn to_u16(&self) -> u16 {
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
        vec!(
            SudokuValue::Value1,
            SudokuValue::Value2,
            SudokuValue::Value3,

            SudokuValue::Value4,
            SudokuValue::Value5,
            SudokuValue::Value6,

            SudokuValue::Value7,
            SudokuValue::Value8,
            SudokuValue::Value9,
        )
    }}

pub type NumberItem = Value<Option<SudokuValue>>;

pub fn create_number_item(deps: &Dependencies, value: Option<SudokuValue>) -> Value<Option<SudokuValue>> {
    deps.newValue(value)
}
