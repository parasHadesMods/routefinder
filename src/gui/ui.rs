use druid::widget::{Button, Flex, Label, Scroll, TextBox, SizedBox};
use druid::{Widget, WidgetExt, Selector, Event, EventCtx, Env, LifeCycle, LifeCycleCtx};
use druid::widget::Controller;
use std::sync::{Arc, Mutex};
use crate::gui::AppState;

pub const BUTTON_PRESSED: Selector<String> = Selector::new("button-pressed");
pub const CALCULATE_PRESSED: Selector<()> = Selector::new("calculate-pressed");

// Shared state to track if any text field has focus
type TextFieldFocusState = Arc<Mutex<bool>>;

struct KeyboardController {
    focus_state: TextFieldFocusState,
}

struct TextFieldController {
    focus_state: TextFieldFocusState,
}

impl<W: Widget<AppState>> Controller<AppState, W> for KeyboardController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        // Handle mouse events to request focus
        if let Event::MouseDown(_) = event {
            ctx.request_focus();
        }
        
        
        // Let child widgets handle the event first
        child.event(ctx, event, data, env);
        
        // Only process keyboard events if no text field has focus
        if let Event::KeyDown(key_event) = event {
            let text_field_has_focus = *self.focus_state.lock().unwrap();
            if !text_field_has_focus {
                match key_event.key {
                    druid::keyboard_types::Key::Character(ref c) => {
                        match c.to_uppercase().as_str() {
                            "T" => {
                                ctx.submit_command(BUTTON_PRESSED.with("Top".to_string()));
                                ctx.set_handled();
                            }
                            "H" => {
                                ctx.submit_command(BUTTON_PRESSED.with("High".to_string()));
                                ctx.set_handled();
                            }
                            "M" => {
                                ctx.submit_command(BUTTON_PRESSED.with("Middle".to_string()));
                                ctx.set_handled();
                            }
                            "L" => {
                                ctx.submit_command(BUTTON_PRESSED.with("Low".to_string()));
                                ctx.set_handled();
                            }
                            "B" => {
                                ctx.submit_command(BUTTON_PRESSED.with("Bottom".to_string()));
                                ctx.set_handled();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

}

impl<W: Widget<AppState>> Controller<AppState, W> for TextFieldController {
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        if let LifeCycle::FocusChanged(gained_focus) = event {
            *self.focus_state.lock().unwrap() = *gained_focus;
        }
        child.lifecycle(ctx, event, data, env);
    }
}

fn create_button_with_underlined_first_letter(text: &str) -> impl Widget<AppState> {
    let chars: Vec<char> = text.chars().collect();
    let formatted_text = if !chars.is_empty() {
        let first_char = chars[0];
        let remaining: String = chars[1..].iter().collect();
        format!("{}\u{0332}{}", first_char, remaining)
    } else {
        text.to_string()
    };
    
    let button_name = text.to_string();
    
    Button::new(formatted_text)
        .on_click(move |_ctx, _data, _env| {
            _ctx.submit_command(BUTTON_PRESSED.with(button_name.clone()));
        })
}

pub fn build_ui() -> impl Widget<AppState> {
    let focus_state = Arc::new(Mutex::new(false));
    let top_panel = build_top_panel(focus_state.clone());
    let bottom_panel = build_bottom_panel();
    
    Flex::column()
        .with_child(top_panel)
        .with_flex_child(bottom_panel, 1.0)
        .controller(KeyboardController { focus_state })
        .expand()
}

fn build_top_panel(focus_state: TextFieldFocusState) -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Label::new("Save File Path:").fix_width(120.0))
                .with_flex_child(
                    TextBox::new()
                        .lens(AppState::save_file_path)
                        .expand_width()
                        .controller(TextFieldController { focus_state: focus_state.clone() }),
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
                        .expand_width()
                        .controller(TextFieldController { focus_state: focus_state.clone() }),
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
                        .expand_width()
                        .controller(TextFieldController { focus_state: focus_state.clone() }),
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
        .with_child(create_button_with_underlined_first_letter("Top"))
        .with_child(create_button_with_underlined_first_letter("High"))
        .with_child(create_button_with_underlined_first_letter("Middle"))
        .with_child(create_button_with_underlined_first_letter("Low"))
        .with_child(create_button_with_underlined_first_letter("Bottom"))
        .with_child(
            Button::new("Calculate")
                .on_click(|_ctx, _data, _env| {
                    _ctx.submit_command(CALCULATE_PRESSED);
                })
                .padding((0.0, 5.0))
        )
}