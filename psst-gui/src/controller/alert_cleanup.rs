use std::time::Duration;

use druid::{widget::Controller, Event, Widget};

use crate::data::AppState;

pub struct AlertCleanupController;

const CLEANUP_INTERNAL: Duration = Duration::from_secs(1);

impl<W: Widget<AppState>> Controller<AppState, W> for AlertCleanupController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        match event {
            Event::WindowConnected => {
                ctx.request_timer(CLEANUP_INTERNAL);
            }
            Event::Timer(_) => {
                data.cleanup_alerts();
                ctx.request_timer(CLEANUP_INTERNAL);
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}
