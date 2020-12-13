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

pub type NumberItem = Value<Option<SudokuValue>>;

pub fn create_number_item(deps: &Dependencies, value: Option<SudokuValue>) -> Value<Option<SudokuValue>> {
    deps.newValue(value)
}
