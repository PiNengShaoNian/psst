use std::marker::PhantomData;

use druid::{widget::Controller, Cursor, Data, Event, Widget};

pub struct ExCursor<T> {
    cursor: Cursor,
    phantom: PhantomData<T>,
}

impl<T: Data> ExCursor<T> {
    pub fn new(cursor: Cursor) -> Self {
        Self {
            cursor,
            phantom: PhantomData,
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for ExCursor<T> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        if let Event::MouseMove(_) = event {
            ctx.set_cursor(&self.cursor);
        }

        child.event(ctx, event, data, env);
    }
}
