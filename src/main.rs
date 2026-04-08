#![no_std]
#![no_main]
extern crate alloc;
mod lines;
mod state;

use crate::{lines::*, state::*};
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
    } else {
        handle_page_input(state);
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
        firefly_ui::Input::Select => {
            state.toc = false;
            let font = state.font.as_font();
            let page = &manual.pages[state.page];
            let lines = wrap_lines(page, &font);
            state.lines = Some(lines);
        }
        firefly_ui::Input::Back => {}
        firefly_ui::Input::None => {}
    }
}

fn handle_page_input(state: &mut State) {
    let font = state.font.as_font();
    let h = i32::from(font.char_height());
    match state.input.get() {
        firefly_ui::Input::Up => {
            state.offset += h;
            if state.offset > 0 {
                state.offset = 0;
            }
        }
        firefly_ui::Input::Down => {
            state.offset -= h;
        }
        firefly_ui::Input::Left => state.offset = 0,
        firefly_ui::Input::Right => {}
        firefly_ui::Input::Select => {}
        firefly_ui::Input::Back => state.toc = true,
        firefly_ui::Input::None => {}
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    if state.toc {
        render_toc(state);
    } else {
        render_page(state);
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

fn render_page(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let Some(lines) = &state.lines else {
        return;
    };

    clear_screen(theme.bg);

    for line in lines {
        match &line.block {
            Block::H2(text) => {
                let point = Point::new(line.point.x, line.point.y + state.offset);
                draw_text(text, &font, point, theme.accent);
            }
            Block::H3(_) => todo!(),
            Block::P(_) | Block::Oli(_) | Block::Uli(_) => {
                let words = line.words.as_ref().unwrap();
                draw_words(words, state.offset, theme, &font);
            }
            Block::A(_) => todo!(),
            Block::Img(_) => todo!(),
            Block::Quote(_) => todo!(),
            Block::Qr(_) => todo!(),
        }
    }
}

fn draw_words(words: &[Word], offset: i32, theme: Theme, font: &Font) {
    for word in words {
        let mut color = theme.primary;
        match word.kind {
            InlineKind::Plain => {}
            InlineKind::Bold => color = theme.accent,
            InlineKind::Italic => color = theme.secondary,
            InlineKind::Image => {}
            InlineKind::Icon => {}
            InlineKind::Br => return,
        }
        let point = Point::new(word.point.x, word.point.y + offset);
        draw_text(&word.content, font, point, color);
    }
}
