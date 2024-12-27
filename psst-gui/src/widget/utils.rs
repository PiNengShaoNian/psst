use druid::{
    kurbo::{Line, Shape},
    widget::{BackgroundBrush, Painter},
    Color, Data, KeyOrValue, RenderContext, Size, Widget, WidgetId,
};

pub struct Clip<S, W> {
    shape: S,
    inner: W,
}

impl<S, W> Clip<S, W> {
    pub fn new(shape: S, inner: W) -> Self {
        Self { shape, inner }
    }
}

impl<T: Data, S: Shape, W: Widget<T>> Widget<T> for Clip<S, W> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        self.inner.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        let size = self.inner.layout(ctx, bc, data, env);
        let bbox = self.shape.bounding_box().size();
        Size::new(size.width.min(bbox.width), size.height.min(bbox.height))
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        ctx.with_save(|ctx| {
            ctx.clip(&self.shape);
            self.inner.paint(ctx, data, env);
        });
    }

    fn id(&self) -> Option<WidgetId> {
        self.inner.id()
    }
}

pub enum Border {
    Top,
    Bottom,
}

impl Border {
    pub fn with_color<T: Data>(
        self,
        color: impl Into<KeyOrValue<Color>>,
    ) -> impl Into<BackgroundBrush<T>> {
        let color = color.into();

        Painter::new(move |ctx, _, env| {
            let h = 1.0;
            let y = match self {
                Self::Top => h / 2.0,
                Self::Bottom => ctx.size().height - h / 2.0,
            };
            let line = Line::new((0.0, y), (ctx.size().width, y));
            ctx.stroke(line, &color.resolve(env), h);
        })
    }
}
