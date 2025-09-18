use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use serde::{Deserialize, Serialize};
use sqlx::{Column, PgPool, Row, TypeInfo};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio_postgres::{Client, NoTls};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: SslMode,
    pub connection_timeout: u32,
    pub created_at: String,
    pub last_connected: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl Default for SslMode {
    fn default() -> Self {
        SslMode::Prefer
    }
}

impl SslMode {
    pub fn as_str(&self) -> &str {
        match self {
            SslMode::Disable => "disable",
            SslMode::Allow => "allow",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
            SslMode::VerifyCa => "verify-ca",
            SslMode::VerifyFull => "verify-full",
        }
    }

    pub fn all() -> Vec<SslMode> {
        vec![
            SslMode::Disable,
            SslMode::Allow,
            SslMode::Prefer,
            SslMode::Require,
            SslMode::VerifyCa,
            SslMode::VerifyFull,
        ]
    }
}

impl std::fmt::Display for SslMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "New Connection".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            ssl_mode: SslMode::default(),
            connection_timeout: 30,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_connected: None,
            is_active: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionManager {
    pub connections: HashMap<String, DatabaseConnection>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_connection(&mut self, connection: DatabaseConnection) {
        self.connections.insert(connection.id.clone(), connection);
    }

    pub fn update_connection(&mut self, connection: DatabaseConnection) {
        self.connections.insert(connection.id.clone(), connection);
    }

    pub fn remove_connection(&mut self, id: &str) {
        self.connections.remove(id);
    }

    pub fn get_connection(&self, id: &str) -> Option<&DatabaseConnection> {
        self.connections.get(id)
    }

    pub fn get_all_connections(&self) -> Vec<&DatabaseConnection> {
        self.connections.values().collect()
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let manager: ConnectionManager = serde_json::from_str(&content)?;
        Ok(manager)
    }

    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        path.push("rbeaver");
        path.push("connections.json");
        path
    }
}

#[derive(Debug)]
pub enum ConnectionTestResult {
    Success,
    Failed(String),
}

impl DatabaseConnection {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.username, self.password, self.host, self.port, self.database, self.ssl_mode
        )
    }

    pub async fn test_connection(&self) -> ConnectionTestResult {
        match self.validate() {
            Ok(_) => match self.create_tokio_postgres_client().await {
                Ok(_) => ConnectionTestResult::Success,
                Err(e) => ConnectionTestResult::Failed(format!("Connection failed: {}", e)),
            },
            Err(e) => ConnectionTestResult::Failed(e),
        }
    }

    /// Create a tokio-postgres client for direct database operations
    pub async fn create_tokio_postgres_client(&self) -> Result<Client> {
        let config = format!(
            "host={} port={} user={} password={} dbname={} sslmode={}",
            self.host, self.port, self.username, self.password, self.database, self.ssl_mode
        );

        let (client, connection) = tokio_postgres::connect(&config, NoTls).await?;

        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(client)
    }

    /// Create a deadpool connection pool for high-performance applications
    pub async fn create_connection_pool(&self) -> Result<Pool> {
        let mut cfg = Config::new();
        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.user = Some(self.username.clone());
        cfg.password = Some(self.password.clone());
        cfg.dbname = Some(self.database.clone());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        Ok(pool)
    }

    /// Create a SQLx connection pool for query building and migrations
    pub async fn create_sqlx_pool(&self) -> Result<PgPool> {
        let database_url = self.connection_string();
        let pool = PgPool::connect(&database_url).await?;
        Ok(pool)
    }

    /// Test connection using SQLx (alternative method)
    pub async fn test_connection_sqlx(&self) -> ConnectionTestResult {
        match self.validate() {
            Ok(_) => {
                match self.create_sqlx_pool().await {
                    Ok(pool) => {
                        // Test with a simple query
                        match sqlx::query("SELECT 1 as test").fetch_one(&pool).await {
                            Ok(_) => ConnectionTestResult::Success,
                            Err(e) => {
                                ConnectionTestResult::Failed(format!("Query test failed: {}", e))
                            }
                        }
                    }
                    Err(e) => ConnectionTestResult::Failed(format!("Connection failed: {}", e)),
                }
            }
            Err(e) => ConnectionTestResult::Failed(e),
        }
    }

    /// Execute a simple query to get database information
    pub async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let pool = self.create_sqlx_pool().await?;

        let version_row = sqlx::query("SELECT version()").fetch_one(&pool).await?;
        let version: String = version_row.get(0);

        let size_row = sqlx::query("SELECT pg_database_size(current_database())")
            .fetch_one(&pool)
            .await?;
        let size: i64 = size_row.get(0);

        let tables_row = sqlx::query(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'",
        )
        .fetch_one(&pool)
        .await?;
        let table_count: i64 = tables_row.get(0);

        Ok(DatabaseInfo {
            version,
            size_bytes: size,
            table_count,
            connection_id: self.id.clone(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Connection name cannot be empty".to_string());
        }
        if self.host.trim().is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        if self.port == 0 || self.port > 65535 {
            return Err("Port must be between 1 and 65535".to_string());
        }
        if self.database.trim().is_empty() {
            return Err("Database name cannot be empty".to_string());
        }
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.connection_timeout == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn update_last_connected(&mut self) {
        self.last_connected = Some(chrono::Utc::now().to_rfc3339());
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if active {
            self.update_last_connected();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub version: String,
    pub size_bytes: i64,
    pub table_count: i64,
    pub connection_id: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pub connection_manager: ConnectionManager,
    pub active_pools: HashMap<String, PgPool>,
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self {
            connection_manager: ConnectionManager::default(),
            active_pools: HashMap::new(),
        }
    }
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(&mut self, connection_id: &str) -> Result<()> {
        if let Some(conn) = self.connection_manager.get_connection(connection_id) {
            let pool = conn.create_sqlx_pool().await?;
            self.active_pools.insert(connection_id.to_string(), pool);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Connection not found: {}", connection_id))
        }
    }

    pub async fn disconnect(&mut self, connection_id: &str) {
        if let Some(pool) = self.active_pools.remove(connection_id) {
            pool.close().await;
        }
    }

    pub fn get_pool(&self, connection_id: &str) -> Option<&PgPool> {
        self.active_pools.get(connection_id)
    }

    pub async fn execute_query(
        &self,
        connection_id: &str,
        sql: &str,
    ) -> Result<Vec<serde_json::Value>> {
        if let Some(pool) = self.get_pool(connection_id) {
            let rows = sqlx::query(sql).fetch_all(pool).await?;

            let mut results = Vec::new();
            for row in rows {
                let mut json_row = serde_json::Map::new();

                // Convert each column to JSON value
                for (i, column) in row.columns().iter().enumerate() {
                    let column_name = column.name();

                    // Handle different PostgreSQL types
                    let value: serde_json::Value = match column.type_info().name() {
                        "INT4" => {
                            let val: Option<i32> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::Number(v.into()),
                                None => serde_json::Value::Null,
                            }
                        }
                        "INT8" => {
                            let val: Option<i64> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::Number(v.into()),
                                None => serde_json::Value::Null,
                            }
                        }
                        "TEXT" | "VARCHAR" => {
                            let val: Option<String> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::String(v),
                                None => serde_json::Value::Null,
                            }
                        }
                        "BOOL" => {
                            let val: Option<bool> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::Bool(v),
                                None => serde_json::Value::Null,
                            }
                        }
                        "TIMESTAMPTZ" | "TIMESTAMP" => {
                            let val: Option<chrono::DateTime<chrono::Utc>> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::String(v.to_rfc3339()),
                                None => serde_json::Value::Null,
                            }
                        }
                        _ => {
                            // Try to get as string for unknown types
                            let val: Option<String> = row.try_get(i).ok();
                            match val {
                                Some(v) => serde_json::Value::String(v),
                                None => serde_json::Value::Null,
                            }
                        }
                    };

                    json_row.insert(column_name.to_string(), value);
                }

                results.push(serde_json::Value::Object(json_row));
            }

            Ok(results)
        } else {
            Err(anyhow::anyhow!(
                "No active connection found for: {}",
                connection_id
            ))
        }
    }

    pub async fn get_tables(&self, connection_id: &str) -> Result<Vec<TableInfo>> {
        let sql = "
            SELECT
                schemaname,
                tablename,
                tableowner,
                hasindexes,
                hasrules,
                hastriggers
            FROM pg_tables
            WHERE schemaname = 'public'
            ORDER BY tablename
        ";

        if let Some(pool) = self.get_pool(connection_id) {
            let rows = sqlx::query(sql).fetch_all(pool).await?;

            let mut tables = Vec::new();
            for row in rows {
                let table = TableInfo {
                    schema: row.get("schemaname"),
                    name: row.get("tablename"),
                    owner: row.get("tableowner"),
                    has_indexes: row.get("hasindexes"),
                    has_rules: row.get("hasrules"),
                    has_triggers: row.get("hastriggers"),
                };
                tables.push(table);
            }

            Ok(tables)
        } else {
            Err(anyhow::anyhow!(
                "No active connection found for: {}",
                connection_id
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    pub owner: String,
    pub has_indexes: bool,
    pub has_rules: bool,
    pub has_triggers: bool,
}

/// Utility functions for database operations
pub mod utils {
    use super::*;

    pub async fn test_all_connections(
        manager: &ConnectionManager,
    ) -> HashMap<String, ConnectionTestResult> {
        let mut results = HashMap::new();

        for (id, connection) in &manager.connections {
            let result = connection.test_connection().await;
            results.insert(id.clone(), result);
        }

        results
    }

    pub fn format_connection_string(
        connection: &DatabaseConnection,
        hide_password: bool,
    ) -> String {
        let password = if hide_password {
            "****"
        } else {
            &connection.password
        };
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            connection.username,
            password,
            connection.host,
            connection.port,
            connection.database,
            connection.ssl_mode
        )
    }

    pub fn get_default_postgresql_port() -> u16 {
        5432
    }

    pub fn validate_postgresql_identifier(identifier: &str) -> Result<(), String> {
        if identifier.is_empty() {
            return Err("Identifier cannot be empty".to_string());
        }

        if identifier.len() > 63 {
            return Err("Identifier too long (max 63 characters)".to_string());
        }

        if !identifier.chars().next().unwrap().is_ascii_alphabetic() && !identifier.starts_with('_')
        {
            return Err("Identifier must start with a letter or underscore".to_string());
        }

        if !identifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(
                "Identifier can only contain letters, numbers, and underscores".to_string(),
            );
        }

        Ok(())
    }
}
