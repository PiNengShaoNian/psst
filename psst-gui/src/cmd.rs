use druid::{Selector, WidgetId};

use crate::data::Nav;

// Widget IDs
pub const WIDGET_SEARCH_INPUT: WidgetId = WidgetId::reserved(1);

// Common
pub const SHOW_MAIN: Selector = Selector::new("app.show-main");
pub const SHOW_ACCOUNT_SETUP: Selector = Selector::new("app.show-initial");
pub const CLOSE_ALL_WINDOWS: Selector = Selector::new("app.close-all-windows");
pub const SET_FOCUS: Selector = Selector::new("app.set-focus");
pub const COPY: Selector<String> = Selector::new("app.copy-to-clipboard");

// Session
pub const SESSION_CONNECT: Selector = Selector::new("app.session-connect");
pub const LOG_OUT: Selector = Selector::new("app.log-out");

// Navigation
pub const NAVIGATE: Selector<Nav> = Selector::new("app.navigates");
