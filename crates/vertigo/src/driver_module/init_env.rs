use log::{Level, Log, Metadata, Record};
use std::panic;

use std::sync::Once;

use crate::{
    dev::command::ConsoleLogLevel,
    driver_module::api::{api_browser_command, api_panic_message},
};

static SET_HOOK: Once = Once::new();

pub fn init_env() {
    SET_HOOK.call_once(|| {
        init_logger();

        panic::set_hook(Box::new(move |info: &panic::PanicHookInfo<'_>| {
            let message = info.to_string();
            api_panic_message().show(message);
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
            lvl_trace: format!("{base} gray;"),
            lvl_debug: format!("{base} blue;"),
            lvl_info: format!("{base} green;"),
            lvl_warn: format!("{base} orange;"),
            lvl_error: format!("{base} darkred;"),
            tgt: String::from("font-weight: bold; color: inherit"),
            args: String::from("background: inherit; color: inherit"),
        }
    }
}

struct WasmLogger {
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
                    api_browser_command().console_log(
                        ConsoleLogLevel::Debug,
                        &s,
                        &style.lvl_trace,
                        tgt_style,
                        args_style,
                    );
                }
                Level::Debug => api_browser_command().console_log(
                    ConsoleLogLevel::Log,
                    &s,
                    &style.lvl_debug,
                    tgt_style,
                    args_style,
                ),
                Level::Info => api_browser_command().console_log(
                    ConsoleLogLevel::Info,
                    &s,
                    &style.lvl_info,
                    tgt_style,
                    args_style,
                ),
                Level::Warn => api_browser_command().console_log(
                    ConsoleLogLevel::Warn,
                    &s,
                    &style.lvl_warn,
                    tgt_style,
                    args_style,
                ),
                Level::Error => api_browser_command().console_log(
                    ConsoleLogLevel::Error,
                    &s,
                    &style.lvl_error,
                    tgt_style,
                    args_style,
                ),
            }
        }
    }

    fn flush(&self) {}
}

fn init_logger() {
    let config = Config::default();
    let max_level = config.level;
    let wl = WasmLogger {
        config,
        style: Style::new(),
    };

    match log::set_boxed_logger(Box::new(wl)) {
        Ok(_) => log::set_max_level(max_level.to_level_filter()),
        Err(e) => {
            let message = e.to_string();
            api_panic_message().show(message);
        }
    }
}
