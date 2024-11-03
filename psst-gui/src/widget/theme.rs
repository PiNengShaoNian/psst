use druid::{Data, Env, LifeCycle, Widget};

use crate::{data::AppState, ui::theme};

pub struct ThemeScope<W> {
    inner: W,
    cached_env: Option<Env>,
}

impl<W> ThemeScope<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            cached_env: None,
        }
    }

    fn set_env(&mut self, data: &AppState, outer_env: &Env) {
        let mut themed_env = outer_env.clone();
        theme::setup(&mut themed_env, data);
        self.cached_env.replace(themed_env);
    }
}

impl<W: Widget<AppState>> Widget<AppState> for ThemeScope<W> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        self.inner
            .event(ctx, event, data, self.cached_env.as_ref().unwrap_or(env));
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = &event {
            self.set_env(data, env);
        }
        self.inner
            .lifecycle(ctx, event, data, self.cached_env.as_ref().unwrap_or(env));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        if !data.config.theme.same(&old_data.config.theme) {
            self.set_env(data, env);
            ctx.request_layout();
            ctx.request_paint();
        }
        self.inner
            .update(ctx, old_data, data, self.cached_env.as_ref().unwrap_or(env));
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> druid::Size {
        self.inner
            .layout(ctx, bc, data, self.cached_env.as_ref().unwrap_or(env))
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &Env) {
        self.inner
            .paint(ctx, data, self.cached_env.as_ref().unwrap_or(env));
    }
}
