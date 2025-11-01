use gpui::{
    App, Context, Entity, FocusHandle, MouseDownEvent, MouseMoveEvent, MouseUpEvent, SharedString,
    Subscription, Window, div, prelude::*, px, rgb, rgba,
};
use gpui_component::{self, menu::AppMenuBar, StyledExt, TitleBar};

use crate::actions::ToggleDatabaseNavigator;
use crate::connection_dialog::ConnectionDialogEvent;
use crate::database_navigator::DatabaseNavigatorEvent;
use crate::statusbar::StatusBarEvent;
use crate::{ConnectionDialog, DatabaseConnection, DatabaseNavigator, StatusBar, ToolBar};

pub struct MainWindow {
    title: SharedString,
    app_menu_bar: Entity<AppMenuBar>,
    toolbar: Entity<ToolBar>,
    statusbar: Entity<StatusBar>,
    database_navigator: Entity<DatabaseNavigator>,
    connection_dialog: Option<Entity<ConnectionDialog>>,
    pending_connection_dialog: Option<DatabaseConnection>,
    database_navigator_visible: bool,
    database_navigator_width: f32,
    is_resizing_navigator: bool,
    resize_start_x: f32,
    resize_start_width: f32,
    _navigator_subscription: Option<Subscription>,
    _statusbar_subscription: Option<Subscription>,
    focus_handle: FocusHandle,
}

impl MainWindow {
    pub fn new(title: SharedString, window: &mut Window, cx: &mut App) -> Self {
        let app_menu_bar = AppMenuBar::new(window, cx);
        let toolbar = cx.new(|_| ToolBar::new());
        let statusbar = cx.new(|_| StatusBar::new().with_database_navigator_visible(true));
        let database_navigator = DatabaseNavigator::new(cx);
        let focus_handle = cx.focus_handle();

        Self {
            title,
            app_menu_bar,
            toolbar,
            statusbar,
            database_navigator,
            connection_dialog: None,
            pending_connection_dialog: None,
            database_navigator_visible: true,
            database_navigator_width: 280.0,
            is_resizing_navigator: false,
            resize_start_x: 0.0,
            resize_start_width: 0.0,
            _navigator_subscription: None,
            _statusbar_subscription: None,
            focus_handle,
        }
    }

    fn _toggle_database_navigator(&mut self, cx: &mut Context<Self>) {
        self.database_navigator_visible = !self.database_navigator_visible;
        // 更新状态栏的显示状态
        self.statusbar.update(cx, |statusbar, cx| {
            statusbar.set_database_navigator_visible(self.database_navigator_visible);
            cx.notify();
        });
        cx.notify();
    }

    pub fn toggle_database_navigator(
        &mut self,
        _: &ToggleDatabaseNavigator,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self._toggle_database_navigator(cx);
    }

    pub fn is_database_navigator_visible(&self) -> bool {
        self.database_navigator_visible
    }

    pub fn set_database_navigator_width(&mut self, width: f32, cx: &mut Context<Self>) {
        self.database_navigator_width = width.max(200.0).min(500.0); // 限制宽度范围
        cx.notify();
    }

    pub fn get_database_navigator_width(&self) -> f32 {
        self.database_navigator_width
    }

    fn start_resize(&mut self, mouse_x: f32, cx: &mut Context<Self>) {
        self.is_resizing_navigator = true;
        self.resize_start_x = mouse_x;
        self.resize_start_width = self.database_navigator_width;
        cx.notify();
    }

    fn update_resize(&mut self, mouse_x: f32, cx: &mut Context<Self>) {
        if self.is_resizing_navigator {
            let delta = mouse_x - self.resize_start_x;
            let new_width = (self.resize_start_width + delta).max(200.0).min(600.0);
            self.database_navigator_width = new_width;
            cx.notify();
        }
    }

    fn stop_resize(&mut self, cx: &mut Context<Self>) {
        self.is_resizing_navigator = false;
        cx.notify();
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
                self.pending_connection_dialog = Some(DatabaseConnection::default());
                cx.notify();
            }
            _ => {}
        }
    }

    fn handle_statusbar_event(
        &mut self,
        _entity: Entity<StatusBar>,
        event: &StatusBarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            StatusBarEvent::ToggleDatabaseNavigator => {
                println!("Receive ToggleDatabaseNavigator");
                self._toggle_database_navigator(cx);
            }
        }
    }

    fn show_connection_dialog(
        &mut self,
        connection: Option<DatabaseConnection>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dialog = ConnectionDialog::new(connection, window, cx);

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 检查是否需要创建连接对话框
        if let Some(connection) = self.pending_connection_dialog.take() {
            self.show_connection_dialog(Some(connection), window, cx);
        }

        // Subscribe to events only once
        if self._navigator_subscription.is_none() {
            self._navigator_subscription =
                Some(cx.subscribe(&self.database_navigator, Self::handle_navigator_event));
            self._statusbar_subscription =
                Some(cx.subscribe(&self.statusbar, Self::handle_statusbar_event));
        }

        div()
            .on_action(cx.listener(Self::toggle_database_navigator))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _view, cx| {
                if this.is_resizing_navigator {
                    this.update_resize(event.position.x.into(), cx);
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _view, cx| {
                    if this.is_resizing_navigator {
                        this.stop_resize(cx);
                    }
                }),
            )
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
            // 菜单栏分割线
            .child(div().w_full().h(px(1.0)).bg(rgb(0xced4da)))
            // 菜单栏容器，限制高度防止占据全部空间
            .child(
                div()
                    .h(px(32.0))
                    .w_full()
                    .child(self.app_menu_bar.clone())
            )
            // 工具栏分割线
            .child(div().w_full().h(px(1.0)).bg(rgb(0xced4da)))
            .child(self.toolbar.clone())
            // 工具栏分割线
            .child(div().w_full().h(px(1.0)).bg(rgb(0xced4da)))
            .child(
                // 主内容区域，占据剩余空间
                div()
                    .track_focus(&self.focus_handle)
                    .flex_1()
                    .flex()
                    .flex_row()
                    .bg(rgb(0xf8f9fa))
                    .min_h_0()
                    .when(self.database_navigator_visible, |this| {
                        this.child(
                            // 左侧数据库导航栏容器
                            div()
                                .flex()
                                .child(
                                    // 数据库导航栏
                                    div()
                                        .w(px(self.database_navigator_width))
                                        .flex_shrink_0()
                                        .border_r_1()
                                        .border_color(rgb(0xced4da))
                                        .child(self.database_navigator.clone()),
                                )
                                .child(
                                    // 可拖拽的分隔条
                                    div()
                                        .w(px(2.0))
                                        .h_full()
                                        .bg(if self.is_resizing_navigator {
                                            rgb(0x0066cc)
                                        } else {
                                            rgb(0xced4da)
                                        })
                                        .hover(|style| style.bg(rgb(0x0066cc)).cursor_col_resize())
                                        .cursor_col_resize()
                                        .flex_shrink_0()
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(
                                                |this, event: &MouseDownEvent, _view, cx| {
                                                    this.start_resize(event.position.x.into(), cx);
                                                },
                                            ),
                                        )
                                        .on_mouse_move(cx.listener(
                                            |this, event: &MouseMoveEvent, _view, cx| {
                                                if this.is_resizing_navigator {
                                                    this.update_resize(event.position.x.into(), cx);
                                                }
                                            },
                                        ))
                                        .on_mouse_up(
                                            gpui::MouseButton::Left,
                                            cx.listener(
                                                |this, _event: &MouseUpEvent, _view, cx| {
                                                    this.stop_resize(cx);
                                                },
                                            ),
                                        ),
                                ),
                        )
                    })
                    .child(
                        // 主工作区
                        div()
                            .flex_1()
                            .min_w_0()
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
            // Global connection dialog overlay
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
            })
    }
}
