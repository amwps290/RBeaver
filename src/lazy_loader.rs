//! 懒加载服务

use crate::connection::ConnectionId;
use crate::database_structure::{DatabaseObjectType, DatabaseStructureQuery};
use crate::lazy_tree::{LazyTreeNode, LazyLoadEvent};
use postgres::Client;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 懒加载缓存
#[derive(Debug, Clone)]
pub struct LazyLoadCache {
    /// 按节点ID缓存子节点
    cache: Arc<Mutex<HashMap<String, (Vec<LazyTreeNode>, Instant)>>>,
    /// 缓存过期时间（秒）
    ttl: Duration,
}

impl LazyLoadCache {
    /// 创建新的缓存
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// 获取缓存的节点
    pub fn get(&self, parent_id: &str) -> Option<Vec<LazyTreeNode>> {
        let cache = self.cache.lock().unwrap();
        if let Some((nodes, timestamp)) = cache.get(parent_id) {
            if timestamp.elapsed() < self.ttl {
                return Some(nodes.clone());
            }
        }
        None
    }

    /// 存储节点到缓存
    pub fn set(&self, parent_id: String, nodes: Vec<LazyTreeNode>) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(parent_id, (nodes, Instant::now()));
    }

    /// 清除特定模式的缓存
    pub fn invalidate(&self, pattern: &str) {
        let mut cache = self.cache.lock().unwrap();
        let keys: Vec<String> = cache
            .keys()
            .filter(|k| k.starts_with(pattern))
            .cloned()
            .collect();

        for key in keys {
            cache.remove(&key);
        }
    }

    /// 清除所有缓存
    pub fn clear_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> (usize, Vec<String>) {
        let cache = self.cache.lock().unwrap();
        let count = cache.len();
        let keys: Vec<String> = cache.keys().cloned().collect();
        (count, keys)
    }
}

/// 懒加载服务
#[derive(Debug)]
pub struct LazyLoadService {
    /// 缓存管理器
    cache: LazyLoadCache,
    /// 正在加载的节点集合
    loading_queue: Arc<Mutex<HashSet<String>>>,
    /// 分页大小
    page_size: usize,
}

impl LazyLoadService {
    /// 创建新的懒加载服务
    pub fn new() -> Self {
        Self {
            cache: LazyLoadCache::new(1800),
            loading_queue: Arc::new(Mutex::new(HashSet::new())),
            page_size: 100,
        }
    }

    /// 检查节点是否正在加载
    pub fn is_loading(&self, node_id: &str) -> bool {
        let queue = self.loading_queue.lock().unwrap();
        queue.contains(node_id)
    }

    /// 标记节点为正在加载
    pub fn start_loading(&self, node_id: String) {
        let mut queue = self.loading_queue.lock().unwrap();
        queue.insert(node_id);
    }

    /// 标记节点加载完成
    pub fn finish_loading(&self, node_id: &str) {
        let mut queue = self.loading_queue.lock().unwrap();
        queue.remove(node_id);
    }

    /// 懒加载数据库对象
    pub fn load_objects(
        &self,
        client: &mut Client,
        parent_id: &str,
        schema: Option<&str>,
        object_type: DatabaseObjectType,
    ) -> Result<Vec<LazyTreeNode>, Box<dyn std::error::Error>> {
        let cache_key = format!("{}:{}:{:?}", parent_id, schema.unwrap_or(""), object_type);

        // 检查缓存
        if let Some(nodes) = self.cache.get(&cache_key) {
            return Ok(nodes);
        }

        // 执行加载
        let nodes = match object_type {
            DatabaseObjectType::Table => {
                let tables = DatabaseStructureQuery::get_tables(client, schema)?;
                tables
                    .into_iter()
                    .map(|table| {
                        LazyTreeNode::new(
                            format!("{}:table:{}:{}", parent_id, schema.unwrap_or(""), table.name),
                            table.name,
                            DatabaseObjectType::Table,
                        )
                    })
                    .collect()
            }
            DatabaseObjectType::Function => {
                let functions = DatabaseStructureQuery::get_functions(client, schema)?;
                functions
                    .into_iter()
                    .map(|function| {
                        LazyTreeNode::new(
                            format!(
                                "{}:function:{}:{}",
                                parent_id,
                                schema.unwrap_or(""),
                                function.name
                            ),
                            format!("{} ({})", function.name, function.return_type),
                            DatabaseObjectType::Function,
                        )
                    })
                    .collect()
            }
            DatabaseObjectType::Index => {
                let indexes = DatabaseStructureQuery::get_indexes(client, schema)?;
                indexes
                    .into_iter()
                    .map(|index| {
                        LazyTreeNode::new(
                            format!(
                                "{}:index:{}:{}",
                                parent_id,
                                schema.unwrap_or(""),
                                index.index_name
                            ),
                            format!("{} ({})", index.index_name, index.table_name),
                            DatabaseObjectType::Index,
                        )
                    })
                    .collect()
            }
            DatabaseObjectType::Type => {
                let types = DatabaseStructureQuery::get_types(client, schema)?;
                types
                    .into_iter()
                    .map(|type_info| {
                        LazyTreeNode::new(
                            format!("{}:type:{}:{}", parent_id, schema.unwrap_or(""), type_info.name),
                            format!("{} ({})", type_info.name, type_info.type_category),
                            DatabaseObjectType::Type,
                        )
                    })
                    .collect()
            }
            _ => Vec::new(),
        };

        // 缓存结果
        self.cache.set(cache_key, nodes.clone());

        Ok(nodes)
    }

    /// 获取缓存大小
    pub fn get_cache_size(&self) -> usize {
        self.cache.get_stats().0
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        self.cache.clear_all();
    }
}

impl Default for LazyLoadService {
    fn default() -> Self {
        Self::new()
    }
}
