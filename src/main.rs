use gpui::{
    App, Application, Bounds, KeyBinding, WindowBounds, WindowKind, WindowOptions, prelude::*, px,
    size,
};
use gpui_component::{self, Root, TitleBar};
use rbeaver::{Assets, MainWindow, actions::ToggleDatabaseNavigator, init_actions};

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        init_actions(cx);

        cx.bind_keys(vec![KeyBinding::new(
            "ctrl-b",
            ToggleDatabaseNavigator,
            None,
        )]);
        let mut window_size = size(px(1600.0), px(1200.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }

        let window_bounds = Bounds::centered(None, window_size, cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(480.),
                height: px(320.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| MainWindow::new("RBeaver".into(), window, cx));
            cx.new(|cx| Root::new(view.into(), window, cx))
        })
        .unwrap();
    });
}
