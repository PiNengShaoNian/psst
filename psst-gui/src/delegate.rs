use druid::{AppDelegate, WindowId};

use crate::data::AppState;

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

    pub fn with_preferences(preferences_window: WindowId) -> Self {
        let mut this = Self::new();
        this.preferences_window.replace(preferences_window);
        this
    }
}

impl AppDelegate<AppState> for Delegate {}
