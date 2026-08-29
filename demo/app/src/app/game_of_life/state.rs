use std::rc::Rc;
use vertigo::{ClickEvent, Computed, DropResource, Value, get_driver, transaction};

use super::next_generation::next_generation;
use super::patterns::Pattern;

#[derive(Clone)]
pub struct State {
    pub matrix: Rc<Vec<Vec<Value<bool>>>>,
    pub timer: Value<Option<Rc<DropResource>>>,
    pub delay: Value<u32>,
    /// What the delay field currently holds, as typed.
    /// It is parsed once, when Set is pressed - see [`State::accept_new_delay`].
    pub new_delay: Value<String>,
    /// Why the last Set was refused, or `None` if it was accepted.
    pub delay_error: Value<Option<String>>,
    pub year: Value<u32>,
    /// How many cells are alive. Every one of the 8400 `Value<bool>` feeds this.
    pub population: Computed<usize>,
}

impl State {
    const X_LEN: u16 = 120;
    const Y_LEN: u16 = 70;

    const DEFAULT_DELAY: u32 = 50;

    /// Share of the board that [`State::randomize`] fills, in percent.
    const RANDOM_DENSITY: u32 = 35;

    pub fn new() -> Self {
        let matrix = Rc::new(create_matrix(Self::X_LEN, Self::Y_LEN));

        let timer = Value::new(None);
        let delay = Value::new(Self::DEFAULT_DELAY);
        let new_delay = Value::new(Self::DEFAULT_DELAY.to_string());
        let delay_error = Value::new(None);
        let year = Value::new(1);

        let population = Computed::from({
            let matrix = matrix.clone();
            move |context| {
                matrix
                    .iter()
                    .flatten()
                    .filter(|cell| cell.get(context))
                    .count()
            }
        });

        Self {
            matrix,
            timer,
            delay,
            new_delay,
            delay_error,
            year,
            population,
        }
    }

    pub fn on_toggle_timer(&self) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| {
            transaction(|context| {
                let timer = state.timer.get(context);

                if timer.is_some() {
                    state.timer.set(None);
                } else {
                    state.start_timer();
                }
            });
        }
    }

    /// Fill the board with a random soup, roughly [`State::RANDOM_DENSITY`] percent alive.
    pub fn randomize(&self) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| {
            let mut rng = Rng::new();
            log::info!("randomize: filling ~{}% of the board", Self::RANDOM_DENSITY);
            state.set_board(|_, _| rng.next() % 100 < Self::RANDOM_DENSITY);
        }
    }

    /// Put one of the presets on the board.
    pub fn load_pattern(&self, pattern: &'static Pattern) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| {
            log::info!("loading {}", pattern.name);
            let live = pattern.live_cells(Self::X_LEN, Self::Y_LEN);
            state.set_board(|y, x| live.contains(&(y, x)));
        }
    }

    pub fn clear(&self) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| {
            log::info!("clear");
            state.set_board(|_, _| false);
        }
    }

    /// Advance one generation, without needing the timer to be running.
    pub fn step(&self) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| state.advance()
    }

    /// Replace the board and start counting generations again.
    ///
    /// One transaction, so the 8400 writes reach `population` and the grid as a single change
    /// rather than as 8400 of them.
    fn set_board(&self, mut live: impl FnMut(u16, u16) -> bool) {
        transaction(|_| {
            for (y, row) in self.matrix.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    cell.set(live(y as u16, x as u16));
                }
            }

            self.year.set(1);
        });
    }

    fn advance(&self) {
        transaction(|context| {
            self.year.set(self.year.get(context) + 1);
            next_generation(Self::X_LEN, Self::Y_LEN, &self.matrix);
        });
    }

    pub fn start_timer(&self) {
        transaction(|context| {
            let delay = self.delay.get(context);

            log::info!("Setting timer for {delay} ms");

            let timer = get_driver().set_interval(delay, {
                let state = self.clone();
                move || state.advance()
            });

            self.timer.set(Some(Rc::new(timer)));
        })
    }

    /// Parse what is in the delay field and, if it is a number, adopt it.
    pub fn accept_new_delay(&self) -> impl Fn(ClickEvent) + 'static {
        let state = self.clone();
        move |_| {
            transaction(|context| {
                let typed = state.new_delay.get(context);

                let Ok(requested) = typed.trim().parse::<u32>() else {
                    state
                        .delay_error
                        .set(Some(format!("{typed:?} is not a number of milliseconds")));
                    return;
                };

                state.delay_error.set(None);
                state.delay.set(requested);

                if state.timer.get(context).is_some() {
                    state.start_timer();
                }
            });
        }
    }
}

/// A tiny xorshift32, seeded once from the browser.
///
/// Deliberately not one `get_driver().get_random(..)` per cell: the board is 8400 cells and
/// every one of those calls is a round trip out to JS. One seed from the driver, and the rest
/// of the sequence is produced here.
struct Rng(u32);

impl Rng {
    fn new() -> Self {
        // `get_random` is inclusive at both ends. The range starts at 1 because xorshift
        // seeded with zero emits nothing but zeros - which would fill the board with nothing.
        Rng(get_driver().get_random(1, u32::MAX))
    }

    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
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
