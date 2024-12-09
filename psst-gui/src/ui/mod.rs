pub mod playlist;
pub mod preferences;
pub mod theme;
pub mod utils;

use druid::{
    widget::{Scroll, SizedBox},
    Widget, WidgetExt, WindowDesc,
};

use crate::{
    data::{AppState, Config},
    widget::theme::ThemeScope,
};

pub fn main_window(config: &Config) -> WindowDesc<AppState> {
    let win = WindowDesc::new(root_widget())
        .with_min_size((theme::grid(65.0), theme::grid(50.0)))
        .window_size(config.window_size)
        .show_title(false)
        .transparent_titlebar(true);
    if cfg!(target_os = "macos") {
        todo!()
    } else {
        win
    }
}

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

fn root_widget() -> impl Widget<AppState> {
    let playlists = Scroll::new(playlist::list_widget())
        .vertical()
        .expand_height();
    playlists
}
