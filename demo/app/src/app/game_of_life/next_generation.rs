use vertigo::{get_driver, Value};

enum Offset {
    Sub,  // -1
    None, // 0
    Add,  // 1
}

fn modulo(base: u16, current: u16, offset: Offset) -> usize {
    let result = match offset {
        Offset::Sub => {
            if current == 0 {
                base - 1
            } else {
                current - 1
            }
        }
        Offset::None => current,
        Offset::Add => {
            if current + 1 == base {
                0
            } else {
                current + 1
            }
        }
    };

    result as usize
}

#[test]
fn test_mod() {
    assert_eq!(modulo(10, 0, Offset::Sub), 9);
    assert_eq!(modulo(10, 0, Offset::None), 0);
    assert_eq!(modulo(10, 0, Offset::Add), 1);

    assert_eq!(modulo(10, 8, Offset::Sub), 7);
    assert_eq!(modulo(10, 8, Offset::None), 8);
    assert_eq!(modulo(10, 8, Offset::Add), 9);

    assert_eq!(modulo(10, 9, Offset::Sub), 8);
    assert_eq!(modulo(10, 9, Offset::None), 9);
    assert_eq!(modulo(10, 9, Offset::Add), 0);
}

fn next_life(current_life: bool, neighbours: usize) -> bool {
    if neighbours == 3 {
        return true;
    }

    current_life && neighbours == 2
}

pub fn next_generation(x_count: u16, y_count: u16, matrix: &[Vec<Value<bool>>]) {
    let mut next_generation: Vec<Vec<bool>> = {
        let mut matrix = Vec::new();

        for _ in 0..y_count {
            let row = vec![false; x_count.into()];
            matrix.push(row);
        }

        matrix
    };

    get_driver().transaction(|| {
        for y in 0..y_count {
            for x in 0..x_count {
                let x_prev = modulo(x_count, x, Offset::Sub);
                let x_curr = modulo(x_count, x, Offset::None);
                let x_next = modulo(x_count, x, Offset::Add);

                let y_prev = modulo(y_count, y, Offset::Sub);
                let y_curr = modulo(y_count, y, Offset::None);
                let y_next = modulo(y_count, y, Offset::Add);

                let mut neighbours = 0;

                //prev row
                if matrix[y_prev][x_prev].get() {
                    neighbours += 1;
                }

                if matrix[y_prev][x_curr].get() {
                    neighbours += 1;
                }

                if matrix[y_prev][x_next].get() {
                    neighbours += 1;
                }

                //current row
                if matrix[y_curr][x_prev].get() {
                    neighbours += 1;
                }

                let current_life = matrix[y_curr][x_curr].get();

                if matrix[y_curr][x_next].get() {
                    neighbours += 1;
                }

                //next row
                if matrix[y_next][x_prev].get() {
                    neighbours += 1;
                }

                if matrix[y_next][x_curr].get() {
                    neighbours += 1;
                }

                if matrix[y_next][x_next].get() {
                    neighbours += 1;
                }

                next_generation[y_curr][x_curr] = next_life(current_life, neighbours);
            }
        }

        for y in 0..y_count as usize {
            for x in 0..x_count as usize {
                matrix[y][x].set_value_and_compare(next_generation[y][x]);
            }
        }
    });
}
