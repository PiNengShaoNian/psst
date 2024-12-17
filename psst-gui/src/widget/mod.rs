mod dispatcher;
mod empty;
pub mod icons;
mod link;
mod promise;
pub mod theme;
mod utils;

use std::{sync::Arc, time::Duration};

use druid::{
    widget::{ControllerHost, Padding},
    Cursor, Data, Env, EventCtx, Insets, Menu, MouseButton, MouseEvent, Selector, Widget,
};
pub use empty::Empty;
pub use link::Link;
pub use promise::Async;
pub use utils::Border;

use crate::{
    controller::{ExClick, ExCursor, ExScroll, OnCommand, OnCommandAsync, OnDebounce},
    data::{AppState, SliderScrollScale},
};

pub trait MyWidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn link(self) -> Link<T> {
        Link::new(self)
    }

    fn padding_left(self, p: f64) -> Padding<T, Self> {
        Padding::new(Insets::new(p, 0.0, 0.0, 0.0), self)
    }

    fn padding_right(self, p: f64) -> Padding<T, Self> {
        Padding::new(Insets::new(0.0, 0.0, p, 0.0), self)
    }

    fn padding_horizontal(self, p: f64) -> Padding<T, Self> {
        Padding::new(Insets::new(p, 0.0, p, 0.0), self)
    }

    fn on_debounce(
        self,
        duration: Duration,
        handler: impl Fn(&mut EventCtx, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, OnDebounce<T>> {
        ControllerHost::new(self, OnDebounce::trailing(duration, handler))
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

    fn on_scroll(
        self,
        scale_picker: impl Fn(&mut T) -> &SliderScrollScale + 'static,
        action: impl Fn(&mut EventCtx, &mut T, &Env, f64) + 'static,
    ) -> ControllerHost<Self, ExScroll<T>> {
        ControllerHost::new(self, ExScroll::new(scale_picker, action))
    }

    fn with_cursor(self, cursor: Cursor) -> ControllerHost<Self, ExCursor<T>> {
        ControllerHost::new(self, ExCursor::new(cursor))
    }

    fn on_command<U, F>(
        self,
        selector: Selector<U>,
        func: F,
    ) -> ControllerHost<Self, OnCommand<U, F>>
    where
        U: 'static,
        F: Fn(&mut EventCtx, &U, &mut T),
    {
        ControllerHost::new(self, OnCommand::new(selector, func))
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
