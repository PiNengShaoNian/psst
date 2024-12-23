use druid::{widget::Controller, Env, Event, EventCtx, Widget};

use crate::{
    cmd,
    data::{
        config::{SortCriteria, SortOrder},
        AppState,
    },
};

pub struct SortController;

impl<W> Controller<AppState, W> for SortController
where
    W: Widget<AppState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(cmd::TOGGLE_SORT_ORDER) => {
                if data.config.sort_order == SortOrder::Ascending {
                    data.config.sort_order = SortOrder::Descending;
                } else {
                    data.config.sort_order = SortOrder::Ascending;
                }

                ctx.submit_command(cmd::NAVIGATE_REFRESH);
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(cmd::SORT_BY_TITLE) => {
                if data.config.sort_criteria != SortCriteria::Title {
                    data.config.sort_criteria = SortCriteria::Title;
                    ctx.submit_command(cmd::NAVIGATE_REFRESH);
                    ctx.set_handled();
                }
            }
            Event::Command(cmd) if cmd.is(cmd::SORT_BY_ALBUM) => {
                if data.config.sort_criteria != SortCriteria::Album {
                    data.config.sort_criteria = SortCriteria::Album;
                    ctx.submit_command(cmd::NAVIGATE_REFRESH);
                    ctx.set_handled();
                }
            }
            Event::Command(cmd) if cmd.is(cmd::SORT_BY_DATE_ADDED) => {
                if data.config.sort_criteria != SortCriteria::DateAdded {
                    data.config.sort_criteria = SortCriteria::DateAdded;
                    ctx.submit_command(cmd::NAVIGATE_REFRESH);
                    ctx.set_handled();
                }
            }
            Event::Command(cmd) if cmd.is(cmd::SORT_BY_ARTIST) => {
                if data.config.sort_criteria != SortCriteria::Artist {
                    data.config.sort_criteria = SortCriteria::Artist;
                    ctx.submit_command(cmd::NAVIGATE_REFRESH);
                    ctx.set_handled();
                }
            }
            Event::Command(cmd) if cmd.is(cmd::SORT_BY_DURATION) => {
                if data.config.sort_criteria != SortCriteria::Duration {
                    data.config.sort_criteria = SortCriteria::Duration;
                    ctx.submit_command(cmd::NAVIGATE_REFRESH);
                    ctx.set_handled();
                }
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
}
