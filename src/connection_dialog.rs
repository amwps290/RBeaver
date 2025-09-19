use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, ParentElement, Render, Styled, Window, div,
    prelude::*, px, rgb,
};
use gpui_component::{
    IconName, StyledExt,
    button::{Button, ButtonVariants},
    input::*,
    label::Label,
};
use sqlx::types::Text;

use crate::database::{ConnectionTestResult, DatabaseConnection};

pub struct ConnectionDialog {
    connection: DatabaseConnection,
    focus_handle: FocusHandle,
    is_testing: bool,
    test_result: Option<ConnectionTestResult>,
    validation_errors: Vec<String>,

    connection_name: Entity<InputState>,
    connection_host: Entity<InputState>,
    connection_port: Entity<InputState>,
    connection_database: Entity<InputState>,
    connection_username: Entity<InputState>,
    connection_password: Entity<InputState>,
    connection_timeout: Entity<InputState>,
}

#[derive(Clone, Debug)]
pub enum ConnectionDialogEvent {
    Save(DatabaseConnection),
    Cancel,
}

impl EventEmitter<ConnectionDialogEvent> for ConnectionDialog {}

impl ConnectionDialog {
    pub fn new(
        connection: Option<DatabaseConnection>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let connection = connection.unwrap_or_default();
        let connection_name =
            cx.new(|cx| InputState::new(window, cx).placeholder(connection.name.clone()));
        let connection_host =
            cx.new(|cx| InputState::new(window, cx).placeholder(connection.host.clone()));
        let connection_port =
            cx.new(|cx| InputState::new(window, cx).placeholder(connection.port.to_string()));
        let connection_database =
            cx.new(|cx| InputState::new(window, cx).placeholder(connection.database.clone()));
        let connection_username =
            cx.new(|cx| InputState::new(window, cx).placeholder(connection.username.clone()));
        let connection_password = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder(connection.password.clone())
        });
        let connection_timeout = cx.new(|cx| {
            InputState::new(window, cx).placeholder(connection.connection_timeout.to_string())
        });

        cx.new(|cx| Self {
            connection,
            focus_handle: cx.focus_handle(),
            is_testing: false,
            test_result: None,
            validation_errors: Vec::new(),
            connection_name,
            connection_host,
            connection_port,
            connection_database,
            connection_username,
            connection_password,
            connection_timeout,
        })
    }

    fn collect_form_data(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Result<DatabaseConnection, Vec<String>> {
        let mut connection = self.connection.clone();
        let mut errors = Vec::new();

        connection.name = self.connection_name.read(cx).value().into();
        connection.host = self.connection_host.read(cx).value().into();
        let port: String = self.connection_port.read(cx).value().into();
        connection.database = self.connection_database.read(cx).value().into();
        connection.username = self.connection_username.read(cx).value().into();
        connection.password = self.connection_password.read(cx).value().into();
        let timeout: String = self.connection_timeout.read(cx).value().into();

        // Parse port
        match port.parse::<u16>() {
            Ok(port) => connection.port = port,
            Err(_) => errors.push("Invalid port number".to_string()),
        }

        // Parse timeout
        match timeout.parse::<u32>() {
            Ok(timeout) => connection.connection_timeout = timeout,
            Err(_) => errors.push("Invalid timeout value".to_string()),
        }

        // Validate
        if let Err(validation_error) = connection.validate() {
            errors.push(validation_error);
        }

        if errors.is_empty() {
            Ok(connection)
        } else {
            Err(errors)
        }
    }

    fn handle_save(&mut self, cx: &mut Context<Self>) {
        match self.collect_form_data(cx) {
            Ok(connection) => {
                self.validation_errors.clear();
                cx.emit(ConnectionDialogEvent::Save(connection));
            }
            Err(errors) => {
                self.validation_errors = errors;
                cx.notify();
            }
        }
    }

    fn handle_test_connection(&mut self, cx: &mut Context<Self>) {
        // 如果正在测试中，则不执行新的测试
        if self.is_testing {
            return;
        }

        // 首先收集表单数据并验证
        match self.collect_form_data(cx) {
            Ok(connection) => {
                // 设置测试状态
                self.is_testing = true;
                self.test_result = None;
                self.validation_errors.clear();
                cx.notify();

                // 为了简化，我们暂时使用同步方式进行测试
                // 在实际应用中，应该使用异步方式避免阻塞UI
                let result = {
                    // 创建 tokio runtime 并执行测试
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async { connection.test_connection().await })
                };

                // 更新测试结果
                self.is_testing = false;
                self.test_result = Some(result);
                cx.notify();
            }
            Err(errors) => {
                // 如果表单数据无效，显示验证错误
                self.validation_errors = errors;
                self.test_result = None;
                cx.notify();
            }
        }
    }

    fn handle_cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(ConnectionDialogEvent::Cancel);
    }
}

impl Render for ConnectionDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .border_1()
            .border_color(rgb(0xd1d5db))
            .rounded_lg()
            .shadow_lg()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .p_6()
            .gap_4()
            .bg(rgb(0xffffff))
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb_4()
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        Label::new("PostgreSQL Connection")
                            .text_lg()
                            .font_semibold()
                            .text_color(rgb(0x111827)),
                    )
                    .child(Button::new("close").icon(IconName::Close).ghost().on_click(
                        cx.listener(|this, _event, _view, cx| {
                            this.handle_cancel(cx);
                        }),
                    )),
            )
            .child(
                // Form content
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Label::new("Connection Name")
                                    .text_sm()
                                    .font_medium()
                                    .text_color(rgb(0x374151)),
                            )
                            .child(TextInput::new(&self.connection_name)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Label::new("Host")
                                            .text_sm()
                                            .font_medium()
                                            .text_color(rgb(0x374151)),
                                    )
                                    .child(TextInput::new(&self.connection_host)),
                            )
                            .child(
                                div()
                                    .w(px(120.0))
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Label::new("Port")
                                            .text_sm()
                                            .font_medium()
                                            .text_color(rgb(0x374151)),
                                    )
                                    .child(TextInput::new(&self.connection_port)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Label::new("Database")
                                    .text_sm()
                                    .font_medium()
                                    .text_color(rgb(0x374151)),
                            )
                            .child(TextInput::new(&self.connection_database)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Label::new("Username")
                                            .text_sm()
                                            .font_medium()
                                            .text_color(rgb(0x374151)),
                                    )
                                    .child(TextInput::new(&self.connection_username)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Label::new("Password")
                                            .text_sm()
                                            .font_medium()
                                            .text_color(rgb(0x374151)),
                                    )
                                    .child(TextInput::new(&self.connection_password)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                Label::new("Connection Timeout (seconds)")
                                    .text_sm()
                                    .font_medium()
                                    .text_color(rgb(0x374151)),
                            )
                            .child(TextInput::new(&self.connection_timeout)),
                    )
                    // Test Result
                    .when_some(self.test_result.as_ref(), |this, result| match result {
                        ConnectionTestResult::Success => this.child(
                            div()
                                .p_3()
                                .bg(rgb(0xdcfce7))
                                .border_l_4()
                                .border_color(rgb(0x22c55e))
                                .rounded_md()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(div().child(IconName::Check))
                                        .child(
                                            Label::new("Connection successful!")
                                                .text_color(rgb(0x166534))
                                                .text_sm()
                                                .font_medium(),
                                        ),
                                ),
                        ),
                        ConnectionTestResult::Failed(error) => this.child(
                            div()
                                .p_3()
                                .bg(rgb(0xfee2e2))
                                .border_l_4()
                                .border_color(rgb(0xef4444))
                                .rounded_md()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(div().child(IconName::CircleX))
                                        .child(
                                            Label::new(format!("Connection failed: {}", error))
                                                .text_color(rgb(0x7f1d1d))
                                                .text_sm()
                                                .font_medium(),
                                        ),
                                ),
                        ),
                    })
                    // Validation Errors
                    .when(!self.validation_errors.is_empty(), |this| {
                        this.child(
                            div()
                                .p_3()
                                .bg(rgb(0xfee2e2))
                                .border_l_4()
                                .border_color(rgb(0xef4444))
                                .rounded_md()
                                .child(
                                    div().flex().flex_col().gap_1().children(
                                        self.validation_errors
                                            .iter()
                                            .map(|error| {
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(div().child(IconName::CircleX))
                                                    .child(
                                                        Label::new(error)
                                                            .text_color(rgb(0x7f1d1d))
                                                            .text_sm(),
                                                    )
                                            })
                                            .collect::<Vec<_>>(),
                                    ),
                                ),
                        )
                    }),
            )
            .child(
                // Footer
                div()
                    .flex()
                    .justify_between()
                    .pt_4()
                    .border_t_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        Button::new("test_connection")
                            .label(if self.is_testing {
                                "Testing..."
                            } else {
                                "Test Connection"
                            })
                            .icon(IconName::Globe)
                            .outline()
                            .on_click(cx.listener(|this, _event, _view, cx| {
                                this.handle_test_connection(cx);
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(Button::new("cancel").label("Cancel").outline().on_click(
                                cx.listener(|this, _event, _view, cx| {
                                    this.handle_cancel(cx);
                                }),
                            ))
                            .child(Button::new("save").label("Save").primary().on_click(
                                cx.listener(|this, _event, _view, cx| {
                                    this.handle_save(cx);
                                }),
                            )),
                    ),
            )
    }
}
