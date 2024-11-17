mod cmd;
mod data;
mod delegate;
mod error;
mod ui;
mod webapi;
mod widget;

use data::{config::Config, AppState};
use delegate::Delegate;
use druid::AppLauncher;
use env_logger::{Builder, Env};
use webapi::WebApi;

const ENV_LOG: &str = "PSST_LOG";
const ENV_LOG_STYLE: &str = "PSST_LOG_STYLE";

fn main() {
    // Setup logging from the env variables, with defaults.
    Builder::from_env(
        Env::new()
            .filter_or(ENV_LOG, "info")
            .write_style(ENV_LOG_STYLE),
    )
    .init();

    let config = Config::load().unwrap_or_default();
    let paginated_limit = config.paginated_limit;
    let state = AppState::default_with_config(config);
    WebApi::new(
        state.session.clone(),
        Config::proxy().as_deref(),
        Config::cache_dir(),
        paginated_limit,
    )
    .install_as_global();

    let delegate;
    let launcher;

    if state.config.has_credentials() {
        todo!();
    } else {
        // No configured credentials, open the account setup.
        let window = ui::account_setup_window();
        delegate = Delegate::with_preferences(window.id);
        launcher = AppLauncher::with_window(window).configure_env(ui::theme::setup)
    }

    launcher
        .delegate(delegate)
        .launch(state)
        .expect("Application launch");
}
