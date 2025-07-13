use druid::widget::{Button, Flex, Label, Scroll, TextBox};
use druid::{Widget, WidgetExt, Selector};
use crate::gui::AppState;

pub const BUTTON_PRESSED: Selector<String> = Selector::new("button-pressed");
pub const CALCULATE_PRESSED: Selector<()> = Selector::new("calculate-pressed");

pub fn build_ui() -> impl Widget<AppState> {
    let top_panel = build_top_panel();
    let bottom_panel = build_bottom_panel();
    
    Flex::column()
        .with_child(top_panel)
        .with_flex_child(bottom_panel, 1.0)
}

fn build_top_panel() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Label::new("Save File Path:").fix_width(120.0))
                .with_flex_child(
                    TextBox::new()
                        .lens(AppState::save_file_path)
                        .expand_width(),
                    1.0
                )
                .padding(5.0)
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Scripts Directory:").fix_width(120.0))
                .with_flex_child(
                    TextBox::new()
                        .lens(AppState::scripts_dir_path)
                        .expand_width(),
                    1.0
                )
                .padding(5.0)
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Script File:").fix_width(120.0))
                .with_flex_child(
                    TextBox::new()
                        .lens(AppState::script_file)
                        .expand_width(),
                    1.0
                )
                .padding(5.0)
        )
        .padding(10.0)
}

fn build_bottom_panel() -> impl Widget<AppState> {
    let text_display = Scroll::new(
        Label::new(|data: &AppState, _env: &_| data.text_output.clone())
            .padding(10.0)
    ).expand_width();
    
    let button_panel = build_button_panel();
    
    Flex::row()
        .with_flex_child(text_display, 1.0)
        .with_child(button_panel)
        .must_fill_main_axis(true)
        .expand_width()
}

fn build_button_panel() -> impl Widget<AppState> {
    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Fill)
        .with_child(
            Button::new("Top")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(BUTTON_PRESSED.with("Top".to_string()));
                })
        )
        .with_child(
            Button::new("High")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(BUTTON_PRESSED.with("High".to_string()));
                })
        )
        .with_child(
            Button::new("Middle")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(BUTTON_PRESSED.with("Middle".to_string()));
                })
        )
        .with_child(
            Button::new("Low")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(BUTTON_PRESSED.with("Low".to_string()));
                })
        )
        .with_child(
            Button::new("Bottom")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(BUTTON_PRESSED.with("Bottom".to_string()));
                })
        )
        .with_child(
            Button::new("Calculate")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(CALCULATE_PRESSED);
                })
                .padding((0.0, 5.0))
        )
}