use std::{env, error::Error, io::IsTerminal, path::PathBuf};

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub use tracing::{
    Level, Span, debug, debug_span, error, error_span, event, field, info, info_span, instrument,
    span, trace, trace_span, warn, warn_span,
};

pub const ENV_FILTER_VAR: &str = "KEYWAY_LOG";
pub const FALLBACK_ENV_FILTER_VAR: &str = "RUST_LOG";
pub const DEFAULT_FILTER: &str = "info";

pub type InitResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Debug)]
pub struct InitOptions {
    pub filter: Option<String>,
    pub log_to_file: bool,
    pub log_to_stderr: bool,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            filter: None,
            log_to_file: true,
            log_to_stderr: std::io::stderr().is_terminal(),
        }
    }
}

pub fn init() {
    if let Err(error) = try_init(InitOptions::default()) {
        eprintln!("failed to initialize tracing: {error}");
    }
}

pub fn init_test() {
    let _ = try_init(InitOptions {
        filter: None,
        log_to_file: false,
        log_to_stderr: true,
    });
}

pub fn try_init(options: InitOptions) -> InitResult<()> {
    let filter = env_filter(options.filter.as_deref());

    let file_layer = if options.log_to_file {
        Some(file_layer()?)
    } else {
        None
    };

    let stderr_layer = options.log_to_stderr.then(|| {
        tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(true)
            .with_writer(std::io::stderr)
    });

    let _ = tracing_log::LogTracer::init();

    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .try_init()?;

    Ok(())
}

fn env_filter(explicit: Option<&str>) -> EnvFilter {
    let filter = explicit
        .map(str::to_owned)
        .or_else(|| env::var(ENV_FILTER_VAR).ok())
        .or_else(|| env::var(FALLBACK_ENV_FILTER_VAR).ok())
        .unwrap_or_else(|| DEFAULT_FILTER.to_owned());

    EnvFilter::try_new(&filter).unwrap_or_else(|error| {
        eprintln!("invalid tracing filter `{filter}`: {error}; falling back to `{DEFAULT_FILTER}`");
        EnvFilter::new(DEFAULT_FILTER)
    })
}

fn file_layer<S>() -> InitResult<impl tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber,
    S: for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let log_file = paths::log_file();
    let logs_dir = log_file
        .parent()
        .expect("log file path should include a parent directory");
    let log_file_name = log_file
        .file_name()
        .expect("log file path should include a file name");
    std::fs::create_dir_all(logs_dir)?;

    let file_appender = tracing_appender::rolling::never(logs_dir, PathBuf::from(log_file_name));

    Ok(tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(file_appender))
}
