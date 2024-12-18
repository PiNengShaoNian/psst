use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::{self, JoinHandle},
};

use druid::{
    commands,
    widget::{
        Button, Controller, CrossAxisAlignment, Flex, Label, LineBreaking, MainAxisAlignment,
        RadioGroup, ViewSwitcher,
    },
    Event, LensExt, Selector, Widget, WidgetExt,
};
use psst_core::{connection::Credentials, oauth, session::SessionConfig};

use crate::{
    cmd,
    data::{
        config::{Authentication, PreferencesTab, Theme},
        AppState, Config,
    },
    webapi::WebApi,
    widget::{
        icons::{self, SvgIcon},
        MyWidgetExt,
    },
};

use super::theme;

pub fn account_setup_widget() -> impl Widget<AppState> {
    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new("Please insert your Spotify Premium credentials.")
                .with_font(theme::UI_FONT_MEDIUM)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new(
                "Psst connects only to the official servers, and does not store your password.",
            )
            .with_text_color(theme::PLACEHOLDER_COLOR)
            .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(6.0))
        .with_child(account_tab_widget(AccountTab::FirstSetup).expand_width())
        .padding(theme::grid(4.0))
}

pub fn preferences_widget() -> impl Widget<AppState> {
    const PROPAGATE_FLAGS: Selector = Selector::new("app.preferences.propagate-flags");

    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_child(
            tabs_widget()
                .padding(theme::grid(2.0))
                .background(theme::BACKGROUND_LIGHT),
        )
        .with_child(ViewSwitcher::new(
            |state: &AppState, _| state.preferences.active,
            |active, _, _| match active {
                PreferencesTab::General => general_tab_widget().boxed(),
                PreferencesTab::Account => account_tab_widget(AccountTab::InPreferences).boxed(),
                PreferencesTab::Cache => todo!(),
                PreferencesTab::About => todo!(),
            },
        ))
}

fn tabs_widget() -> impl Widget<AppState> {
    Flex::row()
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(tab_link_widget(
            "General",
            &icons::PREFERENCES,
            PreferencesTab::General,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Account",
            &icons::ACCOUNT,
            PreferencesTab::Account,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Cache",
            &icons::STORAGE,
            PreferencesTab::Cache,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "About",
            &icons::HEART,
            PreferencesTab::About,
        ))
}

fn tab_link_widget(
    text: &'static str,
    icon: &SvgIcon,
    tab: PreferencesTab,
) -> impl Widget<AppState> {
    Flex::column()
        .with_child(icon.scale(theme::ICON_SIZE_LARGE))
        .with_default_spacer()
        .with_child(Label::new(text).with_font(theme::UI_FONT_MEDIUM))
        .padding(theme::grid(1.0))
        .link()
        .rounded(theme::BUTTON_BORDER_RADIUS)
        .active(move |state: &AppState, _| tab == state.preferences.active)
        .on_left_click(move |_, _, state: &mut AppState, _| {
            state.preferences.active = tab;
        })
        .env_scope(|env, _| {
            env.set(theme::LINK_ACTIVE_COLOR, env.get(theme::BACKGROUND_DARK));
        })
}

fn general_tab_widget() -> impl Widget<AppState> {
    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true);

    // Theme
    col = col
        .with_child(Label::new("Theme").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            RadioGroup::column(vec![("Light", Theme::Light), ("Dark", Theme::Dark)])
                .lens(AppState::config.then(Config::theme)),
        );

    col
}

#[derive(Copy, Clone)]
enum AccountTab {
    FirstSetup,
    InPreferences,
}

fn account_tab_widget(tab: AccountTab) -> impl Widget<AppState> {
    let mut col = Flex::column().cross_axis_alignment(match tab {
        AccountTab::FirstSetup => CrossAxisAlignment::Center,
        AccountTab::InPreferences => CrossAxisAlignment::Start,
    });

    if matches!(tab, AccountTab::InPreferences) {
        col = col
            .with_child(Label::new("Credentials").with_font(theme::UI_FONT_MEDIUM))
            .with_spacer(theme::grid(2.0));
    }

    col = col
        .with_child(Button::new("Log in with Spotify").on_click(|ctx, _, _| {
            ctx.submit_command(Authenticate::REQUEST);
        }))
        .with_spacer(theme::grid(1.0));
    // TODO

    if matches!(tab, AccountTab::InPreferences) {
        col = col.with_child(Button::new("Log Out").on_click(|ctx, _, _| {
            ctx.submit_command(cmd::LOG_OUT);
        }))
    }

    col.controller(Authenticate::new(tab))
}

struct Authenticate {
    tab: AccountTab,
    thread: Option<JoinHandle<()>>,
}

impl Authenticate {
    fn new(tab: AccountTab) -> Self {
        Self { tab, thread: None }
    }
}

impl Authenticate {
    const REQUEST: Selector = Selector::new("app.preferences.authenticate-request");
    const RESPONSE: Selector<Result<Credentials, String>> =
        Selector::new("app.preferences.authenticate-response");
}

impl<W: Widget<AppState>> Controller<AppState, W> for Authenticate {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &druid::Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(Self::REQUEST) => {
                data.preferences.auth.result.defer_default();

                let (auth_url, pkce_verifier) = oauth::generate_auth_url(8888);
                if open::that(&auth_url).is_err() {
                    data.error_alert("Failed to open browser");
                    return;
                }

                let config = data.preferences.auth.session_config();
                let widget_id = ctx.widget_id();
                let event_sink = ctx.get_external_handle();
                let thread = thread::spawn(move || {
                    match oauth::get_authcode_listener(
                        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888),
                        std::time::Duration::from_secs(300),
                    ) {
                        Ok(code) => {
                            let token = oauth::exchange_code_for_token(8888, code, pkce_verifier);
                            let response =
                                Authentication::authenticate_and_get_credentials(SessionConfig {
                                    login_creds: Credentials::from_access_token(token),
                                    ..config
                                });
                            event_sink
                                .submit_command(Self::RESPONSE, response, widget_id)
                                .unwrap();
                        }
                        Err(e) => {
                            event_sink
                                .submit_command(Self::RESPONSE, Err(e), widget_id)
                                .unwrap();
                        }
                    }
                });
            }
            Event::Command(cmd) if cmd.is(Self::RESPONSE) => {
                self.thread.take();

                let result = cmd
                    .get_unchecked(Self::RESPONSE)
                    .to_owned()
                    .map(|credentials| {
                        let username = credentials.username.clone().unwrap_or_default();
                        WebApi::global().load_local_tracks(&username);
                        data.config.store_credentials(credentials);
                        data.config.save();
                    });
                let is_ok = result.is_ok();

                data.preferences.auth.result.resolve_or_reject((), result);

                if is_ok {
                    match &self.tab {
                        AccountTab::FirstSetup => {
                            ctx.submit_command(cmd::SHOW_MAIN);
                            ctx.submit_command(commands::CLOSE_WINDOW);
                        }
                        AccountTab::InPreferences => {
                            ctx.submit_command(cmd::SESSION_CONNECT);
                        }
                    }
                }
                data.preferences.auth.access_token.clear();
                ctx.set_handled();
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }
}
