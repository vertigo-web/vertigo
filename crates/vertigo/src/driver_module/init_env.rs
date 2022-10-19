use std::panic;
use log::{Level, Log, Metadata, Record};
use crate::{ApiImport};

use std::sync::{Once};

static SET_HOOK: Once = Once::new();

pub fn init_env(api: ApiImport) {
    SET_HOOK.call_once(|| {
        let panic_message = api.panic_message;
        init_logger(api);

        panic::set_hook(Box::new(move |info: &panic::PanicInfo<'_>| {
            let message = info.to_string();
            panic_message.show(message);
        }));
    });
}


/// Specify what to be logged
pub struct Config {
    level: Level,
    module_prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: Level::Debug,
            module_prefix: None,
        }
    }
}

/// The log styles
struct Style {
    lvl_trace: String,
    lvl_debug: String,
    lvl_info: String,
    lvl_warn: String,
    lvl_error: String,
    tgt: String,
    args: String,
}

impl Style {
    fn new() -> Style {
        let base = String::from("color: white; padding: 0 3px; background:");
        Style {
            lvl_trace: format!("{} gray;", base),
            lvl_debug: format!("{} blue;", base),
            lvl_info: format!("{} green;", base),
            lvl_warn: format!("{} orange;", base),
            lvl_error: format!("{} darkred;", base),
            tgt: String::from("font-weight: bold; color: inherit"),
            args: String::from("background: inherit; color: inherit"),
        }
    }
}

struct WasmLogger {
    api: ApiImport,
    config: Config,
    style: Style,
}

unsafe impl Send for WasmLogger {}
unsafe impl Sync for WasmLogger {}

impl Log for WasmLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if let Some(ref prefix) = self.config.module_prefix {
            metadata.target().starts_with(prefix)
        } else {
            true
        }
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let style = &self.style;
            let message_separator = " ";

            let s = format!(
                "%c{}%c {}:{}%c{}{}",
                record.level(),
                record.file().unwrap_or_else(|| record.target()),
                record
                    .line()
                    .map_or_else(|| "[Unknown]".to_string(), |line| line.to_string()),
                message_separator,
                record.args(),
            );
            let tgt_style = &style.tgt;
            let args_style = &style.args;

            match record.level() {
                Level::Trace => {
                    self.api.console_debug_4(&s, &style.lvl_trace, tgt_style, args_style);
                },
                Level::Debug => {
                    self.api.console_log_4(&s, &style.lvl_debug, tgt_style, args_style)
                },
                Level::Info => {
                    self.api.console_info_4(&s, &style.lvl_info, tgt_style, args_style)
                }
                Level::Warn => {
                    self.api.console_warn_4(&s, &style.lvl_warn, tgt_style, args_style)
                }
                Level::Error => {
                    self.api.console_error_4(&s, &style.lvl_error, tgt_style, args_style)
                }
            }
        }
    }

    fn flush(&self) {}
}

fn init_logger(api: ApiImport) {
    let config = Config::default();
    let max_level = config.level;
    let wl = WasmLogger {
        api: api.clone(),
        config,
        style: Style::new(),
    };

    match log::set_boxed_logger(Box::new(wl)) {
        Ok(_) => log::set_max_level(max_level.to_level_filter()),
        Err(e) => {
            let message = e.to_string();
            api.show_panic_message(message);
        },
    }
}
