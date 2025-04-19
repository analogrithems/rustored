use rustored::{backup, config};
use anyhow::Result;
use clap::{Parser, Subcommand, command, arg};
use rustored::postgres;
use tokio_postgres::config::SslMode;
use tokio_postgres::Config as PgConfig;
use log::{error, info, warn, debug, LevelFilter};
use log4rs::{append::file::FileAppender, config::{Appender, Config as LogConfig, Root}, encode::pattern::PatternEncoder};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use rustored::ui::rustored::RustoredApp;

#[derive(Parser)]
#[command(name = "rustored")]
#[command(about = "PostgreSQL database management tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, help = "Postgres File Path")]
    file: Option<String>,

    #[arg(short = 'H', long, env = "PG_HOST", help = "Postgres Host")]
    host: Option<String>,

    #[arg(short, default_value = "5432", long, env = "PG_PORT", help = "Postgres Port")]
    port: Option<u16>,

    #[arg(short, long, env = "PG_USERNAME", help = "Postgres Username")]
    username: Option<String>,

    #[arg(short = 'P', long, env = "PG_PASSWORD", help = "Postgres Password")]
    password: Option<String>,

    #[arg(short = 'D', default_value = "postgres", long, env = "PG_DB_NAME", help = "Postgres Database Name")]
    db_name: Option<String>,

    #[arg(long, default_value = "false", env = "PG_USE_SSL", help = "Postgres Enable SSL")]
    use_ssl: bool,

    #[arg(long, env = "PG_ROOT_CERT_PATH", help = "Postgres Path to custom root certificates")]
    root_cert_path: Option<String>,

    #[arg(long, default_value = "false", env = "PG_VERIFY_SSL", help = "Postgres Verify SSL certificates")]
    verify_ssl: bool,

    #[arg(short = 'B', long, env = "S3_BUCKET", help = "S3 Bucket Name")]
    bucket: Option<String>,

    #[arg(short = 'R', long, env = "S3_REGION", help = "S3 Region")]
    region: Option<String>,

    #[arg(short = 'x', long, default_value = "postgres", env = "S3_PREFIX", help = "S3 Prefix for snapshot keys")]
    prefix: Option<String>,

    #[arg(short = 'E', long, env = "S3_ENDPOINT_URL", help = "S3 Endpoint URL")]
    endpoint_url: Option<String>,

    #[arg(short = 'A', long, env = "S3_ACCESS_KEY_ID", help = "S3 Access Key ID")]
    access_key_id: Option<String>,

    #[arg(short = 'S', long, env = "S3_SECRET_ACCESS_KEY", help = "S3 Secret Access Key")]
    secret_access_key: Option<String>,

    #[arg(long, default_value = "true", env = "S3_PATH_STYLE", help = "S3 Force path-style")]
    path_style: bool,

    /// Elasticsearch host or URL
    #[arg(long, help = "Elasticsearch host or URL")]
    es_host: Option<String>,

    /// Elasticsearch index or Qdrant collection name
    #[arg(long, help = "Elasticsearch index or Qdrant collection name")]
    es_index: Option<String>,

    /// Qdrant API key (optional)
    #[arg(long, help = "Qdrant API key (optional)")]
    qdrant_api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List all databases")]
    List,

    #[command(about = "Create a new database")]
    Create {
        #[arg(help = "Name of the database to create")]
        name: String,
    },

    #[command(about = "Clone a database")]
    Clone {
        #[arg(help = "Name of the database to clone from. Will create a new database with the name '<same_name>-clone'")]
        name: String,
    },

    #[command(about = "Drop a database")]
    Drop {
        #[arg(help = "Name of the database to drop")]
        name: String,
    },

    #[command(about = "Drop a database with force")]
    DropForce {
        #[arg(help = "Name of the database to drop")]
        name: String,
    },

    #[command(about = "Rename a database")]
    Rename {
        #[arg(help = "Name of the database to rename")]
        old_name: String,

        #[arg(help = "New name for the database")]
        new_name: String,
    },

    #[command(about = "Set database owner")]
    SetOwner {
        #[arg(help = "Name of the database")]
        name: String,

        #[arg(help = "New owner for the database")]
        owner: String,
    },

    #[command(about = "Change the password of a user")]
    ChangePassword {
        #[arg(help = "Name of the user")]
        user: String,

        #[arg(help = "New password for the user")]
        password: String,
    },

    #[command(about = "Dump a database")]
    Dump {
        #[arg(help = "Name of the database to dump")]
        name: String,

        #[arg(help = "Output file path")]
        output: String,
    },

    #[command(about = "Restore a snapshot to a datastore")]
    Restore {
        #[arg(help = "Name of the destination database, index, or collection")]
        name: String,

        #[arg(help = "Input dump file path")]
        input: String,

        #[arg(long, default_value = "postgres", help = "Target datastore: postgres, elasticsearch, or qdrant")]
        target: String,

        // Elasticsearch/Qdrant options
        #[arg(long, help = "Elasticsearch/Qdrant host or URL")]
        es_host: Option<String>,
        #[arg(long, help = "Elasticsearch index or Qdrant collection name")]
        es_index: Option<String>,
        #[arg(long, help = "Qdrant API key (optional)")]
        qdrant_api_key: Option<String>,
    },

    /// Browse and restore S3 snapshots using TUI
    BrowseSnapshots,
}

async fn connect(cli: &Cli) -> Result<Option<tokio_postgres::Client>> {
    debug!("Attempting to connect to PostgreSQL with settings: host={:?}, port={:?}, user={:?}, ssl={}", 
           cli.host, cli.port, cli.username, cli.use_ssl);
    if !cli.host.is_some() && !cli.port.is_some() && !cli.username.is_some() && !cli.password.is_some() {
        // If no PostgreSQL settings are provided, return None
        debug!("No PostgreSQL connection settings provided, skipping connection");
        return Ok(None);
    }

    let mut config = PgConfig::new();

    if cli.use_ssl {
        config.ssl_mode(SslMode::Require);
    }

    // Set default host and port if not provided
    config.host(&cli.host.clone().unwrap_or_else(|| "localhost".to_string()));
    config.port(cli.port.unwrap_or(5432));

    if let Some(ref user) = cli.username {
        config.user(user);
    }

    if let Some(ref password) = cli.password {
        config.password(password);
    }

    let result = if cli.use_ssl {
        postgres::connect_ssl(&config, cli.verify_ssl, cli.root_cert_path.as_deref()).await
    } else {
        postgres::connect_no_ssl(&config).await
    };

    match result {
        Ok(client) => Ok(Some(client)),
        Err(e) => {
            warn!("Failed to connect to PostgreSQL: {}", e);
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    debug!("Starting Rustored application");
    // Configure logging
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}")))
        .build("rustored.log")?;

    let log_config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Debug))?;

    log4rs::init_config(log_config)?;
    info!("Starting rustored");

    // Load environment variables from .env file
    config::load_env();
    info!("Loaded environment variables");

    let cli: Cli = Cli::parse();
    let client = connect(&cli).await?;

    // Add PGSSLMODE environment variable if SSL is enabled
    if cli.use_ssl {
        std::env::set_var("PGSSLMODE", "require");
    }

    match &cli.command {
        Commands::ChangePassword { user, password } => {
            if let Some(client) = client {
                postgres::change_password(&client, &user, &password).await?;
            } else {
                error!("PostgreSQL connection required for postgres::change_password");
                return Ok(());
            }
        }
        Commands::List => {
            if let Some(client) = client {
                postgres::list_databases(&client).await?;
            } else {
                error!("PostgreSQL connection required for postgres::list_databases");
                return Ok(());
            }
        }
        Commands::Create { name } => {
            if let Some(client) = client {
                postgres::create_database(&client, &name).await?;
            } else {
                error!("PostgreSQL connection required for postgres::create_database");
                return Ok(());
            }
        }
        Commands::Drop { name } => {
            if let Some(client) = client {
                postgres::drop_database(&client, &name).await?;
            } else {
                error!("PostgreSQL connection required for postgres::drop_database");
                return Ok(());
            }
        }
        Commands::Clone { name } => {
            if let Some(client) = client {
                postgres::clone_database(&client, &name).await?;
            } else {
                error!("PostgreSQL connection required for postgres::clone_database");
                return Ok(());
            }
        }
        Commands::DropForce { name } => {
            if let Some(client) = client {
                postgres::drop_database_with_force(&client, &name).await?;
            } else {
                error!("PostgreSQL connection required for postgres::drop_database_with_force");
                return Ok(());
            }
        }
        Commands::Rename { old_name, new_name } => {
            if let Some(client) = client {
                postgres::rename_database(&client, &old_name, &new_name).await?;
            } else {
                error!("PostgreSQL connection required for postgres::rename_database");
                return Ok(());
            }
        }
        Commands::SetOwner { name, owner } => {
            if let Some(client) = client {
                postgres::set_database_owner(&client, &name, &owner).await?;
            } else {
                error!("PostgreSQL connection required for postgres::set_database_owner");
                return Ok(());
            }
        }
        Commands::Dump { name, output } => {
            if let Some(_) = client {
                info!("Dumping database '{}' to '{}'", name, output);
                backup::dump_database(
                    &name,
                    &output,
                    &cli.host.clone().unwrap_or_else(|| "localhost".to_string()),
                    cli.port.unwrap_or(5432),
                    cli.username.as_deref(),
                    cli.password.as_deref(),
                    cli.use_ssl,
                )
                .await?
            } else {
                error!("PostgreSQL connection required for postgres::dump_database");
                return Ok(());
            }
        }
        Commands::Restore { name, input, target, es_host, es_index, qdrant_api_key } => {
            use rustored::datastore::DatastoreRestoreTarget;
            let datastore = match target.as_str() {
                "postgres" => DatastoreRestoreTarget::Postgres,
                "elasticsearch" => DatastoreRestoreTarget::Elasticsearch {
                    host: es_host.clone().unwrap_or_else(|| "http://localhost:9200".to_string()),
                    index: es_index.clone().unwrap_or_else(|| name.clone()),
                },
                "qdrant" => DatastoreRestoreTarget::Qdrant {
                    host: es_host.clone().unwrap_or_else(|| "http://localhost:6333".to_string()),
                    collection: es_index.clone().unwrap_or_else(|| name.clone()),
                    api_key: qdrant_api_key.clone(),
                },
                other => {
                    error!("Unknown restore target: {}", other);
                    return Ok(());
                }
            };
            datastore.restore(&name, &input).await?;
        }
        Commands::BrowseSnapshots => {
            // TUI using RustoredApp
            enable_raw_mode()?;
            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;
            let mut app = RustoredApp::new(
                &cli.bucket,
                &cli.region,
                &cli.prefix,
                &cli.endpoint_url,
                &cli.access_key_id,
                &cli.secret_access_key,
                cli.path_style,
                &cli.host,
                &cli.port,
                &cli.username,
                &cli.password,
                cli.use_ssl,
                &cli.db_name,
                &cli.es_host,
                &cli.es_index,
                &cli.qdrant_api_key,
            );

            let res = app.run(&mut terminal).await?;
            disable_raw_mode()?;
            execute!(std::io::stdout(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
            terminal.show_cursor()?;
            if let Some(snapshot_key) = res {
                info!("Snapshot processed: {}", snapshot_key);
            }
        }
    }

    Ok(())
}
