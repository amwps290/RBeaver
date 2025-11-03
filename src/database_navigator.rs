use gpui::{
    App, Context, Entity, EventEmitter, ParentElement, Render, Styled,Task, Window, div, prelude::*, px,
    rgb,
};
use gpui_component::{
    IconName, StyledExt,
    button::{Button, ButtonVariants},
    label::Label,
};

use crate::connection::{BindingType, ComponentId, ConnectionId, GlobalConnectionManager};
use crate::database::{DatabaseConnection, DatabaseManager};
use crate::database_structure::{DatabaseObjectType, DatabaseStructureQuery, DatabaseTreeNode};
use crate::lazy_loader::LazyLoadService;
use crate::lazy_tree::LazyTreeNode;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum DatabaseNavigatorEvent {
    ConnectionSelected(ConnectionId),
    ConnectionAdded(DatabaseConnection),
    ConnectionUpdated(DatabaseConnection),
    ConnectionDeleted(ConnectionId),
    ConnectionConnected(ConnectionId),
    ConnectionDisconnected(ConnectionId),
    NewConnectionRequested,
    ObjectSelected(String, DatabaseObjectType), // object_id, object_type
    StructureExpanded(ConnectionId, String),    // connection_id, node_id
}

pub struct DatabaseNavigator {
    global_manager: Arc<GlobalConnectionManager>,
    component_id: ComponentId,
    selected_connection_id: Option<ConnectionId>,
    expanded_nodes: HashMap<String, bool>,
    loading_connections: HashMap<ConnectionId, bool>,
    // 缓存的连接列表
    connections: Vec<(ConnectionId, DatabaseConnection)>,
    // 数据库对象树
    database_tree: Vec<LazyTreeNode>,
    // 懒加载服务
    lazy_loader: Arc<LazyLoadService>,
}

impl EventEmitter<DatabaseNavigatorEvent> for DatabaseNavigator {}

impl DatabaseNavigator {
    pub fn new(cx: &mut App) -> Entity<Self> {
        let global_manager = GlobalConnectionManager::get();
        let component_id = ComponentId::new();
        let lazy_loader = Arc::new(LazyLoadService::new());

        cx.new(|_| Self {
            global_manager,
            component_id,
            selected_connection_id: None,
            expanded_nodes: HashMap::new(),
            loading_connections: HashMap::new(),
            connections: Vec::new(),
            database_tree: Vec::new(),
            lazy_loader,
        })
    }

    /// 初始化时加载已保存的连接
    pub fn load_saved_connections(&mut self, cx: &mut Context<'_, Self>) {
        let global_manager = self.global_manager.clone();

        eprintln!("[DatabaseNavigator] Starting to load saved connections on startup");

        cx.spawn(async move |this, cx| {
            match global_manager.load_connections() {
                Ok(connection_ids) => {
                    eprintln!(
                        "[DatabaseNavigator] load_connections returned {} IDs",
                        connection_ids.len()
                    );

                    let mut connections = Vec::new();
                    for id in connection_ids {
                        if let Some(context) = global_manager.get_context(&id) {
                            eprintln!(
                                "[DatabaseNavigator] Found connection: {} ({})",
                                context.config.name,
                                id.as_str()
                            );
                            connections.push((id, context.config));
                        }
                    }

                    eprintln!(
                        "[DatabaseNavigator] Total connections loaded: {}",
                        connections.len()
                    );

                    // 更新UI
                    this.update(cx, |this, _cx| {
                        this.connections = connections;
                        eprintln!("[DatabaseNavigator] UI updated with loaded connections");
                    });
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[DatabaseNavigator] Failed to load connections: {}", e);
                    Err(e)
                }
            }
        }).detach();
    }

    /// 添加新连接
    pub fn add_connection(
        &mut self,
        connection: DatabaseConnection,
        cx: &mut Context<'_, Self>,
    ) -> Result<ConnectionId, Box<dyn std::error::Error>> {
        let connection_id = self
            .global_manager
            .create_connection(connection.clone())?;
        self.connections
            .push((connection_id.clone(), connection.clone()));
        cx.emit(DatabaseNavigatorEvent::ConnectionAdded(connection));
        cx.notify();
        Ok(connection_id)
    }

    /// 刷新连接列表（同步版本，用于UI更新）
    pub fn refresh_connections_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[DatabaseNavigator] Refreshing connection list (sync)");

        // 获取连接列表（现在是同步的）
        let connection_ids = self.global_manager.get_all_connections();

        eprintln!(
            "[DatabaseNavigator] Found {} connection IDs",
            connection_ids.len()
        );

        let mut connections = Vec::new();

        // 获取每个连接的上下文（现在是同步的）
        for id in connection_ids {
            if let Some(context) = self.global_manager.get_context(&id)
            {
                eprintln!(
                    "[DatabaseNavigator] Adding connection to list: {} ({})",
                    context.config.name,
                    id.as_str()
                );
                connections.push((id, context.config));
            }
        }

        self.connections = connections;
        eprintln!(
            "[DatabaseNavigator] Connection list refreshed. Total: {}",
            self.connections.len()
        );
        Ok(())
    }

    /// 刷新连接列表（异步版本）
    pub fn refresh_connections(
        &mut self,
        cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[DatabaseNavigator] Refreshing connection list");
        let connection_ids = self.global_manager.get_all_connections();
        eprintln!(
            "[DatabaseNavigator] Found {} connection IDs",
            connection_ids.len()
        );

        let mut connections = Vec::new();

        for id in connection_ids {
            if let Some(context) = self.global_manager.get_context(&id) {
                eprintln!(
                    "[DatabaseNavigator] Adding connection to list: {} ({})",
                    context.config.name,
                    id.as_str()
                );
                connections.push((id, context.config));
            }
        }

        self.connections = connections;
        eprintln!(
            "[DatabaseNavigator] Connection list refreshed. Total: {}",
            self.connections.len()
        );
        cx.notify();
        Ok(())
    }

    /// 删除连接
    pub fn delete_connection(
        &mut self,
        connection_id: &ConnectionId,
        cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.global_manager.delete_connection(connection_id)?;

        // 从缓存中移除
        self.connections.retain(|(id, _)| id != connection_id);

        // 如果删除的是当前选中的连接，清除选中状态
        if self.selected_connection_id.as_ref() == Some(connection_id) {
            self.selected_connection_id = None;
        }

        cx.emit(DatabaseNavigatorEvent::ConnectionDeleted(
            connection_id.clone(),
        ));
        cx.notify();
        Ok(())
    }

    /// 连接到数据库
    pub fn connect_to_database(
        &mut self,
        connection_id: ConnectionId,
        cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 绑定组件到连接（独占模式）
        self.global_manager
            .bind_component(
                self.component_id.clone(),
                connection_id.clone(),
                BindingType::Exclusive,
            )?;

        // 设置加载状态
        self.loading_connections.insert(connection_id.clone(), true);
        cx.notify();

        // 获取连接池来验证连接（同步操作）
        match self.global_manager.get_pool(&connection_id) {
            Ok(pool) => {
                // 测试连接（使用同步客户端）
                match pool.get() {
                    Ok(mut client) => {
                        let result = client.query("SELECT 1", &[]);

                        if result.is_ok() {
                            self.loading_connections.remove(&connection_id);
                            self.selected_connection_id = Some(connection_id.clone());

                            cx.emit(DatabaseNavigatorEvent::ConnectionConnected(connection_id));
                            cx.notify();
                        } else {
                            self.loading_connections.remove(&connection_id);
                            cx.emit(DatabaseNavigatorEvent::ConnectionDisconnected(
                                connection_id.clone(),
                            ));
                            cx.notify();
                        }
                    }
                    Err(e) => {
                        self.loading_connections.remove(&connection_id);
                        eprintln!("Failed to get client from pool: {}", e);
                        cx.emit(DatabaseNavigatorEvent::ConnectionDisconnected(
                            connection_id.clone(),
                        ));
                        cx.notify();
                    }
                }
            }
            Err(e) => {
                self.loading_connections.remove(&connection_id);
                eprintln!("Failed to connect: {}", e);
                cx.emit(DatabaseNavigatorEvent::ConnectionDisconnected(
                    connection_id.clone(),
                ));
                cx.notify();
            }
        }

        Ok(())
    }

    /// 从数据库断开连接
    pub fn disconnect_from_database(
        &mut self,
        connection_id: &ConnectionId,
        cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.global_manager
            .unbind_component(&self.component_id, connection_id)
            ?;

        if self.selected_connection_id.as_ref() == Some(connection_id) {
            self.selected_connection_id = None;
        }

        cx.emit(DatabaseNavigatorEvent::ConnectionDisconnected(
            connection_id.clone(),
        ));
        cx.notify();
        Ok(())
    }

    /// 切换数据库连接
    pub fn switch_connection(
        &mut self,
        new_connection_id: ConnectionId,
        cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 如果有当前选中的连接，先断开
        if let Some(current_id) = self.selected_connection_id.clone() {
            if current_id != new_connection_id {
                self.disconnect_from_database(&current_id, cx)?;
            }
        }

        // 连接到新连接
        self.connect_to_database(new_connection_id.clone(), cx)
            ?;

        // 加载数据库对象结构
        self.load_database_structure(new_connection_id.clone(), cx)
            ?;

        Ok(())
    }

    /// 加载数据库对象结构
    pub fn load_database_structure(
        &mut self,
        connection_id: ConnectionId,
        _cx: &mut Context<'_, Self>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取连接池
        let pool = match self.global_manager.get_pool(&connection_id) {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Failed to get pool: {}", e);
                return Err(e.into());
            }
        };

        // 直接加载 schema 列表（同步操作）
        let mut client = pool.get()?;
        match DatabaseStructureQuery::get_schemas(&mut client) {
            Ok(schemas) => {
                eprintln!("Loaded {} schemas", schemas.len());
                // TODO: 实际更新 UI
            }
            Err(e) => {
                eprintln!("Failed to load schemas: {}", e);
            }
        }

        // 临时设置一个占位符
        self.database_tree = vec![
            LazyTreeNode::new(
                format!("{}:schema:public", connection_id.as_str()),
                "public".to_string(),
                DatabaseObjectType::Schema,
            ),
            LazyTreeNode::new(
                format!("{}:schema:information_schema", connection_id.as_str()),
                "information_schema".to_string(),
                DatabaseObjectType::Schema,
            ),
        ];

        Ok(())
    }

    /// 处理节点展开（简化版本）
    pub fn handle_node_toggle(&mut self, node_id: String, _cx: &mut Context<'_, Self>) {
        // 查找节点
        let node_index = self.database_tree.iter().position(|n| n.id == node_id);
        if let Some(index) = node_index {
            let node = &mut self.database_tree[index];

            if node.is_loading {
                return; // 正在加载，忽略
            }

            // 切换展开状态
            node.is_expanded = !node.is_expanded;

            // 如果展开且未加载，添加示例子节点
            if node.is_expanded && node.children.is_empty() {
                let (conn_id, obj_type, schema, _) = node.parse_id();
                if matches!(obj_type, DatabaseObjectType::Schema) {
                    if let Some(schema_name) = schema {
                        // 添加示例对象类型
                        let object_types = vec![
                            DatabaseObjectType::Table,
                            DatabaseObjectType::View,
                            DatabaseObjectType::Function,
                        ];

                        for obj_type in object_types {
                            let child_id =
                                format!("{}:{}:{}", conn_id, obj_type.as_str(), schema_name);
                            let child_node = LazyTreeNode::new(
                                child_id,
                                obj_type.display_name().to_string(),
                                obj_type,
                            );
                            node.children.push(child_node);
                        }
                    }
                }
            }
        }
    }

    /// 查找并修改节点（递归版本，暂时未使用）
    fn find_node_mut<'a>(
        _nodes: &'a mut [LazyTreeNode],
        _target_id: &str,
    ) -> Option<&'a mut LazyTreeNode> {
        // TODO: 实现递归查找
        None
    }

    /// 获取组件ID
    pub fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }

    /// 渲染单个树节点（简化版本）
    fn render_tree_node(
        &self,
        _node: &LazyTreeNode,
        _depth: usize,
        _cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        div().child("Tree node (UI not fully implemented)")
    }

    // Simplified render method - connection items are now rendered inline in the main render method
    // This avoids lifetime issues with cx

    // Database structure rendering will be implemented in future updates
    fn render_database_structure(
        &self,
        _node: &DatabaseTreeNode,
        _depth: usize,
    ) -> impl IntoElement {
        div().child("Database structure coming soon...")
    }

    fn handle_new_connection(&mut self, cx: &mut Context<Self>) {
        cx.emit(DatabaseNavigatorEvent::NewConnectionRequested);
    }
}

impl Render for DatabaseNavigator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let connections = &self.connections;

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0xffffff))
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .px_3()
                    .bg(rgb(0xf8f9fa))
                    .border_b_1()
                    .border_color(rgb(0xced4da))
                    .child(
                        Label::new("Database Navigator")
                            .text_sm()
                            .font_semibold()
                            .text_color(rgb(0x495057)),
                    )
                    .child(
                        Button::new("new_connection")
                            .icon(IconName::Plus)
                            .ghost()
                            .tooltip("New Connection")
                            .on_click(cx.listener(|this, _event, _view, cx| {
                                this.handle_new_connection(cx);
                            })),
                    ),
            )
            .child(
                // Connection list
                div()
                    .flex_1()
                    .overflow_hidden()
                    .when(connections.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .h_full()
                                .p_4()
                                .child(
                                    div()
                                        .mb_2()
                                        .child(IconName::SquareTerminal)
                                        .text_color(rgb(0xced4da)),
                                )
                                .child(
                                    Label::new("No connections")
                                        .text_sm()
                                        .text_color(rgb(0x6c757d))
                                        .text_center()
                                        .mb_2(),
                                )
                                .child(
                                    Label::new("Click + to create your first connection")
                                        .text_xs()
                                        .text_color(rgb(0x9e9e9e))
                                        .text_center(),
                                ),
                        )
                    })
                    .when(!connections.is_empty(), |this| {
                        this.child(
                            div().flex().flex_col().p_2().gap_1().children(
                                connections
                                    .iter()
                                    .map(|(connection_id, connection)| {
                                        let is_active = self
                                            .selected_connection_id
                                            .as_ref()
                                            .map(|id| id == connection_id)
                                            .unwrap_or(false);
                                        let is_loading = self
                                            .loading_connections
                                            .get(connection_id)
                                            .copied()
                                            .unwrap_or(false);

                                        div()
                                            .flex()
                                            .items_center()
                                            .w_full()
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .hover(|style| style.bg(rgb(0xf8f9fa)))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .flex_1()
                                                    .child(
                                                        // Connection status indicator
                                                        div()
                                                            .w(px(8.0))
                                                            .h(px(8.0))
                                                            .rounded_full()
                                                            .bg(if is_loading {
                                                                rgb(0xffc107) // Yellow for loading
                                                            } else if is_active {
                                                                rgb(0x4caf50) // Green for connected
                                                            } else {
                                                                rgb(0x9e9e9e) // Gray for disconnected
                                                            }),
                                                    )
                                                    .child(
                                                        // Database icon
                                                        div().child(IconName::SquareTerminal),
                                                    )
                                                    .child(
                                                        // Connection details
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .flex_1()
                                                            .child(
                                                                Label::new(if is_loading {
                                                                    format!(
                                                                        "{} (Connecting...)",
                                                                        connection.name
                                                                    )
                                                                } else {
                                                                    connection.name.clone()
                                                                })
                                                                .text_sm()
                                                                .font_medium()
                                                                .text_color(if is_active {
                                                                    rgb(0x212529)
                                                                } else {
                                                                    rgb(0x6c757d)
                                                                }),
                                                            )
                                                            .child(
                                                                Label::new(format!(
                                                                    "{}:{}",
                                                                    connection.host,
                                                                    connection.port
                                                                ))
                                                                .text_xs()
                                                                .text_color(rgb(0x9e9e9e)),
                                                            ),
                                                    ),
                                            )
                                            .child(
                                                // Toggle connection button with proper event handling
                                                Button::new("connection_toggle")
                                                    .icon(if is_loading {
                                                        IconName::Globe
                                                    } else if is_active {
                                                        IconName::CircleX
                                                    } else {
                                                        IconName::Globe
                                                    })
                                                    .ghost()
                                                    .tooltip(if is_loading {
                                                        "Connecting..."
                                                    } else if is_active {
                                                        "Disconnect"
                                                    } else {
                                                        "Connect"
                                                    })
                                                    .when(!is_loading, |button| {
                                                        button.on_click(cx.listener(
                                                            move |_this, _event, _view, _cx| {
                                                                // 直接调用操作，不使用异步
                                                                // 注意：这里只是示例，实际应该用更合适的方法
                                                            },
                                                        ))
                                                    }),
                                            )
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                        )
                    }),
            )
            .child(
                // Footer with connection count
                div()
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .px_3()
                    .bg(rgb(0xf8f9fa))
                    .border_t_1()
                    .border_color(rgb(0xced4da))
                    .child(
                        Label::new(format!(
                            "{} connection{}",
                            connections.len(),
                            if connections.len() == 1 { "" } else { "s" }
                        ))
                        .text_xs()
                        .text_color(rgb(0x6c757d)),
                    ),
            )
    }
}
