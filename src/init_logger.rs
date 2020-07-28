use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            policy::compound::{
                roll::delete::DeleteRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::{cmp, path::Path};

pub fn init_logger<P: AsRef<Path>>(
    verbosity: i32,
    log_path: Option<P>,
    show_line_numbers: bool,
    progress_bar: Option<indicatif::ProgressBar>,
) -> Result<(), SetLoggerError> {
    let log_config = Config::builder();
    let log_root = Root::builder();

    // Add Console Logger

    let (log_config, log_root) = {
        let pattern = if show_line_numbers {
            "{h({l})} {f}:{L} - {m}\n"
        } else {
            "{h({l})} - {m}\n"
        };

        let encoder = Box::new(PatternEncoder::new(pattern));

        let console_log_level = match cmp::max(0, verbosity) {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };

        // TODO: DRY refactor
        let appender = if let Some(progress_bar) = progress_bar {
            let inner_appender = Box::new(
                crate::log4rs_progress::ProgressAppender::builder()
                    .encoder(encoder)
                    .progress_bar(progress_bar)
                    .build(),
            );

            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_log_level)))
                .build("stderr", inner_appender)
        } else {
            let inner_appender = Box::new(
                ConsoleAppender::builder()
                    .encoder(encoder)
                    .target(Target::Stderr)
                    .build(),
            );

            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_log_level)))
                .build("stderr", inner_appender)
        };

        (log_config.appender(appender), log_root.appender("stderr"))
    };

    // Add File Logger

    let (log_config, log_root) = match log_path {
        Some(log_path) => {
            // SizeTrigger: Trigger at # of bytes
            // DeleteRoller: Delete on trigger
            let policy = Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(10_000_000)),
                Box::new(DeleteRoller::new()),
            ));

            let log_file = RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S %Z)} {l} {M} {f}:{L} - {m}\n",
                )))
                .build(log_path, policy)
                .expect("could not build file logger");
            (
                log_config.appender(Appender::builder().build("logfile", Box::new(log_file))),
                log_root.appender("logfile"),
            )
        }
        None => (log_config, log_root),
    };

    // Filter out some modules
    let log_config = log_config
        .logger(Logger::builder().build("hyper", LevelFilter::Info))
        .logger(Logger::builder().build("reqwest", LevelFilter::Info))
        .logger(Logger::builder().build("mio", LevelFilter::Info))
        .logger(Logger::builder().build("want", LevelFilter::Info));

    // Build logger

    let log_config = log_config
        .build(log_root.build(LevelFilter::Trace))
        .expect("could not build log config");

    // Init log4rs

    let _handle = log4rs::init_config(log_config)?;

    Ok(())
}
