use crate::database::{DatabaseConnection, DatabaseManager, SslMode};
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

/// Test utilities for database connections
pub struct DatabaseTest;

impl DatabaseTest {
    /// Create a test PostgreSQL connection with default settings
    pub fn create_test_connection() -> DatabaseConnection {
        DatabaseConnection {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Test Connection".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: "password".to_string(),
            ssl_mode: SslMode::Prefer,
            connection_timeout: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_connected: None,
            is_active: false,
        }
    }

    /// Create a test connection with custom parameters
    pub fn create_custom_connection(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> DatabaseConnection {
        DatabaseConnection {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("Test - {}", database),
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            ssl_mode: SslMode::Prefer,
            connection_timeout: 10,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_connected: None,
            is_active: false,
        }
    }

    /// Test basic connection functionality
    pub async fn test_connection_basic(connection: &DatabaseConnection) -> Result<bool> {
        println!(
            "Testing connection to {}:{}/{}",
            connection.host, connection.port, connection.database
        );

        match timeout(
            Duration::from_secs(connection.connection_timeout as u64),
            connection.test_connection(),
        )
        .await
        {
            Ok(result) => match result {
                crate::database::ConnectionTestResult::Success => {
                    println!("✓ Connection test successful");
                    Ok(true)
                }
                crate::database::ConnectionTestResult::Failed(error) => {
                    println!("✗ Connection test failed: {}", error);
                    Ok(false)
                }
            },
            Err(_) => {
                println!("✗ Connection test timed out");
                Ok(false)
            }
        }
    }

    /// Test SQLx connection functionality
    pub async fn test_sqlx_connection(connection: &DatabaseConnection) -> Result<bool> {
        println!(
            "Testing SQLx connection to {}:{}/{}",
            connection.host, connection.port, connection.database
        );

        match timeout(
            Duration::from_secs(connection.connection_timeout as u64),
            connection.test_connection_sqlx(),
        )
        .await
        {
            Ok(result) => match result {
                crate::database::ConnectionTestResult::Success => {
                    println!("✓ SQLx connection test successful");
                    Ok(true)
                }
                crate::database::ConnectionTestResult::Failed(error) => {
                    println!("✗ SQLx connection test failed: {}", error);
                    Ok(false)
                }
            },
            Err(_) => {
                println!("✗ SQLx connection test timed out");
                Ok(false)
            }
        }
    }

    /// Test database information retrieval
    pub async fn test_database_info(connection: &DatabaseConnection) -> Result<()> {
        println!("Testing database info retrieval...");

        match timeout(Duration::from_secs(30), connection.get_database_info()).await {
            Ok(Ok(info)) => {
                println!("✓ Database info retrieved successfully:");
                println!("  Version: {}", info.version);
                println!("  Size: {} bytes", info.size_bytes);
                println!("  Tables: {}", info.table_count);
            }
            Ok(Err(error)) => {
                println!("✗ Failed to retrieve database info: {}", error);
            }
            Err(_) => {
                println!("✗ Database info retrieval timed out");
            }
        }

        Ok(())
    }

    /// Test database manager functionality
    pub async fn test_database_manager() -> Result<()> {
        println!("Testing DatabaseManager functionality...");

        let mut manager = DatabaseManager::new();
        let connection = Self::create_test_connection();
        let connection_id = connection.id.clone();

        // Add connection to manager
        manager.connection_manager.add_connection(connection);
        println!("✓ Connection added to manager");

        // Test connection through manager
        match manager.connect(&connection_id).await {
            Ok(_) => {
                println!("✓ Connected through manager");

                // Test query execution
                match manager.execute_query(&connection_id, "SELECT 1 as test_column, 'Hello' as message, true as flag, NOW() as timestamp").await {
                    Ok(results) => {
                        println!("✓ Query executed successfully:");
                        for (i, row) in results.iter().enumerate() {
                            println!("  Row {}: {}", i + 1, serde_json::to_string_pretty(row)?);
                        }
                    }
                    Err(error) => {
                        println!("✗ Query execution failed: {}", error);
                    }
                }

                // Test table listing
                match manager.get_tables(&connection_id).await {
                    Ok(tables) => {
                        println!("✓ Tables retrieved: {} tables found", tables.len());
                        for table in tables.iter().take(5) {
                            println!("  Table: {}.{}", table.schema, table.name);
                        }
                    }
                    Err(error) => {
                        println!("✗ Failed to retrieve tables: {}", error);
                    }
                }

                // Disconnect
                manager.disconnect(&connection_id).await;
                println!("✓ Disconnected from database");
            }
            Err(error) => {
                println!("✗ Failed to connect through manager: {}", error);
            }
        }

        Ok(())
    }

    /// Run comprehensive database tests
    pub async fn run_comprehensive_tests() -> Result<()> {
        println!("=== Running Comprehensive Database Tests ===\n");

        let test_connection = Self::create_test_connection();

        // Test 1: Basic connection test
        println!("1. Testing basic connection...");
        Self::test_connection_basic(&test_connection).await?;
        println!();

        // Test 2: SQLx connection test
        println!("2. Testing SQLx connection...");
        Self::test_sqlx_connection(&test_connection).await?;
        println!();

        // Test 3: Database info retrieval
        println!("3. Testing database info retrieval...");
        Self::test_database_info(&test_connection).await?;
        println!();

        // Test 4: Database manager
        println!("4. Testing database manager...");
        Self::test_database_manager().await?;
        println!();

        println!("=== Database Tests Completed ===");
        Ok(())
    }

    /// Test connection with various SSL modes
    pub async fn test_ssl_modes(base_connection: &DatabaseConnection) -> Result<()> {
        println!("Testing different SSL modes...");

        for ssl_mode in SslMode::all() {
            let mut connection = base_connection.clone();
            connection.ssl_mode = ssl_mode.clone();
            connection.name = format!("Test SSL - {}", ssl_mode);

            println!("Testing SSL mode: {}", ssl_mode);
            match Self::test_connection_basic(&connection).await {
                Ok(true) => println!("  ✓ SSL mode {} works", ssl_mode),
                Ok(false) => println!("  ✗ SSL mode {} failed", ssl_mode),
                Err(e) => println!("  ✗ SSL mode {} error: {}", ssl_mode, e),
            }
        }

        Ok(())
    }

    /// Test connection validation
    pub fn test_connection_validation() -> Result<()> {
        println!("Testing connection validation...");

        // Valid connection
        let valid_connection = Self::create_test_connection();
        match valid_connection.validate() {
            Ok(_) => println!("✓ Valid connection passed validation"),
            Err(e) => println!("✗ Valid connection failed validation: {}", e),
        }

        // Invalid connections
        let test_cases = vec![
            (
                "empty name",
                DatabaseConnection {
                    name: "".to_string(),
                    ..valid_connection.clone()
                },
            ),
            (
                "empty host",
                DatabaseConnection {
                    host: "".to_string(),
                    ..valid_connection.clone()
                },
            ),
            (
                "invalid port",
                DatabaseConnection {
                    port: 0,
                    ..valid_connection.clone()
                },
            ),
            (
                "empty database",
                DatabaseConnection {
                    database: "".to_string(),
                    ..valid_connection.clone()
                },
            ),
            (
                "empty username",
                DatabaseConnection {
                    username: "".to_string(),
                    ..valid_connection.clone()
                },
            ),
            (
                "zero timeout",
                DatabaseConnection {
                    connection_timeout: 0,
                    ..valid_connection.clone()
                },
            ),
        ];

        for (test_name, invalid_connection) in test_cases {
            match invalid_connection.validate() {
                Ok(_) => println!("✗ {} should have failed validation", test_name),
                Err(e) => println!("✓ {} correctly failed validation: {}", test_name, e),
            }
        }

        Ok(())
    }

    /// Test connection string generation
    pub fn test_connection_string() -> Result<()> {
        println!("Testing connection string generation...");

        let connection = Self::create_custom_connection(
            "test.example.com",
            5432,
            "testdb",
            "testuser",
            "testpass",
        );

        let conn_str = connection.connection_string();
        let expected = "postgresql://testuser:testpass@test.example.com:5432/testdb?sslmode=prefer";

        if conn_str == expected {
            println!("✓ Connection string generated correctly");
            println!("  Generated: {}", conn_str);
        } else {
            println!("✗ Connection string mismatch");
            println!("  Expected: {}", expected);
            println!("  Generated: {}", conn_str);
        }

        // Test with hidden password
        let hidden_str = crate::database::utils::format_connection_string(&connection, true);
        let expected_hidden =
            "postgresql://testuser:****@test.example.com:5432/testdb?sslmode=prefer";

        if hidden_str == expected_hidden {
            println!("✓ Hidden password connection string generated correctly");
            println!("  Generated: {}", hidden_str);
        } else {
            println!("✗ Hidden password connection string mismatch");
            println!("  Expected: {}", expected_hidden);
            println!("  Generated: {}", hidden_str);
        }

        Ok(())
    }

    /// Run quick validation tests that don't require a database
    pub fn run_offline_tests() -> Result<()> {
        println!("=== Running Offline Database Tests ===\n");

        println!("1. Testing connection validation...");
        Self::test_connection_validation()?;
        println!();

        println!("2. Testing connection string generation...");
        Self::test_connection_string()?;
        println!();

        println!("=== Offline Tests Completed ===");
        Ok(())
    }
}

/// Example usage and test runner
pub async fn run_database_tests() {
    println!("Starting database tests...\n");

    // Run offline tests first (no database required)
    if let Err(e) = DatabaseTest::run_offline_tests() {
        eprintln!("Offline tests failed: {}", e);
    }

    println!();

    // Run online tests (require database connection)
    // Note: These will fail if no PostgreSQL database is available
    println!("Attempting online tests (may fail if no database is available)...");
    if let Err(e) = DatabaseTest::run_comprehensive_tests().await {
        println!("Online tests failed (expected if no database): {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let conn = DatabaseTest::create_test_connection();
        assert!(!conn.id.is_empty());
        assert_eq!(conn.name, "Test Connection");
        assert_eq!(conn.host, "localhost");
        assert_eq!(conn.port, 5432);
    }

    #[test]
    fn test_custom_connection_creation() {
        let conn =
            DatabaseTest::create_custom_connection("custom.host", 3306, "customdb", "user", "pass");
        assert_eq!(conn.host, "custom.host");
        assert_eq!(conn.port, 3306);
        assert_eq!(conn.database, "customdb");
        assert_eq!(conn.username, "user");
        assert_eq!(conn.password, "pass");
    }

    #[test]
    fn test_validation_offline() {
        assert!(DatabaseTest::test_connection_validation().is_ok());
    }

    #[test]
    fn test_connection_string_offline() {
        assert!(DatabaseTest::test_connection_string().is_ok());
    }
}
