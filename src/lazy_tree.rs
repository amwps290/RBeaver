//! 懒加载数据库树形结构

use crate::database_structure::DatabaseObjectType;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 懒加载树节点
#[derive(Debug, Clone)]
pub struct LazyTreeNode {
    /// 节点唯一标识符
    pub id: String,
    /// 节点显示名称
    pub name: String,
    /// 节点类型
    pub node_type: DatabaseObjectType,
    /// 是否已展开
    pub is_expanded: bool,
    /// 是否正在加载
    pub is_loading: bool,
    /// 是否已加载过（用于缓存检查）
    pub is_loaded: bool,
    /// 是否有更多数据（分页用）
    pub has_more: bool,
    /// 子节点
    pub children: Vec<LazyTreeNode>,
    /// 额外元数据（如分页信息）
    pub metadata: HashMap<String, String>,
    /// 缓存时间戳
    pub cache_timestamp: Option<Instant>,
    /// 错误信息（如果加载失败）
    pub error: Option<String>,
}

impl LazyTreeNode {
    /// 创建新的懒加载节点
    pub fn new(id: String, name: String, node_type: DatabaseObjectType) -> Self {
        Self {
            id,
            name,
            node_type,
            is_expanded: false,
            is_loading: false,
            is_loaded: false,
            has_more: false,
            children: Vec::new(),
            metadata: HashMap::new(),
            cache_timestamp: None,
            error: None,
        }
    }

    /// 创建 Schema 节点
    pub fn new_schema(connection_id: &str, schema_name: String) -> Self {
        let id = format!("{}:schema:{}", connection_id, schema_name);
        Self::new(id, schema_name, DatabaseObjectType::Schema)
    }

    /// 创建对象类型节点（如 Tables, Functions）
    pub fn new_object_type(
        connection_id: &str,
        schema_name: String,
        object_type: DatabaseObjectType,
    ) -> Self {
        let id = format!(
            "{}:{}:{}",
            connection_id,
            object_type.as_str(),
            schema_name
        );
        let display_name = object_type.display_name().to_string();
        Self::new(id, display_name, object_type)
    }

    /// 创建具体对象节点（如特定表、函数）
    pub fn new_object(
        connection_id: &str,
        schema_name: String,
        object_type: DatabaseObjectType,
        object_name: String,
    ) -> Self {
        let id = format!(
            "{}:{}:{}:{}",
            connection_id,
            object_type.as_str(),
            schema_name,
            object_name
        );
        Self::new(id, object_name, object_type)
    }

    /// 检查缓存是否有效（默认30分钟）
    pub fn is_cache_valid(&self) -> bool {
        if let Some(timestamp) = self.cache_timestamp {
            timestamp.elapsed() < Duration::from_secs(1800)
        } else {
            false
        }
    }

    /// 更新缓存时间戳
    pub fn update_cache_timestamp(&mut self) {
        self.cache_timestamp = Some(Instant::now());
    }

    /// 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if !loading && !self.is_loaded {
            self.is_loaded = true;
        }
    }

    /// 设置错误信息
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_loading = false;
    }

    /// 清除错误
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// 检查是否有子节点或可以加载子节点
    pub fn can_have_children(&self) -> bool {
        matches!(
            self.node_type,
            DatabaseObjectType::Schema
                | DatabaseObjectType::Table
                | DatabaseObjectType::Extension
        )
    }

    /// 获取节点图标名称
    pub fn get_icon_name(&self) -> &'static str {
        match self.node_type {
            DatabaseObjectType::Schema => "Folder",
            DatabaseObjectType::Extension => "Package",
            DatabaseObjectType::Table => "Table",
            DatabaseObjectType::View => "Eye",
            DatabaseObjectType::Index => "Hash",
            DatabaseObjectType::Function => "Function",
            DatabaseObjectType::Procedure => "Tool",
            DatabaseObjectType::Sequence => "ListOrdered",
            DatabaseObjectType::Trigger => "Zap",
            DatabaseObjectType::Type => "Type",
        }
    }

    /// 从 ID 中提取信息
    pub fn parse_id(&self) -> (String, DatabaseObjectType, Option<String>, Option<String>) {
        let parts: Vec<&str> = self.id.split(':').collect();

        if parts.len() >= 2 {
            let connection_id = parts[0].to_string();
            let object_type = DatabaseObjectType::from_str(parts[1]);

            if parts.len() == 3 {
                // Schema 节点
                (connection_id, object_type, Some(parts[2].to_string()), None)
            } else if parts.len() == 4 {
                // 具体对象节点
                (
                    connection_id,
                    object_type,
                    Some(parts[2].to_string()),
                    Some(parts[3].to_string()),
                )
            } else {
                (
                    connection_id,
                    object_type,
                    None,
                    None,
                )
            }
        } else {
            (
                "unknown".to_string(),
                DatabaseObjectType::Table,
                None,
                None,
            )
        }
    }
}

impl PartialEq for LazyTreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for LazyTreeNode {}

impl PartialOrd for LazyTreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LazyTreeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

/// 懒加载事件
#[derive(Debug, Clone)]
pub enum LazyLoadEvent {
    /// 开始加载
    LoadStarted {
        node_id: String,
        parent_id: String,
    },
    /// 加载完成
    LoadCompleted {
        node_id: String,
        parent_id: String,
        children_count: usize,
    },
    /// 加载失败
    LoadFailed {
        node_id: String,
        parent_id: String,
        error: String,
    },
    /// 缓存被清除
    CacheCleared {
        pattern: String,
    },
}

impl LazyLoadEvent {
    pub fn node_id(&self) -> &str {
        match self {
            LazyLoadEvent::LoadStarted { node_id, .. }
            | LazyLoadEvent::LoadCompleted { node_id, .. }
            | LazyLoadEvent::LoadFailed { node_id, .. } => node_id,
            LazyLoadEvent::CacheCleared { .. } => "",
        }
    }

    pub fn parent_id(&self) -> &str {
        match self {
            LazyLoadEvent::LoadStarted { parent_id, .. }
            | LazyLoadEvent::LoadCompleted { parent_id, .. }
            | LazyLoadEvent::LoadFailed { parent_id, .. } => parent_id,
            LazyLoadEvent::CacheCleared { .. } => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_schema_node() {
        let node = LazyTreeNode::new_schema("conn1", "public".to_string());
        assert_eq!(node.id, "conn1:schema:public");
        assert_eq!(node.name, "public");
        assert_eq!(node.node_type, DatabaseObjectType::Schema);
    }

    #[test]
    fn test_parse_schema_node_id() {
        let node = LazyTreeNode::new_schema("conn1", "public".to_string());
        let (conn_id, obj_type, schema, object) = node.parse_id();
        assert_eq!(conn_id, "conn1");
        assert_eq!(schema, Some("public".to_string()));
        assert_eq!(object, None);
    }

    #[test]
    fn test_cache_validity() {
        let mut node = LazyTreeNode::new(
            "id".to_string(),
            "test".to_string(),
            DatabaseObjectType::Table,
        );
        assert!(!node.is_cache_valid());

        node.update_cache_timestamp();
        assert!(node.is_cache_valid());
    }

    #[test]
    fn test_can_have_children() {
        let schema_node = LazyTreeNode::new_schema("conn", "public".to_string());
        assert!(schema_node.can_have_children());

        let object_type_node = LazyTreeNode::new(
            "id".to_string(),
            "Tables".to_string(),
            DatabaseObjectType::Table,
        );
        assert!(object_type_node.can_have_children());
    }
}
