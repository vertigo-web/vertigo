use vertigo::computed::Dependencies;


#[derive(Clone)]
pub struct Transaction {
    dependencies: Dependencies
}

impl Transaction {
    pub fn new(dependencies: &Dependencies) -> Transaction {
        Transaction {
            dependencies: dependencies.clone(),
        }
    }

    pub fn exec<F: FnOnce() -> ()>(&self, fun: F) {
        self.dependencies.transaction(move || {
            fun();
        });
    }
}
