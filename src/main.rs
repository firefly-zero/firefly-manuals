#![no_std]
#![no_main]
extern crate alloc;
mod state;

use crate::state::*;
use firefly_rust::*;
use firefly_types::manuals::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.input.update();
    if state.toc {
        handle_toc_input(state);
    }
}

fn handle_toc_input(state: &mut State) {
    let Some(manual) = state.manual.as_ref() else {
        return;
    };
    match state.input.get() {
        firefly_ui::Input::Up => {
            if state.page > 0 {
                state.page -= 1;
            }
        }
        firefly_ui::Input::Down => {
            if state.page < manual.pages.len() - 1 {
                state.page += 1;
            }
        }
        firefly_ui::Input::Left => state.page = 0,
        firefly_ui::Input::Right => state.page = manual.pages.len() - 1,
        firefly_ui::Input::Select => state.toc = false,
        firefly_ui::Input::Back => {}
        firefly_ui::Input::None => {}
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    if state.toc {
        render_toc(state);
    }
}

fn render_toc(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    firefly_ui::draw_bg(theme);
    let Some(manual) = state.manual.as_ref() else {
        return;
    };
    let pressed = state.input.pressed();
    firefly_ui::draw_cursor(state.page as u32, theme, &font, pressed, 0);
    for (page, i) in manual.pages.iter().zip(1..) {
        let mut point = Point::new(20, 12 + i * 13);
        if pressed && i == state.page as i32 {
            point.x += 1;
            point.y += 1;
        }
        draw_text(&page.title, &font, point, theme.primary);
    }
}
