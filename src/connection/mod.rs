//! 连接管理模块
//!
//! 该模块提供多数据库连接管理功能，包括：
//! - 全局连接管理器
//! - 连接池管理
//! - 连接生命周期管理
//! - 组件与连接的绑定机制

pub mod manager;
pub mod pool_manager;
pub mod binding;

pub use manager::{GlobalConnectionManager, ConnectionManager};
pub use pool_manager::{ConnectionPoolManager, PoolConfig};
pub use binding::{ConnectionId, ComponentId, ConnectionContext, BindingType, ConnectionEvent};
