use gpui::{App, Context, Entity, SharedString, Window, div, prelude::*, px, rgb, rgba};
use gpui_component::{self, StyledExt, TitleBar};

use crate::connection_dialog::ConnectionDialogEvent;
use crate::database_navigator::DatabaseNavigatorEvent;
use crate::{ConnectionDialog, DatabaseConnection, DatabaseNavigator, MenuBar, StatusBar, ToolBar};

pub struct MainWindow {
    title: SharedString,
    top_menubar: Entity<MenuBar>,
    toolbar: Entity<ToolBar>,
    statusbar: Entity<StatusBar>,
    database_navigator: Entity<DatabaseNavigator>,
    connection_dialog: Option<Entity<ConnectionDialog>>,
}

impl MainWindow {
    pub fn new(title: SharedString, cx: &mut App) -> Self {
        let top_menubar = cx.new(|_| MenuBar {});
        let toolbar = cx.new(|_| ToolBar::new());
        let statusbar = cx.new(|_| StatusBar::new().with_connection_status("Connected"));
        let database_navigator = DatabaseNavigator::new(cx);

        Self {
            title,
            top_menubar,
            toolbar,
            statusbar,
            database_navigator,
            connection_dialog: None,
        }
    }
}

impl MainWindow {
    fn handle_navigator_event(
        &mut self,
        _entity: Entity<DatabaseNavigator>,
        event: &DatabaseNavigatorEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            DatabaseNavigatorEvent::NewConnectionRequested => {
                self.show_connection_dialog(None, cx);
            }
            _ => {}
        }
    }

    fn show_connection_dialog(
        &mut self,
        connection: Option<DatabaseConnection>,
        cx: &mut Context<Self>,
    ) {
        let dialog = ConnectionDialog::new(connection, cx);

        cx.subscribe(&dialog, |this, _dialog, event, cx| match event {
            ConnectionDialogEvent::Save(connection) => {
                this.database_navigator.update(cx, |nav, cx| {
                    nav.add_connection(connection.clone(), cx);
                });
                this.connection_dialog = None;
                cx.notify();
            }
            ConnectionDialogEvent::Cancel => {
                this.connection_dialog = None;
                cx.notify();
            }
        })
        .detach();

        self.connection_dialog = Some(dialog);
        cx.notify();
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Subscribe to navigator events
        cx.subscribe(&self.database_navigator, |this, entity, event, cx| {
            this.handle_navigator_event(entity, event, cx);
        })
        .detach();
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xf5f5f5))
            .child(
                TitleBar::new()
                    .child(div().flex().items_center().child(self.title.clone()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child(div().ml_auto().w(px(12.)).h_full()),
                    ),
            )
            .child(self.top_menubar.clone())
            // 菜单栏分割线
            .child(div().w_full().h(px(1.0)).bg(rgb(0xced4da)))
            .child(self.toolbar.clone())
            // 工具栏分割线
            .child(div().w_full().h(px(1.0)).bg(rgb(0xced4da)))
            .child(
                // 主内容区域，占据剩余空间
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .bg(rgb(0xf8f9fa))
                    .min_h_0()
                    .child(
                        // 左侧数据库导航栏
                        div()
                            .w(px(280.0))
                            .flex_shrink_0()
                            .border_r_1()
                            .border_color(rgb(0xced4da))
                            .child(self.database_navigator.clone())
                            .when_some(self.connection_dialog.clone(), |this, dialog| {
                                this.child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .bg(rgba(0x000000aa))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .bg(rgb(0xffffff))
                                                .rounded_lg()
                                                .shadow_lg()
                                                .w(px(600.0))
                                                .max_h(px(700.0))
                                                .child(dialog),
                                        ),
                                )
                            }),
                    )
                    .child(
                        // 主工作区
                        div()
                            .flex_1()
                            .bg(rgb(0xffffff))
                            .flex()
                            .flex_col()
                            .child(
                                // 工作区标题栏/标签页区域
                                div()
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .px_3()
                                    .bg(rgb(0xf8f9fa))
                                    .border_b_1()
                                    .border_color(rgb(0xced4da))
                                    .child(
                                        div().text_sm().text_color(rgb(0x6c757d)).child("Welcome"),
                                    ),
                            )
                            .child(
                                // 主工作内容
                                div()
                                    .flex_1()
                                    .p_6()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_semibold()
                                            .text_color(rgb(0x495057))
                                            .mb_4()
                                            .child("Welcome to RBeaver"),
                                    )
                                    .child(
                                        div()
                                            .text_color(rgb(0x6c757d))
                                            .text_center()
                                            .child("Your Database Management Tool"),
                                    )
                                    .child(
                                        div()
                                            .mt_6()
                                            .text_sm()
                                            .text_color(rgb(0x6c757d))
                                            .text_center()
                                            .child(
                                                "Create a new database connection to get started",
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // 右侧属性面板（可选）
                        div()
                            .w(px(250.0))
                            .flex_shrink_0()
                            .bg(rgb(0xffffff))
                            .border_l_1()
                            .border_color(rgb(0xced4da))
                            .flex()
                            .flex_col()
                            .child(
                                // 属性面板标题
                                div()
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .px_3()
                                    .bg(rgb(0xf8f9fa))
                                    .border_b_1()
                                    .border_color(rgb(0xced4da))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_semibold()
                                            .text_color(rgb(0x495057))
                                            .child("Properties"),
                                    ),
                            )
                            .child(
                                // 属性内容
                                div().flex_1().p_3().child(
                                    div()
                                        .text_color(rgb(0x6c757d))
                                        .text_sm()
                                        .child("No selection"),
                                ),
                            ),
                    ),
            )
            .child(self.statusbar.clone())
    }
}
