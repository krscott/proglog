use indicatif::ProgressStyle;
use log::LevelFilter;
use log4rs::config::Logger;
use proglog::{ProgLog, ProgLogConfig};
use std::thread;
use std::time::Duration;

fn logall(msg: &str) {
    log::error!("error {}", msg);
    log::warn!("warn {}", msg);
    log::info!("info {}", msg);
    log::debug!("debug {}", msg);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Based on https://github.com/mitsuhiko/indicatif/blob/main/examples/multi.rs

    let config = ProgLogConfig::builder()
        .verbosity(1)
        .style(
            ProgressStyle::default_bar()
                .template("{elapsed_precise} [{wide_bar}] {pos:>7}/{len:7} {eta} {msg:40}")
                .progress_chars("=> "),
        )
        .loggers(vec![
            Logger::builder().build("hyper", LevelFilter::Info),
            Logger::builder().build("reqwest", LevelFilter::Info),
            Logger::builder().build("mio", LevelFilter::Info),
            Logger::builder().build("want", LevelFilter::Info),
        ])
        .build();

    let mut plog = ProgLog::new(config)?;

    for j in 0..2 {
        let pb = plog.add(128);

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

        let pb = plog.add(128);
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

        let pb = plog.add(1024);
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
