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
        firefly_ui::Input::Back => quit(),
        firefly_ui::Input::None => {}
    }
}

fn handle_page_input(state: &mut State) {
    let font = state.font.as_font();
    let h = i32::from(font.char_height());
    let max_offset = if let Some(lines) = &state.lines
        && let Some(last_line) = lines.last()
    {
        last_line.point.y - HEIGHT / 2
    } else {
        0
    };

    match state.input.get() {
        firefly_ui::Input::Up => {
            state.offset = i32::max(state.offset - h, 0);
        }
        firefly_ui::Input::Down => {
            state.offset = i32::min(state.offset + h, max_offset);
        }
        firefly_ui::Input::Left => {
            state.offset = 0;
        }
        firefly_ui::Input::Right => {
            state.offset = i32::min(state.offset + HEIGHT, max_offset);
        }
        firefly_ui::Input::Select => {}
        firefly_ui::Input::Back => {
            state.toc = true;
        }
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
    let h = font.char_height() as i32;
    let Some(lines) = &state.lines else {
        return;
    };

    clear_screen(theme.bg);

    for line in lines {
        match &line.block {
            Block::H2(text) => {
                let point = Point::new(line.point.x, line.point.y - state.offset);
                draw_rect(
                    Point::new(0, point.y - h + 2),
                    Size::new(WIDTH, h + 2),
                    Style::solid(theme.accent),
                );
                draw_text(text, &font, point, theme.bg);
            }
            Block::H3(text) => {
                let point = Point::new(line.point.x, line.point.y - state.offset);
                draw_rect(
                    Point::new(0, point.y - h + 2),
                    Size::new(WIDTH, h + 2),
                    Style::solid(theme.primary),
                );
                draw_text(text, &font, point, theme.bg);
            }
            Block::P(_) => {
                let words = line.words.as_ref().unwrap();
                draw_words(words, state.offset, theme, &font);
            }
            Block::Oli(_) => {
                let words = line.words.as_ref().unwrap();
                draw_words(words, state.offset, theme, &font);
            }
            Block::Uli(_) => {
                let words = line.words.as_ref().unwrap();
                draw_words(words, state.offset, theme, &font);
                let x = line.point.x / 2 - 1;
                let y = line.point.y - state.offset - 3;
                let point = Point::new(x, y);
                draw_circle(point, 3, Style::solid(theme.accent));
            }
            Block::A(_) => todo!(),
            Block::Img(_) => todo!(),
            Block::Quote(_) => {
                let words = line.words.as_ref().unwrap();
                draw_words(words, state.offset, theme, &font);
                for word in words {
                    let x = line.point.x / 2;
                    let y = word.point.y + 2 - state.offset;
                    draw_line(
                        Point::new(x, y - h),
                        Point::new(x, y),
                        LineStyle::new(theme.primary, 1),
                    );
                }
            }
            Block::Qr(text) => {
                draw_qr(text, line.point, theme.primary, theme.bg);
            }
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
        let point = Point::new(word.point.x, word.point.y - offset);
        draw_text(&word.content, font, point, color);
    }
}
