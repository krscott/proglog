use crate::log4rs_progress::ProgressAppender;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use log4rs::encode::pattern::PatternEncoder;
use std::io;

const DEFAULT_ENCODER_PATTERN: &str = "{h({l})} {f}:{L} - {m}\n";

pub struct ProgLogBuilder {
    style: Option<ProgressStyle>,
    encoder: Option<Box<PatternEncoder>>,
}

impl ProgLogBuilder {
    fn new() -> Self {
        Self {
            style: None,
            encoder: None,
        }
    }

    pub fn style(mut self, value: ProgressStyle) -> Self {
        self.style = Some(value);
        self
    }

    pub fn encoder(mut self, value: Box<PatternEncoder>) -> Self {
        self.encoder = Some(value);
        self
    }

    pub fn build(self) -> (ProgLog, ProgressAppender) {
        ProgLog::from_builder(self)
    }
}

pub struct ProgLog {
    style: ProgressStyle,
    logger_progress_bar: ProgressBar,
    is_first_pb: bool,
    multi_progress: MultiProgress,
}

impl ProgLog {
    pub fn builder() -> ProgLogBuilder {
        ProgLogBuilder::new()
    }

    fn from_builder(builder: ProgLogBuilder) -> (Self, ProgressAppender) {
        let style = builder
            .style
            .unwrap_or_else(|| ProgressStyle::default_bar());

        let logger_progress_bar = new_progress_bar();
        hide_progress_bar(&logger_progress_bar);

        let multi_progress = MultiProgress::new();
        multi_progress.add(logger_progress_bar.clone());

        let progress_appender = ProgressAppender::builder()
            .encoder(
                builder
                    .encoder
                    .unwrap_or_else(|| Box::new(PatternEncoder::new(DEFAULT_ENCODER_PATTERN))),
            )
            .progress_bar(logger_progress_bar.clone())
            .build();

        (
            Self {
                style,
                multi_progress,
                logger_progress_bar,
                is_first_pb: true,
            },
            progress_appender,
        )
    }

    pub fn add_with_length(&mut self, len: u64) -> ProgressBar {
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

    pub fn add(&mut self) -> ProgressBar {
        self.add_with_length(1)
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

fn new_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new(1);
    pb.set_draw_target(ProgressDrawTarget::stderr());
    pb
}

fn hide_progress_bar(pb: &ProgressBar) {
    pb.set_style(ProgressStyle::default_bar().template(" "));
}
