use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use log::SetLoggerError;
use std::{io, path::Path};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct ProgLogConfig<'a> {
    #[builder(default = 1)]
    verbosity: i32,

    #[builder(default, setter(strip_option))]
    log_path: Option<&'a Path>,

    #[builder(default = false)]
    show_line_numbers: bool,

    #[builder(default, setter(strip_option))]
    style: Option<ProgressStyle>,
}

pub struct ProgLog {
    style: ProgressStyle,
    logger_progress_bar: ProgressBar,
    is_first_pb: bool,
    multi_progress: MultiProgress,
}

impl ProgLog {
    pub fn new(config: ProgLogConfig) -> Result<Self, SetLoggerError> {
        let logger_progress_bar = new_progress_bar(1);
        hide_progress_bar(&logger_progress_bar);

        crate::init_logger::init_logger(
            config.verbosity,
            config.log_path,
            config.show_line_numbers,
            Some(logger_progress_bar.clone()),
        )?;

        let multi_progress = MultiProgress::new();
        multi_progress.add(logger_progress_bar.clone());

        Ok(Self {
            style: config.style.unwrap_or_else(|| ProgressStyle::default_bar()),
            multi_progress,
            logger_progress_bar,
            is_first_pb: true,
        })
    }

    pub fn add(&mut self, len: u64) -> ProgressBar {
        let pb = if self.is_first_pb {
            self.is_first_pb = false;

            let pb = self.logger_progress_bar.clone();
            pb.set_position(0);
            pb.set_length(len);
            pb.reset_eta();
            pb.reset_elapsed();

            pb
        } else {
            self.multi_progress.add(ProgressBar::new(len))
        };

        pb.set_style(self.style.clone());
        pb
    }

    fn pre_join(&mut self) {
        if self.is_first_pb {
            self.logger_progress_bar.finish();
        }
        self.multi_progress.set_move_cursor(true);
    }

    fn reset(&mut self) {
        self.is_first_pb = true;
        self.logger_progress_bar.reset();
        hide_progress_bar(&self.logger_progress_bar);
        self.multi_progress = MultiProgress::new();
        self.multi_progress.add(self.logger_progress_bar.clone());
    }

    pub fn join(&mut self) -> Result<(), io::Error> {
        self.pre_join();
        self.multi_progress.join()?;
        self.reset();
        Ok(())
    }

    pub fn join_and_clear(&mut self) -> Result<(), io::Error> {
        self.pre_join();
        self.multi_progress.join_and_clear()?;
        self.reset();
        Ok(())
    }
}

fn new_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_draw_target(ProgressDrawTarget::stderr());
    pb
}

fn hide_progress_bar(pb: &ProgressBar) {
    pb.set_style(ProgressStyle::default_bar().template(" "));
}
