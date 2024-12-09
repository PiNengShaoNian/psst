use druid::{
    widget::{CrossAxisAlignment, Flex, Label, Spinner},
    Data, Widget, WidgetExt,
};

use crate::{error::Error, widget::icons};

use super::theme;

pub fn spinner_widget<T: Data>() -> impl Widget<T> {
    Spinner::new().center()
}

pub fn error_widget() -> impl Widget<Error> {
    let icon = icons::ERROR
        .scale((theme::grid(3.0), theme::grid(3.0)))
        .with_color(theme::PLACEHOLDER_COLOR);
    let error = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            Label::new("Error:")
                .with_font(theme::UI_FONT_MEDIUM)
                .with_text_color(theme::PLACEHOLDER_COLOR),
        )
        .with_child(
            Label::dynamic(|err: &Error, _| err.to_string())
                .with_text_size(theme::TEXT_SIZE_SMALL)
                .with_text_color(theme::PLACEHOLDER_COLOR),
        );

    Flex::row()
        .with_child(icon)
        .with_default_spacer()
        .with_child(error)
        .padding((0.0, theme::grid(6.0)))
        .center()
}
