use crate::struct_mut::VecMut;

struct Hook {
    before_start: Box<dyn Fn()>,
    after_end: Box<dyn Fn()>,
}

pub struct Hooks {
    list: VecMut<Hook>,
}

impl Hooks {
    pub fn new() -> Hooks {
        Hooks {
            list: VecMut::new(),
        }
    }

    pub fn add(&self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.list.push(Hook {
            before_start,
            after_end,
        });
    }

    pub fn fire_start(&self) {
        self.list.for_each(|hook| {
            (hook.before_start)();
        });
    }

    pub fn fire_end(&self) {
        self.list.for_each(|hook| {
            (hook.after_end)();
        });
    }
}
