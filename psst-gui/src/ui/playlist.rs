use druid::{
    widget::{Label, LineBreaking, List},
    Insets, LensExt, Menu, Selector, Widget, WidgetExt,
};

use crate::{
    cmd,
    data::{AppState, Ctx, Library, Nav, Playlist, WithCtx},
    error::Error,
    webapi::WebApi,
    widget::{Async, MyWidgetExt},
};

use super::{theme, utils};

pub const LOAD_LIST: Selector = Selector::new("app.playlist.load-list");

pub fn list_widget() -> impl Widget<AppState> {
    Async::new(
        utils::spinner_widget,
        || {
            List::new(|| {
                Label::raw()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .with_text_size(theme::TEXT_SIZE_SMALL)
                    .lens(Ctx::data().then(Playlist::name))
                    .expand_width()
                    .padding(Insets::uniform_xy(theme::grid(2.0), theme::grid(0.6)))
                    .link()
                    .on_left_click(|ctx, _, playlist, _| {
                        ctx.submit_command(
                            cmd::NAVIGATE.with(Nav::PlaylistDetail(playlist.data.link())),
                        );
                    })
                    .context_menu(playlist_menu_ctx)
            })
        },
        utils::error_widget,
    )
    .lens(
        Ctx::make(
            AppState::common_ctx,
            AppState::library.then(Library::playlists.in_arc()),
        )
        .then(Ctx::in_promise()),
    )
    .on_command_async(
        LOAD_LIST,
        |_| WebApi::global().get_playlists(),
        |_, data, d| data.with_library_mut(|l| l.playlists.defer(d)),
        |_, data, r| data.with_library_mut(|l| l.playlists.update(r)),
    )
}

fn playlist_menu_ctx(playlist: &WithCtx<Playlist>) -> Menu<AppState> {
    // TODO
    Menu::empty()
}
