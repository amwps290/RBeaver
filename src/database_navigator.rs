use gpui::{
    App, Context, Entity, EventEmitter, ParentElement, Render, Styled, Window, div, prelude::*, px,
    rgb,
};
use gpui_component::{
    IconName, StyledExt,
    button::{Button, ButtonVariants},
    label::Label,
};

use crate::database::{ConnectionManager, DatabaseConnection};

#[derive(Clone, Debug)]
pub enum DatabaseNavigatorEvent {
    ConnectionSelected(String),
    ConnectionAdded(DatabaseConnection),
    ConnectionUpdated(DatabaseConnection),
    ConnectionDeleted(String),
    ConnectionConnected(String),
    ConnectionDisconnected(String),
    NewConnectionRequested,
}

pub struct DatabaseNavigator {
    connection_manager: ConnectionManager,
    selected_connection_id: Option<String>,
}

impl EventEmitter<DatabaseNavigatorEvent> for DatabaseNavigator {}

impl DatabaseNavigator {
    pub fn new(cx: &mut App) -> Entity<Self> {
        let connection_manager =
            ConnectionManager::load_from_file(&ConnectionManager::get_config_path())
                .unwrap_or_else(|_| ConnectionManager::new());

        cx.new(|_| Self {
            connection_manager,
            selected_connection_id: None,
        })
    }

    pub fn add_connection(&mut self, connection: DatabaseConnection, cx: &mut Context<Self>) {
        self.connection_manager.add_connection(connection.clone());
        self.save_connections();
        cx.emit(DatabaseNavigatorEvent::ConnectionAdded(connection));
        cx.notify();
    }

    pub fn refresh_connections(&mut self, cx: &mut Context<Self>) {
        if let Ok(manager) =
            ConnectionManager::load_from_file(&ConnectionManager::get_config_path())
        {
            self.connection_manager = manager;
            cx.notify();
        }
    }

    fn save_connections(&self) {
        if let Err(e) = self
            .connection_manager
            .save_to_file(&ConnectionManager::get_config_path())
        {
            eprintln!("Failed to save connections: {}", e);
        }
    }

    fn delete_connection(&mut self, connection_id: String, cx: &mut Context<Self>) {
        self.connection_manager.remove_connection(&connection_id);
        self.save_connections();

        if self.selected_connection_id.as_ref() == Some(&connection_id) {
            self.selected_connection_id = None;
        }

        cx.emit(DatabaseNavigatorEvent::ConnectionDeleted(connection_id));
        cx.notify();
    }

    fn connect_to_database(&mut self, connection_id: String, cx: &mut Context<Self>) {
        if let Some(connection) = self.connection_manager.connections.get_mut(&connection_id) {
            connection.set_active(true);
            self.save_connections();
            cx.emit(DatabaseNavigatorEvent::ConnectionConnected(connection_id));
            cx.notify();
        }
    }

    fn disconnect_from_database(&mut self, connection_id: String, cx: &mut Context<Self>) {
        if let Some(connection) = self.connection_manager.connections.get_mut(&connection_id) {
            connection.set_active(false);
            self.save_connections();
            cx.emit(DatabaseNavigatorEvent::ConnectionDisconnected(
                connection_id,
            ));
            cx.notify();
        }
    }

    fn render_connection_item(&self, connection: &DatabaseConnection) -> impl IntoElement {
        let connection_id = connection.id.clone();
        let is_selected = self.selected_connection_id.as_ref() == Some(&connection.id);
        let is_active = connection.is_active;

        div()
            .flex()
            .items_center()
            .w_full()
            .px_2()
            .py_1()
            .rounded_md()
            .hover(|style| style.bg(rgb(0xf8f9fa)))
            .when(is_selected, |style| style.bg(rgb(0xe3f2fd)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .child(
                        // Connection status indicator
                        div().w(px(8.0)).h(px(8.0)).rounded_full().bg(if is_active {
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
                                Label::new(connection.name.clone())
                                    .text_sm()
                                    .font_medium()
                                    .text_color(if is_active {
                                        rgb(0x212529)
                                    } else {
                                        rgb(0x6c757d)
                                    }),
                            )
                            .child(
                                Label::new(format!("{}:{}", connection.host, connection.port))
                                    .text_xs()
                                    .text_color(rgb(0x9e9e9e)),
                            ),
                    ),
            )
            .child(
                // Toggle connection button
                Button::new("connection_toggle")
                    .icon(if is_active {
                        IconName::CircleX
                    } else {
                        IconName::Globe
                    })
                    .ghost()
                    .tooltip(if is_active { "Disconnect" } else { "Connect" }),
            )
    }

    fn handle_new_connection(&mut self, cx: &mut Context<Self>) {
        cx.emit(DatabaseNavigatorEvent::NewConnectionRequested);
    }
}

impl Render for DatabaseNavigator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let connections: Vec<_> = self.connection_manager.get_all_connections();

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
                                    .into_iter()
                                    .map(|connection| self.render_connection_item(connection))
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
                            self.connection_manager.connections.len(),
                            if self.connection_manager.connections.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ))
                        .text_xs()
                        .text_color(rgb(0x6c757d)),
                    ),
            )
    }
}
