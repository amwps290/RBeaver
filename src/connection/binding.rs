//! 连接绑定相关类型定义

use std::time::Instant;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;

/// 连接唯一标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub String);

impl ConnectionId {
    /// 创建新的连接 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// 从字符串创建连接 ID
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// 获取连接的字符串表示
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件唯一标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId(pub String);

impl ComponentId {
    /// 创建新的组件 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// 从字符串创建组件 ID
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// 获取组件 ID 的字符串表示
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接上下文信息
#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub id: ConnectionId,
    pub name: String,
    pub config: crate::database::DatabaseConnection,
    pub pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
    pub last_used: Instant,
    pub is_active: bool,
    pub attached_components: Vec<ComponentId>,
}

impl ConnectionContext {
    /// 创建新的连接上下文
    pub fn new(id: ConnectionId, config: crate::database::DatabaseConnection) -> Self {
        Self {
            id,
            name: config.name.clone(),
            config,
            pool: None,
            last_used: Instant::now(),
            is_active: false,
            attached_components: Vec::new(),
        }
    }

    /// 检查是否有组件正在使用此连接
    pub fn has_active_components(&self) -> bool {
        !self.attached_components.is_empty()
    }
}

/// 连接绑定类型
#[derive(Debug, Clone, PartialEq)]
pub enum BindingType {
    /// 独占连接 - 组件断开时关闭连接
    Exclusive,
    /// 共享连接 - 组件断开时仅解绑，连接保持活跃
    Shared,
    /// 会话连接 - 组件生命周期内保持连接
    Session,
}

/// 连接事件
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// 连接已创建
    Created(ConnectionId),
    /// 连接已删除
    Deleted(ConnectionId),
    /// 连接已断开
    Disconnected(ConnectionId),
    /// 连接已重新连接
    Reconnected(ConnectionId),
    /// 组件已绑定到连接
    ComponentBound {
        connection_id: ConnectionId,
        component_id: ComponentId,
        binding_type: BindingType,
    },
    /// 组件已从连接解绑
    ComponentUnbound {
        connection_id: ConnectionId,
        component_id: ComponentId,
    },
    /// 连接状态变化
    StateChanged {
        connection_id: ConnectionId,
        old_state: bool,
        new_state: bool,
    },
    /// 连接错误
    Error {
        connection_id: ConnectionId,
        error: String,
    },
}
