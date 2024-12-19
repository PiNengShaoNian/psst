use druid::{commands, AppDelegate, DelegateCtx, Handled, WindowId};
use open::commands;

use crate::{
    cmd,
    data::{AppState, Config},
    ui,
};

pub struct Delegate {
    main_window: Option<WindowId>,
    preferences_window: Option<WindowId>,
    credits_window: Option<WindowId>,
}

impl Delegate {
    pub fn new() -> Self {
        Self {
            main_window: None,
            preferences_window: None,
            credits_window: None,
        }
    }

    pub fn with_main(main_window: WindowId) -> Self {
        let mut this = Self::new();
        this.main_window.replace(main_window);
        this
    }

    pub fn with_preferences(preferences_window: WindowId) -> Self {
        let mut this = Self::new();
        this.preferences_window.replace(preferences_window);
        this
    }

    fn show_main(&mut self, config: &Config, ctx: &mut DelegateCtx) {
        match self.main_window {
            Some(id) => {
                ctx.submit_command(commands::SHOW_WINDOW.to(id));
            }
            None => {
                let window = ui::main_window(config);
                self.main_window.replace(window.id);
                ctx.new_window(window);
            }
        }
    }

    fn show_account_setup(&mut self, ctx: &mut DelegateCtx) {
        match self.preferences_window {
            Some(id) => {
                ctx.submit_command(commands::SHOW_WINDOW.to(id));
            }
            None => {
                let window = ui::account_setup_window();
                self.preferences_window.replace(window.id);
                ctx.new_window(window);
            }
        }
    }

    fn show_preferences(&mut self, ctx: &mut DelegateCtx) {
        match self.preferences_window {
            Some(id) => {
                ctx.submit_command(commands::SHOW_WINDOW.to(id));
            }
            None => {
                let window = ui::preferences_window();
                self.preferences_window.replace(window.id);
                ctx.new_window(window);
            }
        }
    }

    fn close_all_windows(&mut self, ctx: &mut DelegateCtx) {
        ctx.submit_command(commands::CLOSE_ALL_WINDOWS);
        self.main_window = None;
        self.preferences_window = None;
        self.credits_window = None;
    }
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(cmd::SHOW_MAIN) {
            self.show_main(&data.config, ctx);
            Handled::Yes
        } else if cmd.is(cmd::SHOW_ACCOUNT_SETUP) {
            self.show_account_setup(ctx);
            Handled::Yes
        } else if cmd.is(commands::SHOW_PREFERENCES) {
            self.show_preferences(ctx);
            Handled::Yes
        } else if cmd.is(cmd::CLOSE_ALL_WINDOWS) {
            self.close_all_windows(ctx);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
