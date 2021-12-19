struct Hook {
    before_start: Box<dyn Fn()>,
    after_end: Box<dyn Fn()>,
}

pub struct Hooks {
    list: Vec<Hook>,
}

impl Hooks {
    pub fn new() -> Hooks {
        Hooks {
            list: Vec::new(),
        }
    }

    pub fn add(&mut self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.list.push(Hook {
            before_start,
            after_end,
        });
    }

    pub fn fire_start(&self) {
        for hook in &self.list {
            let before_start = &hook.before_start;
            before_start();
        }
    }

    pub fn fire_end(&self) {
        for hook in &self.list {
            let after_end = &hook.after_end;
            after_end();
        }
    }
}
