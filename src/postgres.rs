use anyhow::{Context, Result, anyhow};

use tokio_postgres::Config as PgConfig;
use log::{error, info};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use random_word::{Lang, get as random_word};
use tokio::task;

// Connect to PostgreSQL with SSL
pub async fn connect_ssl(config: &PgConfig, verify: bool, root_cert_path: Option<&str>) -> Result<tokio_postgres::Client> {
  let mut builder = TlsConnector::builder();
  if !verify {
      builder.danger_accept_invalid_certs(true);
  }
  if let Some(path) = root_cert_path {
      let cert_data = std::fs::read(path)?;
      let cert = native_tls::Certificate::from_pem(&cert_data)?;
      builder.add_root_certificate(cert);
  }
  let connector = builder.build()?;
  let connector = MakeTlsConnector::new(connector);

  let (client, connection) = config.connect(connector).await?;

  tokio::spawn(async move {
      if let Err(e) = connection.await {
          error!("connection error: {}", e);
      }
  });

  Ok(client)
}

// Connect to PostgreSQL without SSL
pub async fn connect_no_ssl(config: &PgConfig) -> Result<tokio_postgres::Client> {
  let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

  tokio::spawn(async move {
      if let Err(e) = connection.await {
          error!("connection error: {}", e);
      }
  });

  Ok(client)
}

// List all databases
pub async fn list_databases(client: &tokio_postgres::Client) -> Result<()> {
  let rows = client
      .query("SELECT datname FROM pg_database WHERE datistemplate = false;", &[])
      .await?;

  println!("Available databases:");
  for row in rows {
      let name: String = row.get(0);
      println!("  - {}", name);
  }

  Ok(())
}

// Create a new database
pub async fn create_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  client
      .execute(&format!("CREATE DATABASE \"{}\";", name), &[])
      .await
      .context("Failed to create database")?;

  info!("Database '{}' created successfully", name);
  Ok(())
}

// Clone a database
pub async fn clone_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  let new_name = format!("{}-clone", name);
  client
      .execute(&format!("CREATE DATABASE \"{}\" WITH TEMPLATE \"{}\" OWNER \"{}\" ;", new_name, name, name), &[])
      .await
      .context("Failed to clone database")?;

  info!("Database '{}' cloned to '{}' successfully", name, new_name);
  Ok(())
}

// Drop a database
pub async fn drop_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  client
      .execute(&format!("DROP DATABASE \"{}\" WITH (FORCE);", name), &[])
      .await
      .context("Failed to drop database")?;

  info!("Database '{}' dropped successfully", name);
  Ok(())
}

// Drop a database with force
pub async fn drop_database_with_force(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  client
      .execute(&format!("DROP DATABASE \"{}\" WITH (FORCE);", name), &[])
      .await
      .context("Failed to drop database")?;

  info!("Database '{}' dropped successfully", name);
  Ok(())
}

// Rename a database
pub async fn rename_database(client: &tokio_postgres::Client, old_name: &str, new_name: &str) -> Result<()> {
  client
      .execute(&format!("ALTER DATABASE \"{}\" RENAME TO \"{}\";", old_name, new_name), &[])
      .await
      .context("Failed to rename database")?;

  info!("Database '{}' renamed to '{}' successfully", old_name, new_name);
  Ok(())
}

// Set the owner of a database
pub async fn set_database_owner(client: &tokio_postgres::Client, name: &str, owner: &str) -> Result<()> {
  client
      .execute(&format!("ALTER DATABASE \"{}\" OWNER TO \"{}\";", name, owner), &[])
      .await
      .context("Failed to set database owner")?;

  info!("Database '{}' owner set to '{}' successfully", name, owner);
  Ok(())
}

// Change the password of a user
pub async fn change_password(client: &tokio_postgres::Client, user: &str, password: &str) -> Result<()> {
  client
      .execute(&format!("ALTER USER \"{}\" WITH PASSWORD \"{}\";", user, password), &[])
      .await
      .context(format!("Failed to change password for user {}", user))?;

  info!("Password for user '{}' changed successfully", user);
  Ok(())
}

/// Restore a database from a snapshot file
pub async fn restore_snapshot(
    host: &str,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    use_ssl: bool,
    file_path: &str,
) -> Result<String> {
    // Create a new database with a random name
    let new_dbname = format!("{}-restored", random_word(Lang::En));
    
    // Create a PgConfig for connection
    let mut config = PgConfig::new();
    config.host(host);
    config.port(port);
    
    if let Some(user) = &username {
        config.user(user);
    }
    
    if let Some(pass) = &password {
        config.password(pass);
    }
    
    // Connect to PostgreSQL
    let client = if use_ssl {
        connect_ssl(&config, false, None).await?
    } else {
        connect_no_ssl(&config).await?
    };
    
    // Create the target database
    match create_database(&client, &new_dbname).await {
        Ok(_) => {
            info!("Successfully created restore database: {}", new_dbname);
        },
        Err(e) => {
            let error_msg = format!("Failed to create restore database: {}", e);
            return Err(anyhow!(error_msg));
        }
    }
    
    // Create owned versions of parameters for the blocking task
    let file_path_owned = file_path.to_string();
    let host_owned = host.to_string();
    let new_dbname_owned = new_dbname.clone();
    
    // Spawn a blocking task to handle the restore operation
    let restore_handle = task::spawn_blocking(move || {
        // Call the restore_database function from the backup module
        let result = crate::backup::restore_database(
            &new_dbname_owned,
            &file_path_owned,
            &host_owned,
            port,
            username.as_deref(),
            password.as_deref(),
            use_ssl,
        );
        result
    });
    
    // Wait for the restore operation to complete
    match restore_handle.await {
        Ok(inner_result) => {
            match inner_result {
                Ok(_) => {
                    info!("pg_restore completed successfully to database {}", new_dbname);
                    Ok(new_dbname)
                },
                Err(e) => {
                    error!("pg_restore failed: {}", e);
                    Err(anyhow!("pg_restore task failed: {}", e))
                }
            }
        },
        Err(e) => {
            error!("pg_restore task panicked: {}", e);
            Err(anyhow!("pg_restore task issues: {}", e))
        }
    }
}