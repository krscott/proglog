use indicatif::ProgressStyle;
use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::delete::DeleteRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Handle,
};
use proglog::{ProgLog, ProgressAppender};
use std::thread;
use std::time::Duration;

pub fn init_log4rs(progress_appender: ProgressAppender) -> Result<Handle, SetLoggerError> {
    let console_appender = Appender::builder()
        .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
        .build("stderr", Box::new(progress_appender));

    // SizeTrigger: Trigger at # of bytes
    // DeleteRoller: Delete on trigger
    let file_policy = Box::new(CompoundPolicy::new(
        Box::new(SizeTrigger::new(10_000_000)),
        Box::new(DeleteRoller::new()),
    ));

    let file_appender = Appender::builder().build(
        "logfile",
        Box::new(
            RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S %Z)} {l} {M} {f}:{L} - {m}\n",
                )))
                .build(String::from("example.log"), file_policy)
                .expect("could not build file logger"),
        ),
    );

    let log_config = Config::builder()
        .appender(console_appender)
        .appender(file_appender)
        // (Here for config building demonstration. Not actually used in this example.)
        .logger(Logger::builder().build("hyper", LevelFilter::Info))
        .logger(Logger::builder().build("reqwest", LevelFilter::Info))
        .logger(Logger::builder().build("mio", LevelFilter::Info))
        .logger(Logger::builder().build("want", LevelFilter::Info));

    let log_root = Root::builder().appender("stderr").appender("logfile");

    log4rs::init_config(
        log_config
            .build(log_root.build(LevelFilter::Trace))
            .expect("could not build log config"),
    )
}

fn logall(msg: &str) {
    log::error!("error {}", msg);
    log::warn!("warn {}", msg);
    log::info!("info {}", msg);
    log::debug!("debug {}", msg);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Based on https://github.com/mitsuhiko/indicatif/blob/main/examples/multi.rs

    let (mut plog, progress_appender) = ProgLog::builder()
        .style(
            ProgressStyle::default_bar()
                .template("{elapsed_precise} [{wide_bar}] {pos:>7}/{len:7} {eta} {msg:40}")
                .progress_chars("=> "),
        )
        .encoder(Box::new(PatternEncoder::new("{h({l})} {f}:{L} - {m}\n")))
        .build();

    let _handle = init_log4rs(progress_appender)?;

    for j in 0..2 {
        let pb = plog.add_with_length(128);

        logall(&format!("Loop: {}", j));
        thread::sleep(Duration::from_millis(1000));

        let _ = thread::spawn(move || {
            for i in 0..128 {
                pb.set_message(&format!("item #{}", i + 1));
                pb.inc(1);

                if i % 40 == 0 {
                    logall(&format!("thread 1 {}", i));
                }

                thread::sleep(Duration::from_millis(15));
            }
            pb.finish_with_message("done");
        });

        let pb = plog.add_with_length(128);
        let _ = thread::spawn(move || {
            for _ in 0..2 {
                pb.set_position(0);
                for i in 0..128 {
                    pb.set_message(&format!("item #{}", i + 1));
                    pb.inc(1);

                    if i % 80 == 0 {
                        logall(&format!("thread 2 {}", i));
                    }

                    thread::sleep(Duration::from_millis(4));
                }
            }
            pb.finish_with_message("done");
        });

        let pb = plog.add_with_length(1024);
        let _ = thread::spawn(move || {
            for i in 0..124 {
                pb.set_message(&format!("item #{}", i + 1));
                pb.inc(1);

                if i % 100 == 0 {
                    logall(&format!("thread 3 {}", i));
                }

                thread::sleep(Duration::from_millis(2));
            }
            pb.finish_with_message("done");
        });

        plog.join()?;

        logall("Iteration done!");

        plog.join_and_clear()?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example() {
        main().unwrap();
    }
}
