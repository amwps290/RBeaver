//! 全局连接管理器

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::database::{DatabaseConnection, DatabaseManager};

/// 全局连接管理器 - 单例模式
/// 负责管理所有数据库连接的生命周期
#[derive(Clone)]
pub struct GlobalConnectionManager {
    /// 连接注册表 - 管理所有连接的元信息
    connection_registry: Arc<Mutex<HashMap<ConnectionId, ConnectionContext>>>,
    /// 连接池管理器
    pool_manager: Arc<ConnectionPoolManager>,
    /// 配置持久化存储
    config_store: Arc<ConnectionConfigStore>,
    /// 事件广播器
    event_bus: Arc<EventBus>,
}

/// 连接配置持久化存储
pub struct ConnectionConfigStore {
    /// 配置文件路径
    config_path: std::path::PathBuf,
}

impl ConnectionConfigStore {
    /// 创建新的配置存储
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("rbeaver");

        // 确保目录存在
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create config directory: {}", e);
            });
        }

        let config_path = config_dir.join("connections.json");

        Self { config_path }
    }

    /// 保存连接配置
    pub fn save_connection(
        &self,
        connection_id: &ConnectionId,
        config: &DatabaseConnection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.load_all()?;
        let mut connections = connections.unwrap_or_else(|| HashMap::new());
        connections.insert(connection_id.clone(), config.clone());

        eprintln!("[ConnectionConfigStore] Saving connection '{}' to '{}'", connection_id.as_str(), self.config_path.display());
        eprintln!("[ConnectionConfigStore] Total connections in file: {}", connections.len());

        let json = serde_json::to_string_pretty(&connections)?;
        std::fs::write(&self.config_path, json)?;

        eprintln!("[ConnectionConfigStore] Successfully saved connection");

        Ok(())
    }

    /// 删除连接配置
    pub fn delete_connection(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = match self.load_all()? {
            Some(conns) => conns,
            None => return Ok(()),
        };

        connections.remove(connection_id);

        let json = serde_json::to_string_pretty(&connections)?;
        std::fs::write(&self.config_path, json)?;

        Ok(())
    }

    /// 加载所有连接配置
    pub fn load_all(&self) -> Result<Option<HashMap<ConnectionId, DatabaseConnection>>, Box<dyn std::error::Error>> {
        if !self.config_path.exists() {
            eprintln!("[ConnectionConfigStore] Config file does not exist: '{}'", self.config_path.display());
            return Ok(None);
        }

        eprintln!("[ConnectionConfigStore] Loading connections from '{}'", self.config_path.display());
        let content = std::fs::read_to_string(&self.config_path)?;
        let string_connections: HashMap<String, DatabaseConnection> = serde_json::from_str(&content)?;

        let connections: HashMap<ConnectionId, DatabaseConnection> = string_connections
            .into_iter()
            .map(|(id, config)| (ConnectionId::from_string(id), config))
            .collect();

        eprintln!("[ConnectionConfigStore] Loaded {} connections", connections.len());

        Ok(Some(connections))
    }

    /// 加载单个连接配置
    pub fn load_connection(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<Option<DatabaseConnection>, Box<dyn std::error::Error>> {
        let connections = self.load_all()?;
        Ok(connections.and_then(|mut conns| conns.remove(connection_id)))
    }
}

/// 简单的事件总线
#[derive(Clone, Default)]
pub struct EventBus {
    /// 事件订阅者
    subscribers: Arc<std::sync::Mutex<Vec<Box<dyn ConnectionEventSubscriber + Send + Sync>>>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self::default()
    }

    /// 订阅事件
    pub fn subscribe(&self, subscriber: Box<dyn ConnectionEventSubscriber + Send + Sync>) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(subscriber);
    }

    /// 发布事件
    pub fn emit(&self, event: ConnectionEvent) {
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            subscriber.on_event(&event);
        }
    }
}

/// 连接事件订阅者
pub trait ConnectionEventSubscriber {
    /// 处理事件
    fn on_event(&self, event: &ConnectionEvent);
}

/// 旧的数据库管理器兼容性别名
pub type ConnectionManager = GlobalConnectionManager;

impl GlobalConnectionManager {
    /// 获取全局连接管理器实例（单例）
    pub fn get() -> Arc<GlobalConnectionManager> {
        use std::sync::{OnceLock, Mutex};
        static INSTANCE: OnceLock<Mutex<Option<Arc<GlobalConnectionManager>>>> = OnceLock::new();

        let instance = INSTANCE.get_or_init(|| {
            Mutex::new(Some(Arc::new(Self::new())))
        });

        // 获取锁并克隆实例
        let lock = instance.lock().unwrap();
        lock.as_ref().unwrap().clone()
    }

    /// 创建新的连接管理器实例
    pub fn new() -> Self {
        let config_store = Arc::new(ConnectionConfigStore::new());
        let event_bus = Arc::new(EventBus::new());

        Self {
            connection_registry: Arc::new(Mutex::new(HashMap::new())),
            pool_manager: Arc::new(ConnectionPoolManager::new_with_defaults()),
            config_store,
            event_bus,
        }
    }

    /// 创建新的连接
    pub fn create_connection(
        &self,
        config: DatabaseConnection,
    ) -> Result<ConnectionId, Box<dyn std::error::Error>> {
        let id = ConnectionId::new();
        let context = ConnectionContext::new(id.clone(), config.clone());

        {
            let mut registry = self.connection_registry.lock().unwrap();
            registry.insert(id.clone(), context);
        }

        // 持久化配置
        self.config_store.save_connection(&id, &config)?;

        // 通知事件
        self.event_bus.emit(ConnectionEvent::Created(id.clone()));

        Ok(id)
    }

    /// 从配置加载连接
    pub fn load_connections(&self) -> Result<Vec<ConnectionId>, Box<dyn std::error::Error>> {
        eprintln!("[GlobalConnectionManager] Loading connections from config store");
        let connections = self.config_store.load_all()?;

        if let Some(connections) = connections {
            let mut registry = self.connection_registry.lock().unwrap();

            let ids: Vec<ConnectionId> = connections
                .into_iter()
                .map(|(id, config)| {
                    eprintln!("[GlobalConnectionManager] Loading connection: {} -> {}", id.as_str(), config.name);
                    let context = ConnectionContext::new(id.clone(), config);
                    registry.insert(id.clone(), context);
                    id
                })
                .collect();

            eprintln!("[GlobalConnectionManager] Successfully loaded {} connections", ids.len());
            Ok(ids)
        } else {
            eprintln!("[GlobalConnectionManager] No connections found in config");
            Ok(vec![])
        }
    }

    /// 获取连接池（延迟创建）
    pub fn get_pool(&self, connection_id: &ConnectionId) -> Result<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, Box<dyn std::error::Error>> {
        let mut registry = self.connection_registry.lock().unwrap();

        if let Some(context) = registry.get_mut(connection_id) {
            // 如果池不存在，创建它
            if context.pool.is_none() {
                let pool = self.pool_manager.create_pool(&context.config)?;
                context.pool = Some(pool);
            }

            context.last_used = std::time::Instant::now().into();
            Ok(context.pool.as_ref().unwrap().clone())
        } else {
            Err(format!("Connection not found: {}", connection_id.0).into())
        }
    }

    /// 获取连接上下文
    pub fn get_context(&self, connection_id: &ConnectionId) -> Option<ConnectionContext> {
        let registry = self.connection_registry.lock().unwrap();
        registry.get(connection_id).cloned()
    }

    /// 获取所有连接 ID
    pub fn get_all_connections(&self) -> Vec<ConnectionId> {
        let registry = self.connection_registry.lock().unwrap();
        registry.keys().cloned().collect()
    }

    /// 断开连接
    pub fn disconnect(&self, connection_id: &ConnectionId) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.connection_registry.lock().unwrap();

        if let Some(context) = registry.get_mut(connection_id) {
            // 关闭连接池
            if let Some(pool) = context.pool.take() {
                drop(pool);
            }
            context.is_active = false;

            self.event_bus.emit(ConnectionEvent::Disconnected(connection_id.clone()));
        }

        Ok(())
    }

    /// 删除连接
    pub fn delete_connection(&self, connection_id: &ConnectionId) -> Result<(), Box<dyn std::error::Error>> {
        // 先断开
        self.disconnect(connection_id)?;

        // 从注册表移除
        {
            let mut registry = self.connection_registry.lock().unwrap();
            registry.remove(connection_id);
        }

        // 从持久化存储移除
        self.config_store.delete_connection(connection_id)?;

        // 通知删除
        self.event_bus.emit(ConnectionEvent::Deleted(connection_id.clone()));

        Ok(())
    }

    /// 将组件绑定到连接
    pub fn bind_component(
        &self,
        component_id: ComponentId,
        connection_id: ConnectionId,
        binding_type: BindingType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.connection_registry.lock().unwrap();

        if let Some(context) = registry.get_mut(&connection_id) {
            // 如果是独占连接，移除其他组件的绑定
            if matches!(binding_type, BindingType::Exclusive) {
                context.attached_components.clear();
            }

            // 添加当前组件绑定
            if !context.attached_components.contains(&component_id) {
                context.attached_components.push(component_id.clone());
            }

            // 标记为活跃
            let old_active = context.is_active;
            context.is_active = true;

            // 如果连接池不存在，创建它
            if context.pool.is_none() {
                let pool = self.pool_manager.create_pool(&context.config)?;
                context.pool = Some(pool);
            }

            // 通知状态变化
            if !old_active {
                self.event_bus.emit(ConnectionEvent::StateChanged {
                    connection_id: connection_id.clone(),
                    old_state: old_active,
                    new_state: true,
                });
            }

            self.event_bus.emit(ConnectionEvent::ComponentBound {
                connection_id: connection_id.clone(),
                component_id,
                binding_type,
            });
        }

        Ok(())
    }

    /// 解绑组件
    pub fn unbind_component(
        &self,
        component_id: &ComponentId,
        connection_id: &ConnectionId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.connection_registry.lock().unwrap();

        if let Some(context) = registry.get_mut(connection_id) {
            // 移除组件绑定
            context.attached_components.retain(|id| id != component_id);

            // 检查是否还有活跃组件
            let old_active = context.is_active;
            let new_active = context.has_active_components();

            if old_active && !new_active {
                context.is_active = false;
                // 延迟断开，让连接池保持一段时间以供复用
                // 这里可以添加定时器逻辑
            }

            // 通知状态变化
            if old_active != new_active {
                self.event_bus.emit(ConnectionEvent::StateChanged {
                    connection_id: connection_id.clone(),
                    old_state: old_active,
                    new_state: new_active,
                });
            }

            self.event_bus.emit(ConnectionEvent::ComponentUnbound {
                connection_id: connection_id.clone(),
                component_id: component_id.clone(),
            });
        }

        Ok(())
    }

    /// 获取连接的活跃组件数
    pub fn get_attached_component_count(&self, connection_id: &ConnectionId) -> usize {
        let registry = self.connection_registry.lock().unwrap();
        registry.get(connection_id)
            .map(|ctx| ctx.attached_components.len())
            .unwrap_or(0)
    }

    /// 健康检查所有连接
    pub fn health_check_all(&self) -> HashMap<ConnectionId, bool> {
        let registry = self.connection_registry.lock().unwrap();
        let mut results = HashMap::new();

        for (id, context) in registry.iter() {
            if let Some(pool) = &context.pool {
                let is_healthy = self.pool_manager.check_health(pool);
                results.insert(id.clone(), is_healthy);
            } else {
                results.insert(id.clone(), false);
            }
        }

        results
    }

    /// 获取事件总线
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }
}

// TODO: 需要实现其余的类型和实现
pub use super::{
    ConnectionId, ComponentId, BindingType, ConnectionEvent,
    ConnectionPoolManager, PoolConfig, ConnectionContext
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_load_real_config() {
        // 测试加载真实的配置文件
        let store = ConnectionConfigStore::new();

        if store.config_path.exists() {
            println!("Testing load from: {}", store.config_path.display());

            match store.load_all() {
                Ok(Some(connections)) => {
                    println!("✓ Successfully loaded {} connections", connections.len());
                    for (id, config) in connections {
                        println!("  - {}: {}", id.as_str(), config.name);
                    }
                }
                Ok(None) => {
                    println!("✗ No connections found");
                }
                Err(e) => {
                    println!("✗ Error loading: {}", e);
                }
            }
        } else {
            println!("Config file does not exist");
        }
    }

    #[test]
    fn test_create_connection() {
        let manager = GlobalConnectionManager::new();

        let config = DatabaseConnection {
            name: "Test Connection".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "test".to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            ssl_mode: crate::database::SslMode::Disable,
            connection_timeout: 30,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_connected: None,
            is_active: false,
        };

        let result = manager.create_connection(config.clone());
        assert!(result.is_ok());

        let connection_id = result.unwrap();
        let context = manager.get_context(&connection_id);
        assert!(context.is_some());

        // 清理
        let _ = manager.delete_connection(&connection_id);
    }
}
