use std::fmt;
use std::io::Write;

use owo_colors::OwoColorize;
use time::format_description::FormatItem;
use tracing::metadata::LevelFilter;
use tracing::{Event, Level, Metadata};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_core::Subscriber as CoreSubscriber;
use tracing_subscriber::fmt::format::{DefaultFields, Writer};
use tracing_subscriber::fmt::time::{FormatTime, OffsetTime, SystemTime};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, Subscriber};
use tracing_subscriber::layer::{Context, Filter};
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

use spherix_config::{Log, LogLevel};

/// Performs stripping ANSI sequences in logs to write text into files correctly.
/// Actually it is just decorator for primary [`Writer`].
///
/// [`Writer`]: std::io::Write
struct AnsiStripper<W: Write> {
    inner: W,
}

impl<W: Write> AnsiStripper<W> {
    fn new(inner: W) -> AnsiStripper<W> {
        AnsiStripper {
            inner
        }
    }
}

impl<W: Write> Write for AnsiStripper<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let stripped = strip_ansi_escapes::strip(buf).unwrap();
        self.inner.write(&stripped)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

struct TargetFilter<'a> {
    ignore_targets: Vec<&'a str>,
}

impl<'a> TargetFilter<'a> {
    fn new(ignore_targets: Vec<&'a str>) -> Self {
        Self {
            ignore_targets
        }
    }
}

impl<S> Filter<S> for TargetFilter<'static> {
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        !self.ignore_targets.contains(&meta.target())
    }
}

pub fn configure_logger(config: &Log) -> WorkerGuard {
    let file_appender = AnsiStripper::new(
        tracing_appender::rolling::daily("log", config.file.file_prefix.clone())
    );
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let layers = tracing_subscriber::registry()
        .with(
            console_subscriber::ConsoleLayer::builder()
                .with_default_env()
                .server_addr(([127, 0, 0, 1], 6669))
                .spawn()
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(CustomFormat {
                    timer: create_timer(),
                    ansi: config.terminal.ansi,
                    target: config.terminal.target
                })
                .with_ansi(config.terminal.ansi)
                .with_filter(LevelFilter::from_level(convert_log_level(config.terminal.level)))
                .with_filter(ignore_targets_filter())
        );

    if config.file.enabled {
        layers.with(
            tracing_subscriber::fmt::layer()
                .event_format(CustomFormat {
                    timer: create_timer(),
                    ansi: false,
                    target: config.file.target
                })
                .with_writer(file_writer)
                .with_filter(LevelFilter::from_level(convert_log_level(config.file.level)))
                .with_filter(ignore_targets_filter())
        ).init();
    } else {
        layers.init();
    };

    _guard
}

pub fn configure_temporary_logger() -> Subscriber<DefaultFields, CustomFormat<OffsetTime<Vec<FormatItem<'static>>>>> {
    tracing_subscriber::fmt()
        .event_format(CustomFormat {
            timer: create_timer(),
            ansi: true,
            target: true
        })
        .with_max_level(Level::INFO)
        .finish()
}

fn ignore_targets_filter() -> TargetFilter<'static> {
    TargetFilter::new(vec![
        "want",
        "mio::poll",
        "hyper::proto::h1::decode",
        "hyper::proto::h1::conn",
        "hyper::proto::h1::io",
        "hyper::proto::h1::role",
        "hyper::client::pool",
        "hyper::client::conn",
        "hyper::client::client",
        "hyper::client::connect::http",
        "reqwest::connect",
        "rustyline",
        // console-subscriber
        "runtime::resource::poll_op",
        "runtime::resource::state_update",
        "tokio::task::waker",
    ])
}

fn create_timer() -> OffsetTime<Vec<FormatItem<'static>>> {
    let timer = time::format_description::parse(
        "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]",
    ).unwrap();
    let time_offset = time::UtcOffset::current_local_offset().unwrap_or_else(|_| time::UtcOffset::UTC);
    OffsetTime::new(time_offset, timer)
}

fn convert_log_level(level: LogLevel) -> Level {
    match level {
        LogLevel::TRACE => Level::TRACE,
        LogLevel::DEBUG => Level::DEBUG,
        LogLevel::INFO => Level::INFO,
        LogLevel::WARN => Level::WARN,
        LogLevel::ERROR => Level::ERROR
    }
}

// Code below is rewritten implementation of tracing_subscriber::fmt::format::Format<Full, _>.
// The primary reason why I have rewritten this - to permanently hide very verbose logging of
// event_scope.

pub struct CustomFormat<T = SystemTime> {
    timer: T,
    ansi: bool,
    target: bool
}

impl<C, N, T> FormatEvent<C, N> for CustomFormat<T>
    where
        C: CoreSubscriber + for<'a> LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
        T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, C, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        self.format_timestamp(&mut writer)?;
        let fmt_level = FmtLevel::new(meta.level(), self.ansi);
        write!(writer, "{} ", fmt_level)?;

        if self.target {
            write!(
                writer,
                "{}{} ",
                meta.target().dimmed(),
                ":".dimmed()
            )?;
        }

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

impl<T> CustomFormat<T> {
    #[inline]
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result
        where
            T: FormatTime,
    {
        if writer.has_ansi_escapes() {
            let dimmed = nu_ansi_term::Style::new().dimmed();
            write!(writer, "{}", dimmed.prefix())?;

            if self.timer.format_time(writer).is_err() {
                writer.write_str("<unknown time>")?;
            }

            write!(writer, "{} ", dimmed.suffix())?;

            return Ok(());
        }

        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

struct FmtLevel<'a> {
    level: &'a Level,
    ansi: bool,
}

impl<'a> FmtLevel<'a> {
    pub(crate) fn new(level: &'a Level, ansi: bool) -> Self {
        Self { level, ansi }
    }
}

const TRACE_STR: &str = "TRACE";
const DEBUG_STR: &str = "DEBUG";
const INFO_STR: &str = " INFO";
const WARN_STR: &str = " WARN";
const ERROR_STR: &str = "ERROR";

impl<'a> fmt::Display for FmtLevel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ansi {
            match *self.level {
                Level::TRACE => write!(f, "{}", TRACE_STR.purple()),
                Level::DEBUG => write!(f, "{}", DEBUG_STR.blue()),
                Level::INFO => write!(f, "{}", INFO_STR.green()),
                Level::WARN => write!(f, "{}", WARN_STR.yellow()),
                Level::ERROR => write!(f, "{}", ERROR_STR.red()),
            }
        } else {
            match *self.level {
                Level::TRACE => f.pad(TRACE_STR),
                Level::DEBUG => f.pad(DEBUG_STR),
                Level::INFO => f.pad(INFO_STR),
                Level::WARN => f.pad(WARN_STR),
                Level::ERROR => f.pad(ERROR_STR),
            }
        }
    }
}
