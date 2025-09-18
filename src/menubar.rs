use crate::actions::*;
use gpui::{ParentElement, Render, Styled, div, px};
use gpui_component::{button::Button, popup_menu::PopupMenuExt};

pub struct MenuBar {}

impl Render for MenuBar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().flex().flex_col().w_full().child(
            // 菜单栏
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .h(px(32.0))
                .bg(gpui::rgb(0xf8f9fa))
                .border_b_1()
                .border_color(gpui::rgb(0xced4da))
                // File Menu
                .child(
                    Button::new("file_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("File")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("New Connection", Box::new(DatabaseNewConnection))
                                .separator()
                                .menu("New", Box::new(FileNew))
                                .menu("Open", Box::new(FileOpen))
                                .menu("Recent", Box::new(FileRecent))
                                .separator()
                                .menu("Import", Box::new(FileImport))
                                .menu("Export", Box::new(FileExport))
                                .separator()
                                .menu("Exit", Box::new(FileExit))
                        }),
                )
                // Edit Menu
                .child(
                    Button::new("edit_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("Edit")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("Undo", Box::new(EditUndo))
                                .menu("Redo", Box::new(EditRedo))
                                .separator()
                                .menu("Cut", Box::new(EditCut))
                                .menu("Copy", Box::new(EditCopy))
                                .menu("Paste", Box::new(EditPaste))
                                .separator()
                                .menu("Find", Box::new(EditFind))
                                .menu("Replace", Box::new(EditReplace))
                        }),
                )
                // View Menu
                .child(
                    Button::new("view_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("View")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("Database Navigator", Box::new(ViewDatabaseNavigator))
                                .menu("Project Explorer", Box::new(ViewProjectExplorer))
                                .menu("Properties", Box::new(ViewProperties))
                                .separator()
                                .menu("SQL Editor", Box::new(ViewSqlEditor))
                                .menu("Data Editor", Box::new(ViewDataEditor))
                                .separator()
                                .menu("Toolbar", Box::new(ViewToolbar))
                                .menu("Status Bar", Box::new(ViewStatusBar))
                        }),
                )
                // Navigate Menu
                .child(
                    Button::new("navigate_menu")
                        .w(px(80.0))
                        .h(px(28.0))
                        .label("Navigate")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("Go to Line", Box::new(NavigateGoToLine))
                                .menu("Go to Object", Box::new(NavigateGoToObject))
                                .separator()
                                .menu("Back", Box::new(NavigateBack))
                                .menu("Forward", Box::new(NavigateForward))
                                .separator()
                                .menu("Bookmarks", Box::new(NavigateBookmarks))
                        }),
                )
                // SQL Menu
                .child(
                    Button::new("sql_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("SQL")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("Execute", Box::new(SqlExecute))
                                .menu("Execute Current", Box::new(SqlExecuteCurrent))
                                .menu("Execute Script", Box::new(SqlExecuteScript))
                                .separator()
                                .menu("Format", Box::new(SqlFormat))
                                .menu("Validate", Box::new(SqlValidate))
                                .separator()
                                .menu("Show Execution Plan", Box::new(SqlExecutionPlan))
                        }),
                )
                // Tools Menu
                .child(
                    Button::new("tools_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("Tools")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("Database Compare", Box::new(ToolsDatabaseCompare))
                                .menu("Data Transfer", Box::new(ToolsDataTransfer))
                                .menu("Schema Compare", Box::new(ToolsSchemaCompare))
                                .separator()
                                .menu("Backup/Restore", Box::new(ToolsBackupRestore))
                                .menu("Generate SQL", Box::new(ToolsGenerateSql))
                                .separator()
                                .menu("Preferences", Box::new(ToolsPreferences))
                        }),
                )
                // Window Menu
                .child(
                    Button::new("window_menu")
                        .w(px(70.0))
                        .h(px(28.0))
                        .label("Window")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("New Window", Box::new(WindowNewWindow))
                                .menu("Close Window", Box::new(WindowCloseWindow))
                                .separator()
                                .menu("Reset Layout", Box::new(WindowResetLayout))
                                .menu("Save Layout", Box::new(WindowSaveLayout))
                        }),
                )
                // Help Menu
                .child(
                    Button::new("help_menu")
                        .w(px(60.0))
                        .h(px(28.0))
                        .label("Help")
                        .popup_menu(|this, _window, _cx| {
                            this.menu("User Guide", Box::new(HelpUserGuide))
                                .menu("Shortcuts", Box::new(HelpShortcuts))
                                .separator()
                                .menu("Check for Updates", Box::new(HelpCheckUpdates))
                                .separator()
                                .menu("About RBeaver", Box::new(HelpAbout))
                        }),
                ),
        )
    }
}
