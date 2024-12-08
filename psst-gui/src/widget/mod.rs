pub mod icons;
mod link;
mod promise;
pub mod theme;

use std::sync::Arc;

use druid::{
    widget::ControllerHost, Cursor, Data, Env, EventCtx, Menu, MouseButton, MouseEvent, Selector,
    Widget,
};
pub use link::Link;
pub use promise::Async;

use crate::{
    controller::{ExClick, ExCursor, OnCommandAsync},
    data::AppState,
};

pub trait MyWidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn link(self) -> Link<T> {
        Link::new(self)
    }

    fn on_left_click(
        self,
        func: impl Fn(&mut EventCtx, &MouseEvent, &mut T, &Env) + 'static,
    ) -> ControllerHost<ControllerHost<Self, ExCursor<T>>, ExClick<T>> {
        self.with_cursor(Cursor::Pointer)
            .on_mouse_click(MouseButton::Left, func)
    }

    fn on_right_click(
        self,
        func: impl Fn(&mut EventCtx, &MouseEvent, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, ExClick<T>> {
        self.on_mouse_click(MouseButton::Right, func)
    }

    fn on_mouse_click(
        self,
        button: MouseButton,
        func: impl Fn(&mut EventCtx, &MouseEvent, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, ExClick<T>> {
        ControllerHost::new(self, ExClick::new(Some(button), func))
    }

    fn with_cursor(self, cursor: Cursor) -> ControllerHost<Self, ExCursor<T>> {
        ControllerHost::new(self, ExCursor::new(cursor))
    }

    fn on_command_async<U: Data + Send, V: Data + Send>(
        self,
        selector: Selector<U>,
        request: impl Fn(U) -> V + Sync + Send + 'static,
        preflight: impl Fn(&mut EventCtx, &mut T, U) + 'static,
        response: impl Fn(&mut EventCtx, &mut T, (U, V)) + 'static,
    ) -> OnCommandAsync<Self, T, U, V> {
        OnCommandAsync::new(
            self,
            selector,
            Box::new(preflight),
            Arc::new(request),
            Box::new(response),
        )
    }

    fn context_menu(
        self,
        func: impl Fn(&T) -> Menu<AppState> + 'static,
    ) -> ControllerHost<Self, ExClick<T>> {
        self.on_right_click(move |ctx, event, data, _env| {
            ctx.show_context_menu(func(data), event.window_pos);
        })
    }
}

impl<T: Data, W: Widget<T> + 'static> MyWidgetExt<T> for W {}
