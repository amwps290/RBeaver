use gpui::{
    App, Application, Bounds, KeyBinding, Menu, WindowBounds, WindowKind, WindowOptions,
    prelude::*, px, size, MenuItem,
};
use gpui_component::{self, Root, TitleBar};
use rbeaver::{Assets, MainWindow, actions::*, init_actions};

fn configure_menus() -> Vec<Menu> {
    use rbeaver::actions::*;

    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::Action { name: "New Connection".into(), action: Box::new(DatabaseNewConnection), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "New".into(), action: Box::new(FileNew), os_action: None },
                MenuItem::Action { name: "Open".into(), action: Box::new(FileOpen), os_action: None },
                MenuItem::Action { name: "Recent".into(), action: Box::new(FileRecent), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Import".into(), action: Box::new(FileImport), os_action: None },
                MenuItem::Action { name: "Export".into(), action: Box::new(FileExport), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Exit".into(), action: Box::new(FileExit), os_action: None },
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::Action { name: "Undo".into(), action: Box::new(EditUndo), os_action: None },
                MenuItem::Action { name: "Redo".into(), action: Box::new(EditRedo), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Cut".into(), action: Box::new(EditCut), os_action: None },
                MenuItem::Action { name: "Copy".into(), action: Box::new(EditCopy), os_action: None },
                MenuItem::Action { name: "Paste".into(), action: Box::new(EditPaste), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Find".into(), action: Box::new(EditFind), os_action: None },
                MenuItem::Action { name: "Replace".into(), action: Box::new(EditReplace), os_action: None },
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::Action { name: "Database Navigator".into(), action: Box::new(ViewDatabaseNavigator), os_action: None },
                MenuItem::Action { name: "Project Explorer".into(), action: Box::new(ViewProjectExplorer), os_action: None },
                MenuItem::Action { name: "Properties".into(), action: Box::new(ViewProperties), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "SQL Editor".into(), action: Box::new(ViewSqlEditor), os_action: None },
                MenuItem::Action { name: "Data Editor".into(), action: Box::new(ViewDataEditor), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Toolbar".into(), action: Box::new(ViewToolbar), os_action: None },
                MenuItem::Action { name: "Status Bar".into(), action: Box::new(ViewStatusBar), os_action: None },
            ],
        },
        Menu {
            name: "Navigate".into(),
            items: vec![
                MenuItem::Action { name: "Go to Line".into(), action: Box::new(NavigateGoToLine), os_action: None },
                MenuItem::Action { name: "Go to Object".into(), action: Box::new(NavigateGoToObject), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Back".into(), action: Box::new(NavigateBack), os_action: None },
                MenuItem::Action { name: "Forward".into(), action: Box::new(NavigateForward), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Bookmarks".into(), action: Box::new(NavigateBookmarks), os_action: None },
            ],
        },
        Menu {
            name: "SQL".into(),
            items: vec![
                MenuItem::Action { name: "Execute".into(), action: Box::new(SqlExecute), os_action: None },
                MenuItem::Action { name: "Execute Current".into(), action: Box::new(SqlExecuteCurrent), os_action: None },
                MenuItem::Action { name: "Execute Script".into(), action: Box::new(SqlExecuteScript), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Format".into(), action: Box::new(SqlFormat), os_action: None },
                MenuItem::Action { name: "Validate".into(), action: Box::new(SqlValidate), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Show Execution Plan".into(), action: Box::new(SqlExecutionPlan), os_action: None },
            ],
        },
        Menu {
            name: "Tools".into(),
            items: vec![
                MenuItem::Action { name: "Database Compare".into(), action: Box::new(ToolsDatabaseCompare), os_action: None },
                MenuItem::Action { name: "Data Transfer".into(), action: Box::new(ToolsDataTransfer), os_action: None },
                MenuItem::Action { name: "Schema Compare".into(), action: Box::new(ToolsSchemaCompare), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Backup/Restore".into(), action: Box::new(ToolsBackupRestore), os_action: None },
                MenuItem::Action { name: "Generate SQL".into(), action: Box::new(ToolsGenerateSql), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Preferences".into(), action: Box::new(ToolsPreferences), os_action: None },
            ],
        },
        Menu {
            name: "Window".into(),
            items: vec![
                MenuItem::Action { name: "New Window".into(), action: Box::new(WindowNewWindow), os_action: None },
                MenuItem::Action { name: "Close Window".into(), action: Box::new(WindowCloseWindow), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Reset Layout".into(), action: Box::new(WindowResetLayout), os_action: None },
                MenuItem::Action { name: "Save Layout".into(), action: Box::new(WindowSaveLayout), os_action: None },
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![
                MenuItem::Action { name: "User Guide".into(), action: Box::new(HelpUserGuide), os_action: None },
                MenuItem::Action { name: "Shortcuts".into(), action: Box::new(HelpShortcuts), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "Check for Updates".into(), action: Box::new(HelpCheckUpdates), os_action: None },
                MenuItem::Separator,
                MenuItem::Action { name: "About RBeaver".into(), action: Box::new(HelpAbout), os_action: None },
            ],
        },
    ]
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        init_actions(cx);

        cx.bind_keys(vec![KeyBinding::new(
            "ctrl-b",
            ToggleDatabaseNavigator,
            None,
        )]);

        // Set up menus
        cx.set_menus(configure_menus());
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
