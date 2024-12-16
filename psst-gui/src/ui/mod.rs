pub mod playlist;
pub mod preferences;
pub mod search;
pub mod theme;
pub mod utils;

use std::time::Duration;

use druid::{
    widget::{Flex, Label, Scroll, SizedBox, Slider},
    Cursor, LensExt, Selector, Widget, WidgetExt, WindowDesc,
};

use crate::{
    cmd,
    controller::SessionController,
    data::{AppState, Config, Nav, Playback},
    widget::{theme::ThemeScope, MyWidgetExt},
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

    let playlists = Flex::column()
        .must_fill_main_axis(true)
        .with_child(sidebar_menu_widget())
        .with_default_spacer()
        .with_flex_child(playlists, 1.0);

    let controls = Flex::column()
        .with_default_spacer()
        .with_child(volume_slider());

    ThemeScope::new(playlists).controller(SessionController)
}

fn sidebar_menu_widget() -> impl Widget<AppState> {
    Flex::column()
        .with_default_spacer()
        .with_child(sidebar_link_widget("Home", Nav::Home))
        .with_child(sidebar_link_widget("Tracks", Nav::SavedTracks))
        .with_child(sidebar_link_widget("Albums", Nav::SavedAlbums))
        .with_child(sidebar_link_widget("Podcasts", Nav::SavedShows))
        .with_child(search::input_widget().padding((theme::grid(1.0), theme::grid(1.0))))
}

fn sidebar_link_widget(title: &str, link_nav: Nav) -> impl Widget<AppState> {
    Label::new(title)
        .padding((theme::grid(2.0), theme::grid(1.0)))
        .expand_width()
        .link()
        .env_scope({
            let link_nav = link_nav.clone();
            move |env, nav: &Nav| {
                env.set(
                    theme::LINK_COLD_COLOR,
                    if &link_nav == nav {
                        env.get(theme::MENU_BUTTON_BG_ACTIVE)
                    } else {
                        env.get(theme::MENU_BUTTON_BG_INACTIVE)
                    },
                );
                env.set(
                    theme::TEXT_COLOR,
                    if &link_nav == nav {
                        env.get(theme::MENU_BUTTON_FG_ACTIVE)
                    } else {
                        env.get(theme::MENU_BUTTON_FG_INACTIVE)
                    },
                )
            }
        })
        .on_left_click(move |ctx, _, _, _| {
            ctx.submit_command(cmd::NAVIGATE.with(link_nav.clone()));
        })
        .lens(AppState::nav)
}

fn volume_slider() -> impl Widget<AppState> {
    const SAVE_DELAY: Duration = Duration::from_millis(100);
    const SAVE_TO_CONFIG: Selector = Selector::new("app.volume.save-to-config");

    Flex::row()
        .with_flex_child(
            Slider::new()
                .with_range(0.0, 1.0)
                .expand_width()
                .env_scope(|env, _| {
                    env.set(theme::BASIC_WIDGET_HEIGHT, theme::grid(1.5));
                    env.set(theme::FOREGROUND_LIGHT, env.get(theme::GREY_400));
                    env.set(theme::FOREGROUND_DARK, env.get(theme::GREY_400));
                })
                .with_cursor(Cursor::Pointer),
            1.0,
        )
        .with_default_spacer()
        .with_child(
            Label::dynamic(|&volume: &f64, _| format!("{}%", (volume * 100.0).floor()))
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_text_size(theme::TEXT_SIZE_SMALL),
        )
        .padding((theme::grid(2.0), 0.0))
        .on_debounce(SAVE_DELAY, |ctx, _, _| ctx.submit_command(SAVE_TO_CONFIG))
        .lens(AppState::playback.then(Playback::volume))
        .on_scroll(
            |data| &data.config.slider_scroll_scale,
            |_, data, _, scaled_delta| {
                data.playback.volume = (data.playback.volume + scaled_delta).clamp(0.0, 1.0);
            },
        )
}
