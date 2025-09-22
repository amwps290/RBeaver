use gpui::{EventEmitter, ParentElement, Render, Styled, div, prelude::FluentBuilder, px};
use gpui_component::{
    IconName,
    button::{Button, ButtonVariants},
    label::Label,
};

#[derive(Clone, Debug)]
pub enum StatusBarEvent {
    ToggleDatabaseNavigator,
}

pub struct StatusBar {
    query_status: String,
    row_count: Option<u64>,
    execution_time: Option<String>,
    database_navigator_visible: bool,
}

impl EventEmitter<StatusBarEvent> for StatusBar {}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            query_status: "Ready".to_string(),
            row_count: None,
            execution_time: None,
            database_navigator_visible: true,
        }
    }

    pub fn with_query_status(mut self, status: impl Into<String>) -> Self {
        self.query_status = status.into();
        self
    }

    pub fn with_row_count(mut self, count: u64) -> Self {
        self.row_count = Some(count);
        self
    }

    pub fn with_execution_time(mut self, time: impl Into<String>) -> Self {
        self.execution_time = Some(time.into());
        self
    }

    pub fn with_database_navigator_visible(mut self, visible: bool) -> Self {
        self.database_navigator_visible = visible;
        self
    }

    pub fn set_database_navigator_visible(&mut self, visible: bool) {
        self.database_navigator_visible = visible;
    }
}

impl Render for StatusBar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(24.0))
            .bg(gpui::rgb(0xf8f9fa))
            .border_t_1()
            .border_color(gpui::rgb(0xced4da))
            .px_2()
            .gap_2()
            // 左侧状态信息
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    // 数据库导航栏切换按钮
                    .child(
                        Button::new("toggle_database_navigator")
                            .border_10()
                            .w(px(24.0))
                            .h(px(20.0))
                            .icon(if self.database_navigator_visible {
                                IconName::PanelLeftClose
                            } else {
                                IconName::PanelLeftOpen
                            })
                            .link()
                            .tooltip(if self.database_navigator_visible {
                                "Hide Database Navigator"
                            } else {
                                "Show Database Navigator"
                            })
                            .on_click(cx.listener(|_this, _event, _view, cx| {
                                println!("Emit ToggleDatabaseNavigator");
                                cx.emit(StatusBarEvent::ToggleDatabaseNavigator);
                            })),
                    )
                    // 分隔符
                    .child(div().w(px(1.0)).h(px(16.0)).bg(gpui::rgb(0xced4da)))
                    // 查询状态
                    .child(Label::new(self.query_status.clone()).text_size(px(11.0)))
                    // 行数显示
                    .when_some(self.row_count.clone(), |this, count| {
                        this.child(div().w(px(1.0)).h(px(16.0)).bg(gpui::rgb(0xced4da)))
                            .child(Label::new(format!("{} rows", count)).text_size(px(11.0)))
                    })
                    // 执行时间
                    .when_some(self.execution_time.clone(), |this, time| {
                        this.child(div().w(px(1.0)).h(px(16.0)).bg(gpui::rgb(0xced4da)))
                            .child(Label::new(format!("Execution: {}", time)).text_size(px(11.0)))
                    }),
            )
            // 右侧功能按钮
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    // 数据库编码
                    .child(Label::new("UTF-8").text_size(px(11.0)))
                    // 分隔符
                    .child(div().w(px(1.0)).h(px(16.0)).bg(gpui::rgb(0xced4da)))
                    // 行列位置
                    .child(
                        Button::new("cursor_position")
                            .w(px(60.0))
                            .h(px(20.0))
                            .label("Ln 1, Col 1")
                            .link()
                            .text_size(px(11.0)),
                    )
                    // 缩放控制
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(
                                Button::new("zoom_out")
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .icon(IconName::Minus)
                                    .link(),
                            )
                            .child(
                                Label::new("100%")
                                    .text_size(px(11.0))
                                    .w(px(40.0))
                                    .text_center(),
                            )
                            .child(
                                Button::new("zoom_in")
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .icon(IconName::Plus)
                                    .link(),
                            ),
                    )
                    // 设置按钮
                    .child(
                        Button::new("statusbar_settings")
                            .w(px(24.0))
                            .h(px(20.0))
                            .icon(IconName::Settings)
                            .link()
                            .tooltip("Status Bar Settings"),
                    ),
            )
    }
}
