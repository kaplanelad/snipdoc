use std::path::PathBuf;
mod cmd;
use clap::{ArgAction, Parser, Subcommand};
use snipdoc::{config::Config, parser::SnippetKind, reporters};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(clap::ValueEnum, Default, Clone)]
pub enum Format {
    Table,
    #[default]
    Console,
}

impl Format {
    #[must_use]
    pub fn reporter(&self) -> Box<dyn reporters::ReporterOutput> {
        match self {
            Self::Table => {
                Box::new(reporters::table::Output {}) as Box<dyn reporters::ReporterOutput>
            }
            Self::Console => {
                Box::new(reporters::console::Output {}) as Box<dyn reporters::ReporterOutput>
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(global = true, short, long, value_enum, default_value = "INFO")]
    /// Log level
    log_level: LevelFilter,

    /// Source code directory for collecting documentation
    #[clap(global = true, index = 1, default_value = ".")]
    path: PathBuf,

    /// Application config. by default will search `./snipdoc-config.yml`
    #[arg(global = true, short, long, value_enum)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create a local DB file
    CreateDb {
        /// Show the injection operation without changes
        #[clap(long, action=ArgAction::SetTrue)]
        empty: bool,
    },
    /// Validate if snippets are equal, errors or missing configuration
    Check {
        #[arg(long, default_value = None)]
        db_file: Option<PathBuf>,
    },
    /// Inject snippet into placeholders
    Run {
        #[arg(long, default_value = None)]
        db_file: Option<PathBuf>,

        /// Show the injection operation without changes
        #[clap(long, action=ArgAction::SetTrue)]
        dry_run: bool,

        /// Format of the results
        #[arg(long, value_enum, default_value_t = Format::default())]
        format: Format,
    },
    /// Show snippets
    Show {
        #[arg(long,value_enum, default_value_t = SnippetKind::default())]
        from: SnippetKind,

        #[arg(long, default_value = None)]
        db_file: Option<PathBuf>,

        /// Format of the results
        #[arg(long, value_enum, default_value_t = Format::default())]
        format: Format,
    },
}

fn main() {
    let app: Cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(app.log_level.into())
                .from_env_lossy(),
        )
        .with_line_number(true)
        .with_target(true)
        .init();

    let span = tracing::span!(tracing::Level::INFO, "cli");
    let _guard = span.enter();

    let config = if let Some(config) = app.config {
        match Config::from_file(&config) {
            Ok(config) => config,
            Err(err) => {
                tracing::error!(err = %err, "could not load configuration file");
                std::process::exit(1);
            }
        }
    } else {
        Config::try_from_default_file(app.path.as_path())
    };

    match app.command {
        Commands::CreateDb { empty } => cmd::create_db::exec(&config, app.path.as_path(), empty),
        Commands::Check { db_file } => cmd::check::exec(&config, app.path.as_path(), db_file),
        Commands::Run {
            db_file,
            dry_run,
            format,
        } => cmd::run::exec(&config, app.path.as_path(), db_file, dry_run, &format),
        Commands::Show {
            from,
            db_file,
            format,
        } => cmd::show::exec(&config, app.path.as_path(), &from, db_file, &format),
    }
    .exit();
}
