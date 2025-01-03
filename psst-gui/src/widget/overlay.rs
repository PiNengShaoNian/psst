use druid::{Data, Env, PaintCtx, Point, Vec2, Widget, WidgetPod};

pub enum OverlayPosition {
    Bottom,
}

pub struct Overlay<T, W, O> {
    inner: W,
    overlay: WidgetPod<T, O>,
    position: OverlayPosition,
}

impl<T, W, O> Overlay<T, W, O>
where
    O: Widget<T>,
{
    pub fn bottom(inner: W, overlay: O) -> Self {
        Self {
            inner,
            overlay: WidgetPod::new(overlay),
            position: OverlayPosition::Bottom,
        }
    }
}

impl<T, W, O> Widget<T> for Overlay<T, W, O>
where
    T: Data,
    W: Widget<T>,
    O: Widget<T>,
{
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        self.inner.event(ctx, event, data, env);
        self.overlay.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.inner.lifecycle(ctx, event, data, env);
        self.overlay.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.inner.update(ctx, old_data, data, env);
        self.overlay.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        let inner_size = self.inner.layout(ctx, bc, data, env);
        let over_size = self.overlay.layout(ctx, bc, data, env);
        let pos = match self.position {
            OverlayPosition::Bottom => {
                Point::ORIGIN + Vec2::new(0.0, inner_size.height - over_size.height)
            }
        };
        self.overlay.set_origin(ctx, pos);
        inner_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env);
        self.overlay.paint(ctx, data, env);
    }
}
