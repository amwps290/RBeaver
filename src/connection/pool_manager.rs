//! 连接池管理模块

use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;
use std::time::Duration;

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最小连接数
    pub min_size: u32,
    /// 最大连接数
    pub max_size: u32,
    /// 空闲超时时间
    pub idle_timeout: Duration,
    /// 最大生存时间
    pub max_lifetime: Duration,
    /// 连接超时时间
    pub connect_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 20,
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            connect_timeout: Duration::from_secs(30),
        }
    }
}

impl PoolConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最小连接数
    pub fn min_connections(mut self, size: u32) -> Self {
        self.min_size = size;
        self
    }

    /// 设置最大连接数
    pub fn max_connections(mut self, size: u32) -> Self {
        self.max_size = size;
        self
    }

    /// 设置空闲超时
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// 设置最大生存时间
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// 设置连接超时
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
}

/// 连接池管理器
#[derive(Clone)]
pub struct ConnectionPoolManager {
    /// 连接池缓存
    pools: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, Pool<PostgresConnectionManager<NoTls>>>>>,
    /// 池配置
    config: PoolConfig,
}

impl ConnectionPoolManager {
    /// 创建新的连接池管理器
    pub fn new(config: PoolConfig) -> Self {
        Self {
            pools: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// 使用默认配置创建连接池管理器
    pub fn new_with_defaults() -> Self {
        Self::new(PoolConfig::default())
    }

    /// 创建连接池
    pub fn create_pool(
        &self,
        db_config: &crate::database::DatabaseConnection,
    ) -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn std::error::Error>> {
        let config = format!(
            "host={} port={} user={} password={} dbname={} sslmode={}",
            db_config.host, db_config.port, db_config.username, db_config.password, db_config.database, db_config.ssl_mode
        );

        let manager = PostgresConnectionManager::new(config.parse()?, NoTls);
        let pool = Pool::new(manager)?;

        Ok(pool)
    }

    /// 获取连接池
    pub fn get_pool(&self, connection_id: &str) -> Option<Pool<PostgresConnectionManager<NoTls>>> {
        let pools = self.pools.read().unwrap();
        pools.get(connection_id).cloned()
    }

    /// 添加连接池
    pub fn add_pool(&self, connection_id: String, pool: Pool<PostgresConnectionManager<NoTls>>) {
        let mut pools = self.pools.write().unwrap();
        pools.insert(connection_id, pool);
    }

    /// 移除连接池
    pub fn remove_pool(&self, connection_id: &str) -> Option<Pool<PostgresConnectionManager<NoTls>>> {
        let mut pools = self.pools.write().unwrap();
        pools.remove(connection_id)
    }

    /// 健康检查
    pub fn check_health(&self, pool: &Pool<PostgresConnectionManager<NoTls>>) -> bool {
        match pool.get() {
            Ok(mut client) => {
                match client.query("SELECT 1", &[]) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// 获取所有连接池 ID
    pub fn get_all_pool_ids(&self) -> Vec<String> {
        let pools = self.pools.read().unwrap();
        pools.keys().cloned().collect()
    }

    /// 清空所有连接池
    pub fn clear_all(&self) {
        let mut pools = self.pools.write().unwrap();
        pools.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, 5);
        assert_eq!(config.max_size, 20);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new()
            .min_connections(10)
            .max_connections(50)
            .idle_timeout(std::time::Duration::from_secs(300));

        assert_eq!(config.min_size, 10);
        assert_eq!(config.max_size, 50);
        assert_eq!(config.idle_timeout, std::time::Duration::from_secs(300));
    }

    #[test]
    fn test_connection_id() {
        let id1 = ConnectionId::new();
        let id2 = ConnectionId::new();
        assert_ne!(id1.0, id2.0);
    }

    #[test]
    fn test_component_id() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();
        assert_ne!(id1.0, id2.0);
    }
}
