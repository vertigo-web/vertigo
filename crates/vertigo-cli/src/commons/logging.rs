use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

use crate::Command;

pub(crate) fn setup_logging(command: &Command) {
    let use_local_time = match command {
        Command::Build(opts) => opts.common.log_local_time.unwrap_or(false),
        Command::New(_) => false,
        Command::Serve(opts) => opts.common.log_local_time.unwrap_or(false),
        Command::Watch(opts) => opts.common.log_local_time.unwrap_or(true),
    };

    let mut builder = Builder::new();

    builder
        .filter_level(LevelFilter::Info)
        .parse_env("RUST_LOG")
        .filter(Some("cranelift_codegen"), LevelFilter::Warn)
        .filter(Some("wasmtime_cranelift::compiler"), LevelFilter::Warn);

    if use_local_time {
        builder.format(move |buf, record| {
            // Mimic default formatter but with local time
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{now} {level_style}{:5}{level_style:#} {}] {}",
                record.level(),
                record.module_path().unwrap_or_default(),
                record.args()
            )
        });
    }

    builder.init();
}
