use druid::Selector;

use crate::data::Nav;

// Common
pub const SHOW_MAIN: Selector = Selector::new("app.show-main");

// Session
pub const SESSION_CONNECT: Selector = Selector::new("app.session-connect");
pub const LOG_OUT: Selector = Selector::new("app.log-out");

// Navigation
pub const NAVIGATE: Selector<Nav> = Selector::new("app.navigates");
