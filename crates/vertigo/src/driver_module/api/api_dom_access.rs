use crate::{
    dev::command::{CommandForBrowser, JsApiCommand},
    driver_module::api::panic_message::api_panic_message,
    DomId, JsJson,
};

#[derive(Default)]
pub struct DomAccess {
    commands: Vec<JsApiCommand>,
}

impl DomAccess {
    #[must_use]
    pub fn element(mut self, dom_id: DomId) -> Self {
        self.commands.push(JsApiCommand::RootElement {
            dom_id: dom_id.to_u64(),
        });
        self
    }

    #[must_use]
    pub fn root(mut self, name: impl Into<String>) -> Self {
        self.commands.push(JsApiCommand::Root { name: name.into() });
        self
    }

    #[must_use]
    pub fn get(mut self, name: impl Into<String>) -> Self {
        self.commands.push(JsApiCommand::Get {
            property: name.into(),
        });
        self
    }

    #[must_use]
    pub fn set(mut self, name: impl Into<String>, value: JsJson) -> Self {
        self.commands.push(JsApiCommand::Set {
            property: name.into(),
            value,
        });
        self
    }

    #[must_use]
    pub fn call(mut self, name: impl Into<String>, params: Vec<JsJson>) -> Self {
        self.commands.push(JsApiCommand::Call {
            method: name.into(),
            args: params,
        });
        self
    }

    pub fn exec(self) {
        let result = self.fetch();

        if let JsJson::Null = result {
            //ok
        } else {
            let message = format!("Expected null dump={result:?}");
            api_panic_message().show(message);
        }
    }

    pub fn fetch(self) -> JsJson {
        use crate::driver_module::api::api_browser_command::exec_command;

        let command = CommandForBrowser::JsApiCall {
            commands: self.commands,
        };
        exec_command(command)
    }
}
