use druid::{widget::Controller, Event, EventCtx, Widget};

use crate::{
    cmd,
    data::{AppState, Nav, SpotifyUrl},
    ui::search,
};

pub struct NavController;

impl NavController {
    fn load_route_data(&self, ctx: &mut EventCtx, data: &mut AppState) {
        match &data.nav {
            Nav::Home => {}
            Nav::SearchResults(query) => {
                if let Some(link) = SpotifyUrl::parse(query) {
                    ctx.submit_command(search::OPEN_LINK.with(link));
                } else if !data.search.results.contains(query) {
                    ctx.submit_command(search::LOAD_RESULTS.with(query.to_owned()));
                }
            }
            _ => {
                todo!()
            }
        }
    }
}

impl<W> Controller<AppState, W> for NavController
where
    W: Widget<AppState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(cmd::NAVIGATE) => {
                let nav = cmd.get_unchecked(cmd::NAVIGATE);
                data.navigate(nav);
                ctx.set_handled();
                self.load_route_data(ctx, data);
            }
            Event::Command(cmd) if cmd.is(cmd::NAVIGATE_BACK) => {
                let count = cmd.get_unchecked(cmd::NAVIGATE_BACK);
                for _ in 0..*count {
                    data.navigate_back();
                }
                ctx.set_handled();
                self.load_route_data(ctx, data);
            }
            Event::Command(cmd) if cmd.is(cmd::NAVIGATE_REFRESH) => {
                data.refresh();
                ctx.set_handled();
                self.load_route_data(ctx, data);
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
}
