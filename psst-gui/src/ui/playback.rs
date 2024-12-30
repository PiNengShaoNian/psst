use druid::{
    kurbo::{Affine, BezPath},
    widget::{Either, Flex, Maybe, SizedBox},
    BoxConstraints, Cursor, Data, Env, Event, LifeCycle, MouseButton, PaintCtx, Point, Rect,
    RenderContext, Size, Widget, WidgetExt, WidgetPod,
};

use crate::{
    cmd::{self, ADD_TO_QUEUE},
    data::{AppState, NowPlaying, Playback},
    widget::{Empty, MyWidgetExt},
};

use super::theme;

pub fn panel_widget() -> impl Widget<AppState> {
    let seek_bar = Maybe::or_empty(SeekBar::new).lens(Playback::now_playing);
    let item_info = Maybe::or_empty(playing_item_widget).lens(Playback::now_playing);
    let controls = Either::new(
        |playback, _| playback.now_playing.is_some(),
        player_widget(),
        Empty,
    );

    Flex::column()
        .with_child(seek_bar)
        .with_child(BarLayout::new(item_info, controls))
        .lens(AppState::playback)
        // .controller(PlaybackController::new())
        .on_command(ADD_TO_QUEUE, |_, _, data| {
            data.info_alert("Track added to queue.")
        })
}

fn playing_item_widget() -> impl Widget<NowPlaying> {
    // TODO
    SizedBox::empty()
}

fn player_widget() -> impl Widget<Playback> {
    // TODO
    Flex::row()
}

struct BarLayout<T, I, P> {
    item: WidgetPod<T, I>,
    player: WidgetPod<T, P>,
}

impl<T, I, P> BarLayout<T, I, P>
where
    T: Data,
    I: Widget<T>,
    P: Widget<T>,
{
    fn new(item: I, player: P) -> Self {
        Self {
            item: WidgetPod::new(item),
            player: WidgetPod::new(player),
        }
    }
}

impl<T, I, P> Widget<T> for BarLayout<T, I, P>
where
    T: Data,
    I: Widget<T>,
    P: Widget<T>,
{
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.item.event(ctx, event, data, env);
        self.player.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.item.lifecycle(ctx, event, data, env);
        self.player.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.item.update(ctx, data, env);
        self.player.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let max = bc.max();

        const PLAYER_OPTICAL_CENTER: f64 = 60.0 + theme::GRID * 2.0;

        // Layout the player with loose constraints.
        let player = self.player.layout(ctx, &bc.loosen(), data, env);
        let player_centered = max.width > player.width * 2.25;

        // Layout the item to the available space.
        let item_max = if player_centered {
            Size::new(max.width * 0.5 - PLAYER_OPTICAL_CENTER, max.height)
        } else {
            Size::new(max.width - player.width, max.height)
        };

        let item = self
            .item
            .layout(ctx, &BoxConstraints::new(Size::ZERO, item_max), data, env);
        let total = Size::new(max.width, player.height.max(item.height));

        // Put the item to the top left.
        self.item.set_origin(ctx, Point::ORIGIN);

        // Put the item to the top left.
        self.item.set_origin(ctx, Point::ORIGIN);

        // Put the player either to the center or to the right.
        let player_pos = if player_centered {
            Point::new(
                total.width * 0.5 - PLAYER_OPTICAL_CENTER,
                total.height * 0.5 - player.height * 0.5,
            )
        } else {
            Point::new(
                total.width - player.width,
                total.height * 0.5 - player.height * 0.5,
            )
        };
        self.player.set_origin(ctx, player_pos);

        total
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.item.paint(ctx, data, env);
        self.player.paint(ctx, data, env);
    }
}

struct SeekBar {
    loudness_path: BezPath,
}

impl SeekBar {
    fn new() -> Self {
        Self {
            loudness_path: BezPath::new(),
        }
    }
}

impl Widget<NowPlaying> for SeekBar {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        _data: &mut NowPlaying,
        _env: &druid::Env,
    ) {
        match event {
            Event::MouseMove(_) => {
                ctx.set_cursor(&Cursor::Pointer);
            }
            Event::MouseDown(mouse) => {
                if mouse.button == MouseButton::Left {
                    ctx.set_active(true);
                }
            }
            Event::MouseUp(mouse) => {
                if ctx.is_active() && mouse.button == MouseButton::Left {
                    if ctx.is_hot() {
                        let fraction = mouse.pos.x / ctx.size().width;
                        ctx.submit_command(cmd::PLAY_SEEK.with(fraction));
                    }
                    ctx.set_active(false);
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        _data: &NowPlaying,
        _env: &druid::Env,
    ) {
        match &event {
            LifeCycle::Size(_bounds) => {
                // self.loudness_path = compute_loudness_path(bounds, &data);
            }
            LifeCycle::HotChanged(_) => {
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &NowPlaying,
        data: &NowPlaying,
        _env: &druid::Env,
    ) {
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &NowPlaying,
        _env: &druid::Env,
    ) -> druid::Size {
        Size::new(bc.max().width, theme::grid(1.0))
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &NowPlaying, env: &druid::Env) {
        if self.loudness_path.is_empty() {
            paint_progress_bar(ctx, data, env)
        } else {
            paint_audio_analysis(ctx, data, &self.loudness_path, env)
        }
    }
}

fn paint_audio_analysis(ctx: &mut PaintCtx, data: &NowPlaying, path: &BezPath, env: &Env) {
    let bounds = ctx.size();

    let elapsed_time = data.progress.as_secs_f64();
    let total_time = data.item.duration().as_secs_f64();
    let elapsed_frac = elapsed_time / total_time;
    let elapsed_width = bounds.width * elapsed_frac;
    let elapsed = Size::new(elapsed_width, bounds.height).to_rect();

    let (elapsed_color, remaining_color) = if ctx.is_hot() {
        (env.get(theme::GREY_200), env.get(theme::GREY_500))
    } else {
        (env.get(theme::GREY_300), env.get(theme::GREY_600))
    };

    ctx.with_save(|ctx| {
        ctx.fill(path, &remaining_color);
        ctx.clip(elapsed);
        ctx.fill(path, &elapsed_color);
    });
}

fn paint_progress_bar(ctx: &mut PaintCtx, data: &NowPlaying, env: &Env) {
    let elapsed_time = data.progress.as_secs_f64();
    let total_time = data.item.duration().as_secs_f64();

    let (elapsed_color, remaining_color) = if ctx.is_hot() {
        (env.get(theme::GREY_200), env.get(theme::GREY_500))
    } else {
        (env.get(theme::GREY_300), env.get(theme::GREY_600))
    };

    let bounds = ctx.size();

    let elapsed_frac = elapsed_time / total_time;
    let elapsed_width = bounds.width * elapsed_frac;
    let remaining_width = bounds.width - elapsed_width;
    let elapsed = Size::new(elapsed_width, bounds.height).round();
    let remaining = Size::new(remaining_width, bounds.height).round();

    ctx.fill(
        Rect::from_origin_size(Point::ORIGIN, elapsed),
        &elapsed_color,
    );
    ctx.fill(
        Rect::from_origin_size(Point::new(elapsed.width, 0.0), remaining),
        &remaining_color,
    );
}
