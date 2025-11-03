use anyhow::Result;
use postgres::{Client, NoTls};
use r2d2::{Pool};
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::database_structure::{
    DatabaseObjectType, DatabaseStructureQuery, DatabaseTreeNode,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
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
        self.connections.insert(connection.name.clone(), connection);
    }

    pub fn update_connection(&mut self, connection: DatabaseConnection) {
        self.connections.insert(connection.name.clone(), connection);
    }

    pub fn remove_connection(&mut self, name: &str) {
        self.connections.remove(name);
    }

    pub fn get_connection(&self, name: &str) -> Option<&DatabaseConnection> {
        self.connections.get(name)
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

    pub fn test_connection(&self) -> ConnectionTestResult {
        match self.validate() {
            Ok(_) => match self.create_client() {
                Ok(_) => ConnectionTestResult::Success,
                Err(e) => ConnectionTestResult::Failed(format!("Connection failed: {}", e)),
            },
            Err(e) => ConnectionTestResult::Failed(e),
        }
    }

    /// Create a synchronous postgres client for database operations
    pub fn create_client(&self) -> Result<Client> {
        let config = format!(
            "host={} port={} user={} password={} dbname={} sslmode={}",
            self.host, self.port, self.username, self.password, self.database, self.ssl_mode
        );

        let client = Client::connect(&config, NoTls)?;
        Ok(client)
    }

    /// Create a r2d2 connection pool for high-performance applications
    pub fn create_connection_pool(&self) -> Result<Pool<PostgresConnectionManager<NoTls>>> {
        let config = format!(
            "host={} port={} user={} password={} dbname={} sslmode={}",
            self.host, self.port, self.username, self.password, self.database, self.ssl_mode
        );

        let manager = PostgresConnectionManager::new(config.parse()?, NoTls);
        let pool = Pool::new(manager)?;
        Ok(pool)
    }

    /// Test connection using synchronous client
    pub fn test_connection_sync(&self) -> ConnectionTestResult {
        match self.validate() {
            Ok(_) => {
                match self.create_client() {
                    Ok(_) => ConnectionTestResult::Success,
                    Err(e) => ConnectionTestResult::Failed(format!("Connection failed: {}", e)),
                }
            }
            Err(e) => ConnectionTestResult::Failed(e),
        }
    }

    /// Execute a simple query to get database information
    pub fn get_database_info(&self) -> Result<DatabaseInfo> {
        let mut client = self.create_client()?;

        let version_row = client.query_one("SELECT version()", &[])?;
        let version: String = version_row.get(0);

        let size_row = client.query_one("SELECT pg_database_size(current_database())", &[])?;
        let size: i64 = size_row.get(0);

        let tables_row = client.query_one(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'",
            &[],
        )?;
        let table_count: i64 = tables_row.get(0);

        Ok(DatabaseInfo {
            version,
            size_bytes: size,
            table_count,
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
}

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pub connection_manager: ConnectionManager,
    pub active_pools: HashMap<String, Pool<PostgresConnectionManager<NoTls>>>,
    pub database_structures: HashMap<String, DatabaseTreeNode>,
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self {
            connection_manager: ConnectionManager::new(),
            active_pools: HashMap::new(),
            database_structures: HashMap::new(),
        }
    }
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&mut self, connection_id: &str) -> Result<()> {
        if let Some(conn) = self.connection_manager.get_connection(connection_id) {
            let pool = conn.create_connection_pool()?;
            self.active_pools.insert(connection_id.to_string(), pool);

            // Load database structure after successful connection
            let _ = self.load_database_structure(connection_id);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Connection not found: {}", connection_id))
        }
    }

    pub fn disconnect(&mut self, connection_id: &str) {
        if let Some(pool) = self.active_pools.remove(connection_id) {
            drop(pool);
        }
        self.database_structures.remove(connection_id);
    }

    pub fn get_pool(&self, connection_id: &str) -> Option<&Pool<PostgresConnectionManager<NoTls>>> {
        self.active_pools.get(connection_id)
    }

    pub fn get_pooled_client(&self, connection_id: &str) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        if let Some(pool) = self.get_pool(connection_id) {
            Ok(pool.get()?)
        } else {
            Err(anyhow::anyhow!(
                "No active connection found for: {}",
                connection_id
            ))
        }
    }

    pub fn execute_query(
        &self,
        connection_id: &str,
        sql: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let mut pool = self.get_pooled_client(connection_id)?;
        let rows = pool.query(sql, &[])?;

        let mut results = Vec::new();
        for row in rows {
            let mut json_row = serde_json::Map::new();

            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                let value: serde_json::Value = match column.type_().name() {
                    "int4" => {
                        let val: Option<i32> = row.get(i);
                        match val {
                            Some(v) => serde_json::Value::Number(v.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    "int8" => {
                        let val: Option<i64> = row.get(i);
                        match val {
                            Some(v) => serde_json::Value::Number(v.into()),
                            None => serde_json::Value::Null,
                        }
                    }
                    "text" | "varchar" => {
                        let val: Option<String> = row.get(i);
                        match val {
                            Some(v) => serde_json::Value::String(v),
                            None => serde_json::Value::Null,
                        }
                    }
                    "bool" => {
                        let val: Option<bool> = row.get(i);
                        match val {
                            Some(v) => serde_json::Value::Bool(v),
                            None => serde_json::Value::Null,
                        }
                    }
                    "timestamptz" | "timestamp" => {
                        let val: Option<String> = row.get(i);
                        match val {
                            Some(v) => serde_json::Value::String(v),
                            None => serde_json::Value::Null,
                        }
                    }
                    _ => {
                        let val: Option<String> = row.get(i);
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
    }

    pub fn get_tables(&self, connection_id: &str) -> Result<Vec<TableInfo>> {
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

        let mut pool = self.get_pooled_client(connection_id)?;
        let rows = pool.query(sql, &[])?;

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
    }

    /// 加载数据库结构
    pub fn load_database_structure(&mut self, connection_id: &str) -> Result<()> {
        if let Some(pool) = self.get_pool(connection_id) {
            let mut root = DatabaseTreeNode::new(
                connection_id.to_string(),
                format!("Database ({})", connection_id),
                DatabaseObjectType::Schema,
            );

            let mut client = pool.get()?;

            // 获取schemas
            let schemas = DatabaseStructureQuery::get_schemas(&mut client)?;

            for schema in schemas {
                let mut schema_node = DatabaseTreeNode::new(
                    format!("{}:schema:{}", connection_id, schema.name),
                    schema.name.clone(),
                    DatabaseObjectType::Schema,
                );

                let object_types = vec![
                    DatabaseObjectType::Extension,
                    DatabaseObjectType::Table,
                    DatabaseObjectType::View,
                    DatabaseObjectType::Index,
                    DatabaseObjectType::Type,
                    DatabaseObjectType::Function,
                ];

                for obj_type in object_types {
                    let type_node = DatabaseTreeNode::new(
                        format!("{}:{}:{}", connection_id, obj_type.as_str(), schema.name),
                        obj_type.display_name().to_string(),
                        obj_type,
                    );
                    schema_node.add_child(type_node);
                }

                root.add_child(schema_node);
            }

            // 添加扩展节点
            let extensions = DatabaseStructureQuery::get_extensions(&mut client)?;
            if !extensions.is_empty() {
                let mut ext_node = DatabaseTreeNode::new(
                    format!("{}:extensions", connection_id),
                    "Extensions".to_string(),
                    DatabaseObjectType::Extension,
                );

                for extension in extensions {
                    let ext_item = DatabaseTreeNode::new(
                        format!("{}:extension:{}", connection_id, extension.name),
                        format!("{} ({})", extension.name, extension.version),
                        DatabaseObjectType::Extension,
                    );
                    ext_node.add_child(ext_item);
                }

                root.add_child(ext_node);
            }

            self.database_structures
                .insert(connection_id.to_string(), root);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "No active connection found for: {}",
                connection_id
            ))
        }
    }

    /// 获取数据库结构树
    pub fn get_database_structure(&self, connection_id: &str) -> Option<&DatabaseTreeNode> {
        self.database_structures.get(connection_id)
    }

    /// 加载特定类型的对象
    pub fn load_objects(
        &mut self,
        connection_id: &str,
        schema: &str,
        object_type: DatabaseObjectType,
    ) -> Result<Vec<DatabaseTreeNode>> {
        if let Some(pool) = self.get_pool(connection_id) {
            let mut client = pool.get()?;
            let mut objects = Vec::new();

            match object_type {
                DatabaseObjectType::Table => {
                    let tables = DatabaseStructureQuery::get_tables(&mut client, Some(schema))?;
                    for table in tables {
                        let table_node = DatabaseTreeNode::new(
                            format!("{}:table:{}:{}", connection_id, schema, table.name),
                            table.name,
                            DatabaseObjectType::Table,
                        );
                        objects.push(table_node);
                    }
                }
                DatabaseObjectType::Function => {
                    let functions =
                        DatabaseStructureQuery::get_functions(&mut client, Some(schema))?;
                    for function in functions {
                        let func_node = DatabaseTreeNode::new(
                            format!("{}:function:{}:{}", connection_id, schema, function.name),
                            format!("{} ({})", function.name, function.return_type),
                            DatabaseObjectType::Function,
                        );
                        objects.push(func_node);
                    }
                }
                DatabaseObjectType::Index => {
                    let indexes = DatabaseStructureQuery::get_indexes(&mut client, Some(schema))?;
                    for index in indexes {
                        let index_node = DatabaseTreeNode::new(
                            format!("{}:index:{}:{}", connection_id, schema, index.index_name),
                            format!("{} ({})", index.index_name, index.table_name),
                            DatabaseObjectType::Index,
                        );
                        objects.push(index_node);
                    }
                }
                DatabaseObjectType::Type => {
                    let types = DatabaseStructureQuery::get_types(&mut client, Some(schema))?;
                    for type_info in types {
                        let type_node = DatabaseTreeNode::new(
                            format!("{}:type:{}:{}", connection_id, schema, type_info.name),
                            format!("{} ({})", type_info.name, type_info.type_category),
                            DatabaseObjectType::Type,
                        );
                        objects.push(type_node);
                    }
                }
                _ => {}
            }

            Ok(objects)
        } else {
            Err(anyhow::anyhow!(
                "No active connection found for: {}",
                connection_id
            ))
        }
    }

    /// 检查连接是否活跃
    pub fn is_connected(&self, connection_id: &str) -> bool {
        self.active_pools.contains_key(connection_id)
    }

    /// 获取活跃连接列表
    pub fn get_active_connections(&self) -> Vec<String> {
        self.active_pools.keys().cloned().collect()
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

    pub fn test_all_connections(
        manager: &ConnectionManager,
    ) -> HashMap<String, ConnectionTestResult> {
        let mut results = HashMap::new();

        for (id, connection) in &manager.connections {
            let result = connection.test_connection();
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
