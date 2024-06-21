use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

const MODULE_WHITELIST: &[&str] = &[];

pub fn init(level: LevelFilter) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            EnvFilter::try_new(
                MODULE_WHITELIST
                    .iter()
                    .map(|m| format!("{m}={level}"))
                    .chain(std::iter::once(format!("{}={}", "snipdoc", level)))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .expect("logger initialization failed");

    let builder = tracing_subscriber::FmtSubscriber::builder().with_env_filter(filter);
    builder.compact().init();
}
