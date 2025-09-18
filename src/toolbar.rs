use gpui::{ParentElement, Render, Styled, div, px};
use gpui_component::{
    IconName,
    button::{Button, ButtonVariants},
};

pub struct ToolBar {}

impl ToolBar {
    pub fn new() -> Self {
        Self {}
    }
}

impl Render for ToolBar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(40.0))
            .bg(gpui::rgb(0xf8f9fa))
            .border_b_1()
            .border_color(gpui::rgb(0xced4da))
            .px_2()
            .gap_1()
            // 连接相关按钮
            .child(
                Button::new("new_connection")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Plus)
                    .tooltip("New Connection")
                    .primary(),
            )
            .child(
                Button::new("connect")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Globe)
                    .tooltip("Connect")
                    .outline(),
            )
            .child(
                Button::new("disconnect")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::CircleX)
                    .tooltip("Disconnect")
                    .outline(),
            )
            // 分隔符
            .child(div().w(px(1.0)).h(px(24.0)).bg(gpui::rgb(0xdee2e6)).mx_1())
            // SQL 相关按钮
            .child(
                Button::new("execute_sql")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::ArrowRight)
                    .tooltip("Execute SQL")
                    .outline(),
            )
            .child(
                Button::new("execute_current")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::ChevronRight)
                    .tooltip("Execute Current Statement")
                    .outline(),
            )
            .child(
                Button::new("stop_execution")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::CircleX)
                    .tooltip("Stop Execution")
                    .outline(),
            )
            // 分隔符
            .child(div().w(px(1.0)).h(px(24.0)).bg(gpui::rgb(0xdee2e6)).mx_1())
            // 文件操作按钮
            .child(
                Button::new("open_file")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::FolderOpen)
                    .tooltip("Open File")
                    .outline(),
            )
            .child(
                Button::new("save_file")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Check)
                    .tooltip("Save")
                    .outline(),
            )
            .child(
                Button::new("save_all")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Copy)
                    .tooltip("Save All")
                    .outline(),
            )
            // 分隔符
            .child(div().w(px(1.0)).h(px(24.0)).bg(gpui::rgb(0xdee2e6)).mx_1())
            // 编辑操作按钮
            .child(
                Button::new("undo")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::ArrowLeft)
                    .tooltip("Undo")
                    .outline(),
            )
            .child(
                Button::new("redo")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::ArrowRight)
                    .tooltip("Redo")
                    .outline(),
            )
            // 分隔符
            .child(div().w(px(1.0)).h(px(24.0)).bg(gpui::rgb(0xdee2e6)).mx_1())
            // 搜索和过滤
            .child(
                Button::new("search")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Search)
                    .tooltip("Find")
                    .outline(),
            )
            .child(
                Button::new("filter")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Settings)
                    .tooltip("Filter")
                    .outline(),
            )
            // 右侧推到最右边的按钮
            .child(div().flex_1()) // 占据剩余空间
            // 视图切换按钮
            .child(
                Button::new("toggle_navigator")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::PanelLeft)
                    .tooltip("Toggle Database Navigator")
                    .outline(),
            )
            .child(
                Button::new("toggle_properties")
                    .w(px(36.0))
                    .h(px(32.0))
                    .icon(IconName::Settings)
                    .tooltip("Toggle Properties")
                    .outline(),
            )
    }
}
