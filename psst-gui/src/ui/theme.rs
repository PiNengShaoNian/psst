use druid::{Color, Env, FontDescriptor, Key};

use crate::data::AppState;
pub use druid::theme::*;

pub const UI_FONT_MEDIUM: Key<FontDescriptor> = Key::new("app.ui-font-medium");

pub fn grid(m: f64) -> f64 {
    GRID * m
}

pub const GRID: f64 = 8.0;

pub fn setup(env: &mut Env, state: &AppState) {
    // TODO
}
