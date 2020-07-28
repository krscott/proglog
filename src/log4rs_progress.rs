use indicatif::{ProgressBar, ProgressDrawTarget};

struct StringWriter {
    pub buffer: String,
}

impl StringWriter {
    fn new() -> Self {
        StringWriter {
            buffer: String::new(),
        }
    }
}

impl std::io::Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.buffer.push_str(s);
                Ok(buf.len())
            }
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl log4rs::encode::Write for StringWriter {
    fn set_style(&mut self, _style: &log4rs::encode::Style) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct ProgressAppender {
    encoder: Box<dyn log4rs::encode::Encode>,
    progress_bar: ProgressBar,
    is_a_tty: bool,
}

impl std::fmt::Debug for ProgressAppender {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "ProgressAppender {{encoder: {:?}}}",
            self.encoder
        )
    }
}

impl ProgressAppender {
    pub fn builder() -> ProgressAppenderBuilder {
        ProgressAppenderBuilder {
            encoder: None,
            progress_bar: None,
        }
    }
}

impl log4rs::append::Append for ProgressAppender {
    fn append(
        &self,
        record: &log::Record,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut buf = StringWriter::new();

        self.encoder.encode(&mut buf, record)?;

        if self.is_a_tty {
            self.progress_bar.println(format!(
                "{}{}",
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                buf.buffer
            ));
        } else {
            eprintln!("{}", buf.buffer);
        }

        Ok(())
    }

    fn flush(&self) {}
}

pub struct ProgressAppenderBuilder {
    encoder: Option<Box<dyn log4rs::encode::Encode>>,
    progress_bar: Option<ProgressBar>,
}

impl ProgressAppenderBuilder {
    pub fn encoder(mut self, encoder: Box<dyn log4rs::encode::Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn progress_bar(mut self, progress_bar: ProgressBar) -> Self {
        self.progress_bar = Some(progress_bar);
        self
    }

    pub fn build(self) -> ProgressAppender {
        ProgressAppender {
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(log4rs::encode::pattern::PatternEncoder::default())),
            progress_bar: self.progress_bar.unwrap_or_else(|| {
                let progress_bar = ProgressBar::new(0);
                progress_bar.set_draw_target(ProgressDrawTarget::stderr());
                progress_bar
            }),
            is_a_tty: atty::is(atty::Stream::Stderr),
        }
    }
}
