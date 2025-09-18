use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, ParentElement, Render, Styled, Window, div,
    prelude::*, px, rgb,
};
use gpui_component::{
    IconName, StyledExt,
    button::{Button, ButtonVariants},
    label::Label,
};

use crate::database::{ConnectionTestResult, DatabaseConnection};

pub struct ConnectionDialog {
    connection: DatabaseConnection,
    focus_handle: FocusHandle,
    is_testing: bool,
    test_result: Option<ConnectionTestResult>,
    validation_errors: Vec<String>,

    // Form values
    name_value: String,
    host_value: String,
    port_value: String,
    database_value: String,
    username_value: String,
    password_value: String,
    timeout_value: String,
}

#[derive(Clone, Debug)]
pub enum ConnectionDialogEvent {
    Save(DatabaseConnection),
    Cancel,
}

impl EventEmitter<ConnectionDialogEvent> for ConnectionDialog {}

impl ConnectionDialog {
    pub fn new(connection: Option<DatabaseConnection>, cx: &mut App) -> Entity<Self> {
        let connection = connection.unwrap_or_default();

        cx.new(|cx| Self {
            name_value: connection.name.clone(),
            host_value: connection.host.clone(),
            port_value: connection.port.to_string(),
            database_value: connection.database.clone(),
            username_value: connection.username.clone(),
            password_value: connection.password.clone(),
            timeout_value: connection.connection_timeout.to_string(),
            connection,
            focus_handle: cx.focus_handle(),
            is_testing: false,
            test_result: None,
            validation_errors: Vec::new(),
        })
    }

    fn collect_form_data(&mut self) -> Result<DatabaseConnection, Vec<String>> {
        let mut connection = self.connection.clone();
        let mut errors = Vec::new();

        connection.name = self.name_value.clone();
        connection.host = self.host_value.clone();
        connection.database = self.database_value.clone();
        connection.username = self.username_value.clone();
        connection.password = self.password_value.clone();

        // Parse port
        match self.port_value.parse::<u16>() {
            Ok(port) => connection.port = port,
            Err(_) => errors.push("Invalid port number".to_string()),
        }

        // Parse timeout
        match self.timeout_value.parse::<u32>() {
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
        match self.collect_form_data() {
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

    fn handle_test_connection(&mut self, cx: &mut Context<Self>) {}

    fn handle_cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(ConnectionDialogEvent::Cancel);
    }

    fn render_input(
        &self,
        label: &'static str,
        value: &str,
        placeholder: &'static str,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                Label::new(label)
                    .text_sm()
                    .font_medium()
                    .text_color(rgb(0x374151)),
            )
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .border_1()
                    .border_color(rgb(0xd1d5db))
                    .rounded_md()
                    .bg(rgb(0xffffff))
                    .child(if value.is_empty() {
                        div().text_color(rgb(0x9ca3af)).child(placeholder)
                    } else {
                        div().child(value.to_string())
                    }),
            )
    }
}

impl Render for ConnectionDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
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
                    .child(self.render_input("Connection Name", &self.name_value, "My PostgreSQL"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(div().flex_1().child(self.render_input(
                                "Host",
                                &self.host_value,
                                "localhost",
                            )))
                            .child(div().w(px(120.0)).child(self.render_input(
                                "Port",
                                &self.port_value,
                                "5432",
                            ))),
                    )
                    .child(self.render_input("Database", &self.database_value, "postgres"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(div().flex_1().child(self.render_input(
                                "Username",
                                &self.username_value,
                                "postgres",
                            )))
                            .child(div().flex_1().child(self.render_input(
                                "Password",
                                &self.password_value,
                                "Password",
                            ))),
                    )
                    .child(self.render_input("Timeout (seconds)", &self.timeout_value, "30"))
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
