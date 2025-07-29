use druid::widget::{Button, Flex, Label, Scroll, TextBox};
use druid::{Widget, WidgetExt, Selector};
use crate::sack_finder::AppState;

pub const CALCULATE_PRESSED: Selector<()> = Selector::new("calculate-pressed");
pub const CLEAR_PRESSED: Selector<()> = Selector::new("clear-pressed");

fn create_input_field(label: &str, lens: impl druid::Lens<AppState, String> + 'static, 
                     enabled: bool) -> impl Widget<AppState> {
    Flex::row()
        .with_child(Label::new(label).fix_width(100.0))
        .with_flex_child(
            TextBox::new()
                .lens(lens)
                .disabled_if(move |_data: &AppState, _env| !enabled)
                .expand_width(),
            1.0
        )
        .padding(5.0)
}

pub fn build_ui() -> impl Widget<AppState> {
    let top_panel = build_top_config_panel();
    let bottom_panel = build_bottom_panel();
    
    Flex::column()
        .with_child(top_panel)
        .with_flex_child(bottom_panel, 1.0)
        .expand()
}

fn build_input_panel() -> impl Widget<AppState> {
    Flex::column()
        .with_child(create_input_field("Assault:", AppState::assault, true))
        .with_child(create_input_field("Grasp:", AppState::grasp, false))
        .with_child(create_input_field("Ambush:", AppState::ambush, true))
        .with_child(create_input_field("Favor:", AppState::favor, true))
        .with_child(create_input_field("Lunge:", AppState::lunge, true))
        .with_child(create_input_field("Soul:", AppState::soul, true))
        .with_child(create_input_field("Strike:", AppState::strike, true))
        .with_child(create_input_field("Eclipse:", AppState::eclipse, true))
        .with_child(create_input_field("Affluence:", AppState::affluence, false))
        .with_child(create_input_field("Shot:", AppState::shot, true))
        .with_child(create_input_field("Flourish:", AppState::flourish, true))
        .with_child(create_input_field("Defiance:", AppState::defiance, false))
        .with_child(build_button_panel())
        .padding(10.0)
}

fn build_button_panel() -> impl Widget<AppState> {
    Flex::row()
        .with_child(
            Button::new("Calculate")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(CALCULATE_PRESSED);
                })
                .disabled_if(|data: &AppState, _env| !data.is_valid())
                .padding((0.0, 5.0))
        )
        .with_child(
            Button::new("Clear")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(CLEAR_PRESSED);
                })
                .padding((5.0, 5.0))
        )
        .padding(10.0)
}

fn build_bottom_panel() -> impl Widget<AppState> {
    let text_display = Scroll::new(
        Label::new(|data: &AppState, _env: &_| data.text_output.clone())
            .padding(10.0)
    ).expand();
    
    let input_panel = build_input_panel()
        .fix_width(200.0);

    Flex::row()
        .with_flex_child(text_display, 1.0)
        .with_child(input_panel)
        .must_fill_main_axis(true)
        .expand()
}

fn build_top_config_panel() -> impl Widget<AppState> {
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
        .with_child(
            Flex::row()
                .with_child(Label::new("Found Seed:").fix_width(120.0))
                .with_child(
                    Label::new(|data: &AppState, _env: &_| {
                        match data.found_seed {
                            Some(seed) => seed.to_string(),
                            None => "None".to_string(),
                        }
                    })
                    .expand_width()
                )
                .padding(5.0)
        )
        .padding(10.0)
}