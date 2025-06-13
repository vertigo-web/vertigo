use std::rc::Rc;
use vertigo::{get_driver, transaction, DropResource, Value};

use super::next_generation::next_generation;

#[derive(Clone)]
pub struct State {
    pub matrix: Rc<Vec<Vec<Value<bool>>>>,
    pub timer: Value<Option<Rc<DropResource>>>,
    pub delay: Value<u32>,
    pub new_delay: Value<u32>,
    pub year: Value<u32>,
}

impl State {
    const X_LEN: u16 = 120;
    const Y_LEN: u16 = 70;

    pub fn new() -> Self {
        let matrix = Rc::new(create_matrix(Self::X_LEN, Self::Y_LEN));

        let timer = Value::new(None);
        let delay = Value::new(150);
        let new_delay = Value::new(150);
        let year = Value::new(1);

        Self {
            matrix,
            timer,
            delay,
            new_delay,
            year,
        }
    }

    pub fn randomize(&self) -> impl Fn() {
        let matrix = self.matrix.clone();

        move || {
            log::info!("random ...");

            transaction(|_| {
                for (y, row) in matrix.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        let new_value: bool = (y * 2 + (x + 4)) % 2 == 0;
                        cell.set(new_value);

                        if x as u16 == Self::X_LEN / 2 && y as u16 == Self::Y_LEN / 2 {
                            cell.set(false);
                        }
                    }
                }
            });
        }
    }

    pub fn start_timer(&self) {
        transaction(|context| {
            let delay = self.delay.get(context);
            let matrix = self.matrix.clone();
            let state = self.clone();

            log::info!("Setting timer for {delay} ms");

            let timer = get_driver().set_interval(delay, {
                move || {
                    transaction(|context| {
                        let current = state.year.get(context);
                        state.year.set(current + 1);

                        next_generation(Self::X_LEN, Self::Y_LEN, &matrix)
                    })
                }
            });

            self.timer.set_force(Some(Rc::new(timer)));
        })
    }

    pub fn accept_new_delay(&self) -> impl Fn() {
        let state = self.clone();

        move || {
            transaction(|context| {
                state.delay.set(state.new_delay.get(context));

                if state.timer.get(context).is_some() {
                    state.start_timer();
                }
            });
        }
    }
}

fn create_matrix_row(x_count: u16) -> Vec<Value<bool>> {
    let mut row = Vec::with_capacity(x_count.into());

    for _ in 0..x_count {
        row.push(Value::new(false));
    }

    row
}

fn create_matrix(x_count: u16, y_count: u16) -> Vec<Vec<Value<bool>>> {
    let mut matrix = Vec::with_capacity(y_count.into());

    for _ in 0..y_count {
        matrix.push(create_matrix_row(x_count));
    }

    matrix
}
