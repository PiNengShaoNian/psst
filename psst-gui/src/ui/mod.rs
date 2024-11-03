pub mod preferences;
pub mod theme;

use druid::{Widget, WidgetExt, WindowDesc};

use crate::{data::AppState, widget::theme::ThemeScope};

pub fn account_setup_window() -> WindowDesc<AppState> {
    let win = WindowDesc::new(account_setup_widget())
        .title("Login")
        .window_size((theme::grid(50.0), theme::grid(45.0)))
        .resizable(false)
        .show_title(false)
        .transparent_titlebar(true);

    if cfg!(target_os = "macos") {
        // win.menu(menu::main_menu)
        todo!()
    } else {
        win
    }
}

fn account_setup_widget() -> impl Widget<AppState> {
    ThemeScope::new(
        preferences::account_setup_widget()
            .background(theme::BACKGROUND_DARK)
            .expand(),
    )
}
