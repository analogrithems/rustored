// This module contains PostgreSQL database operations for the Rustored application
// It provides functionality for connecting to PostgreSQL databases with and without SSL,
// as well as various database management operations like listing, creating, cloning,
// dropping, renaming databases, and changing user passwords.

use anyhow::{Result, anyhow};

// Import PostgreSQL configuration from tokio-postgres
use tokio_postgres::Config as PgConfig;
// Import logging utilities for error and info level logging
use log::{error, info, debug};
// Import TLS connector for secure connections
use native_tls::TlsConnector;
// Import PostgreSQL-specific TLS connector
use postgres_native_tls::MakeTlsConnector;
// Import random word generator for creating random database names
use random_word::{Lang, get as random_word};
// Import task utilities for spawning async tasks
use tokio::task;

/// Connect to PostgreSQL with SSL security
/// 
/// This function establishes a connection to a PostgreSQL database using SSL/TLS encryption.
/// It allows for optional certificate verification and custom root certificates.
/// 
/// # Arguments
/// 
/// * `config` - PostgreSQL connection configuration
/// * `verify` - Whether to verify SSL certificates (false to accept invalid certs)
/// * `root_cert_path` - Optional path to a custom root certificate file
/// 
/// # Returns
/// 
/// A Result containing either the connected PostgreSQL client or an error
pub async fn connect_ssl(config: &PgConfig, verify: bool, root_cert_path: Option<&str>) -> Result<tokio_postgres::Client> {
  debug!("Building TLS connector for PostgreSQL connection");
  // Create a new TLS connector builder for configuring the SSL connection
  let mut builder = TlsConnector::builder();
  
  // If verification is disabled, accept invalid certificates (useful for self-signed certs)
  if !verify {
      debug!("SSL certificate verification disabled");
      builder.danger_accept_invalid_certs(true);
  }
  
  // If a custom root certificate path is provided, load and add it to the connector
  if let Some(path) = root_cert_path {
      debug!("Loading custom root certificate from: {}", path);
      // Read the certificate file
      let cert_data = std::fs::read(path)?;
      // Parse the PEM-formatted certificate
      let cert = native_tls::Certificate::from_pem(&cert_data)?;
      // Add the certificate to the trusted roots
      builder.add_root_certificate(cert);
  }
  
  // Build the TLS connector with the configured settings
  let connector = builder.build()?;
  // Create a PostgreSQL-compatible TLS connector
  let connector = MakeTlsConnector::new(connector);

  debug!("Attempting to connect to PostgreSQL with SSL");
  // Establish the connection using the provided config and TLS connector
  // This returns both a client for executing queries and a connection future
  let (client, connection) = config.connect(connector).await?;

  // Spawn a background task to manage the connection lifecycle
  // This is necessary because the connection must be polled to completion
  tokio::spawn(async move {
      debug!("PostgreSQL connection background task started");
      // Wait for the connection to complete and log any errors
      if let Err(e) = connection.await {
          error!("PostgreSQL connection error: {}", e);
      }
  });

  debug!("PostgreSQL SSL connection established successfully");
  // Return the client for executing queries
  Ok(client)
}

/// Connect to PostgreSQL without SSL security
/// 
/// This function establishes a connection to a PostgreSQL database without encryption.
/// It should only be used for local development or in secure networks.
/// 
/// # Arguments
/// 
/// * `config` - PostgreSQL connection configuration
/// 
/// # Returns
/// 
/// A Result containing either the connected PostgreSQL client or an error
pub async fn connect_no_ssl(config: &PgConfig) -> Result<tokio_postgres::Client> {
  debug!("Attempting to connect to PostgreSQL without SSL");
  // Establish the connection using the provided config and no TLS
  // This returns both a client for executing queries and a connection future
  let (client, connection) = config.connect(tokio_postgres::NoTls).await?;

  // Spawn a background task to manage the connection lifecycle
  // This is necessary because the connection must be polled to completion
  tokio::spawn(async move {
      debug!("PostgreSQL connection background task started");
      // Wait for the connection to complete and log any errors
      if let Err(e) = connection.await {
          error!("PostgreSQL connection error: {}", e);
      }
  });

  debug!("PostgreSQL non-SSL connection established successfully");
  // Return the client for executing queries
  Ok(client)
}

/// List all databases in the PostgreSQL server
/// 
/// This function retrieves and displays a list of all non-template databases
/// available on the connected PostgreSQL server.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn list_databases(client: &tokio_postgres::Client) -> Result<()> {
  debug!("Retrieving list of all PostgreSQL databases");
  // Query the pg_database system catalog to get all non-template databases
  // Template databases are special system databases used as templates for new databases
  let rows = client
      .query("SELECT datname FROM pg_database WHERE datistemplate = false;", &[])
      .await?;

  // Iterate through the result rows and display each database name
  debug!("Found {} databases", rows.len());
  for row in rows {
      // Extract the database name from the first column
      let name: String = row.get(0);
      // Log the database name at info level for user visibility
      info!("{}", name);
  }

  debug!("Successfully listed all databases");
  Ok(())
}

/// Create a new PostgreSQL database
/// 
/// This function creates a new database with the specified name.
/// The database name is properly quoted to handle special characters.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `name` - Name for the new database
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn create_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  debug!("Creating new PostgreSQL database: {}", name);
  
  // Format the CREATE DATABASE SQL statement with proper quoting
  // Double quotes are used to preserve case and allow special characters in database names
  let query = format!("CREATE DATABASE \"{}\";", name);
  
  // Execute the query to create the database
  client.execute(&query, &[]).await?;
  
  // Log successful database creation at info level for user visibility
  info!("Created database: {}", name);
  debug!("Database creation completed successfully");
  Ok(())
}

/// Clone a PostgreSQL database
/// 
/// This function creates a new database as an exact copy of an existing database.
/// The new database name is automatically generated using the original name plus a random word.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `name` - Name of the source database to clone
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn clone_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  debug!("Cloning PostgreSQL database: {}", name);
  
  // Generate a new name for the cloned database by appending a random English word
  // This ensures the new database has a unique but recognizable name
  let new_name = format!("{}_clone_{}", name, random_word(Lang::En));
  debug!("Generated clone name: {}", new_name);
  
  // Format the CREATE DATABASE SQL statement with the TEMPLATE option
  // This creates a new database as an exact copy of the specified template
  let query = format!("CREATE DATABASE \"{}\" WITH TEMPLATE \"{}\";", new_name, name);
  
  // Execute the query to create the clone database
  client.execute(&query, &[]).await?;
  
  // Log successful database cloning at info level for user visibility
  info!("Cloned database {} to {}", name, new_name);
  debug!("Database cloning completed successfully");
  Ok(())
}

/// Drop (delete) a PostgreSQL database
/// 
/// This function permanently deletes the specified database.
/// Note that this will fail if there are active connections to the database.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `name` - Name of the database to drop
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn drop_database(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  debug!("Dropping PostgreSQL database: {}", name);
  
  // Format the DROP DATABASE SQL statement with proper quoting
  let query = format!("DROP DATABASE \"{}\";", name);
  
  // Execute the query to drop the database
  // This will fail if there are active connections to the database
  client.execute(&query, &[]).await?;
  
  // Log successful database deletion at info level for user visibility
  info!("Dropped database: {}", name);
  debug!("Database drop completed successfully");
  Ok(())
}

/// Drop (delete) a PostgreSQL database with force
/// 
/// This function forcefully deletes the specified database, terminating any active connections.
/// This is a more aggressive version of drop_database that will succeed even with active connections.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `name` - Name of the database to forcefully drop
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn drop_database_with_force(client: &tokio_postgres::Client, name: &str) -> Result<()> {
  debug!("Force dropping PostgreSQL database: {}", name);
  
  // Format the DROP DATABASE SQL statement with the FORCE option
  // The FORCE option terminates any existing connections before dropping the database
  let query = format!("DROP DATABASE \"{}\" WITH (FORCE);", name);
  
  // Execute the query to forcefully drop the database
  client.execute(&query, &[]).await?;
  
  // Log successful forced database deletion at info level for user visibility
  info!("Force dropped database: {}", name);
  debug!("Forced database drop completed successfully");
  Ok(())
}

/// Rename a PostgreSQL database
/// 
/// This function renames an existing database to a new name.
/// The database name is properly quoted to handle special characters.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `old_name` - Current name of the database to rename
/// * `new_name` - New name for the database
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn rename_database(client: &tokio_postgres::Client, old_name: &str, new_name: &str) -> Result<()> {
  debug!("Renaming PostgreSQL database: {} to {}", old_name, new_name);
  
  // Format the ALTER DATABASE SQL statement with proper quoting
  // Double quotes are used to preserve case and allow special characters in database names
  let query = format!("ALTER DATABASE \"{}\" RENAME TO \"{}\";", old_name, new_name);
  
  // Execute the query to rename the database
  match client.execute(&query, &[]).await {
      Ok(_) => debug!("Database renamed successfully"),
      Err(e) => return Err(anyhow!("Failed to rename database: {}", e)),
  };
  
  // Log successful database renaming at info level for user visibility
  info!("Renamed database {} to {}", old_name, new_name);
  debug!("Database renaming completed successfully");
  Ok(())
}

/// Set the owner of a PostgreSQL database
/// 
/// This function changes the ownership of a database to a different user.
/// Both the database name and owner name are properly quoted to handle special characters.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `name` - Name of the database to change ownership of
/// * `owner` - Name of the new owner (must be an existing PostgreSQL role)
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn set_database_owner(client: &tokio_postgres::Client, name: &str, owner: &str) -> Result<()> {
  debug!("Changing owner of PostgreSQL database: {} to {}", name, owner);
  
  // Format the ALTER DATABASE SQL statement with proper quoting for both database and owner names
  // Double quotes are used to preserve case and allow special characters
  let query = format!("ALTER DATABASE \"{}\" OWNER TO \"{}\";", name, owner);
  
  // Execute the query to change the database owner
  match client.execute(&query, &[]).await {
      Ok(_) => debug!("Database owner changed successfully"),
      Err(e) => return Err(anyhow!("Failed to change database owner: {}", e)),
  };

  // Log successful ownership change at info level for user visibility
  info!("Database '{}' owner changed to '{}' successfully", name, owner);
  debug!("Database ownership change completed successfully");
  Ok(())
}

/// Change the password of a PostgreSQL user
/// 
/// This function sets a new password for an existing PostgreSQL user/role.
/// The username is properly quoted to handle special characters.
/// 
/// # Arguments
/// 
/// * `client` - Connected PostgreSQL client
/// * `user` - Name of the user to change password for
/// * `password` - New password to set
/// 
/// # Returns
/// 
/// A Result indicating success or an error
pub async fn change_password(client: &tokio_postgres::Client, user: &str, password: &str) -> Result<()> {
  debug!("Changing password for PostgreSQL user: {}", user);
  
  // Format the ALTER USER SQL statement with proper quoting for the username
  // Note: Password is single-quoted as it's a string literal, not an identifier
  let query = format!("ALTER USER \"{}\" WITH PASSWORD '{}';", user, password);
  
  // Execute the query to change the user's password
  match client.execute(&query, &[]).await {
      Ok(_) => debug!("User password changed successfully"),
      Err(e) => return Err(anyhow!("Failed to change user password: {}", e)),
  };

  // Log successful password change at info level for user visibility
  // Note: We don't log the actual password for security reasons
  info!("Password for user '{}' changed successfully", user);
  debug!("Password change completed successfully");
  Ok(())
}

/// Restore a PostgreSQL database from a snapshot file
/// 
/// This function restores a database from a previously created snapshot file.
/// It creates a new database with a random name, then restores the snapshot into it.
/// 
/// # Arguments
/// 
/// * `host` - PostgreSQL server hostname or IP address
/// * `port` - PostgreSQL server port
/// * `username` - Optional username for authentication
/// * `password` - Optional password for authentication
/// * `use_ssl` - Whether to use SSL for the connection
/// * `file_path` - Path to the snapshot file to restore
/// 
/// # Returns
/// 
/// A Result containing the name of the newly created database or an error
pub async fn restore_snapshot(
    host: &str,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    use_ssl: bool,
    file_path: &str,
) -> Result<String> {
    debug!("Starting database restore from snapshot file: {}", file_path);
    debug!("Connection parameters: host={}, port={}, use_ssl={}", host, port, use_ssl);
    // Create a new database with a random name by combining a random English word with the suffix
    // This ensures the restored database has a unique but recognizable name
    let new_dbname = format!("{}-restored", random_word(Lang::En));
    debug!("Generated new database name for restoration: {}", new_dbname);
    
    // Create a connection configuration to the default postgres database
    // We need to connect to an existing database first before we can create a new one
    debug!("Setting up connection configuration to PostgreSQL server");
    let mut config = PgConfig::new();
    
    // Set the host and port from the provided parameters
    config.host(host);
    config.port(port);
    
    // Add username to the connection configuration if provided
    if let Some(ref user) = username {
        debug!("Using provided username for authentication: {}", user);
        config.user(user);
    } else {
        debug!("No username provided, using default");
    }
    
    // Add password to the connection configuration if provided
    if let Some(ref pass) = password {
        debug!("Using provided password for authentication");
        config.password(pass);
    } else {
        debug!("No password provided, using default or trust authentication");
    }
    
    // Connect to the default postgres database using the appropriate connection method
    // based on whether SSL is required or not
    debug!("Connecting to PostgreSQL server to create new database");
    let client = if use_ssl {
        debug!("Using SSL connection");
        connect_ssl(&config, false, None).await?
    } else {
        debug!("Using non-SSL connection");
        connect_no_ssl(&config).await?
    };
    
    // Create the new database with the randomly generated name
    // This will be the target database for our restoration
    debug!("Creating new database: {}", new_dbname);
    let create_query = format!("CREATE DATABASE \"{}\";", new_dbname);
    match client.execute(&create_query, &[]).await {
        Ok(_) => debug!("Database creation query executed successfully"),
        Err(e) => return Err(anyhow!("Failed to create new database {}: {}", new_dbname, e)),
    };
    debug!("Successfully created new database");
    
    // Close the connection to the default database
    // We need to do this before connecting to our new database
    debug!("Closing connection to default database");
    drop(client);
    
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
                    // Log the successful restoration at info level for user visibility
                    info!("Snapshot restored to database: {}", new_dbname);
                    
                    // Return the name of the newly created and restored database
                    debug!("Database restoration process completed");
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