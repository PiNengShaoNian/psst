use druid::{
    kurbo::Line,
    widget::{BackgroundBrush, Painter},
    Color, Data, KeyOrValue, RenderContext,
};

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
