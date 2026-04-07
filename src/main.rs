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
        firefly_ui::Input::Select => state.toc = false,
        firefly_ui::Input::Back => {}
        firefly_ui::Input::None => {}
    }
}

fn handle_page_input(state: &mut State) {
    let Some(manual) = state.manual.as_ref() else {
        return;
    };
    let Some(page) = manual.pages.get(state.page) else {
        return;
    };
    match state.input.get() {
        firefly_ui::Input::Up => {}
        firefly_ui::Input::Down => {}
        firefly_ui::Input::Left => {}
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
    let h = i32::from(font.char_height());
    let Some(manual) = state.manual.as_ref() else {
        return;
    };
    let Some(page) = manual.pages.get(state.page) else {
        return;
    };

    clear_screen(theme.bg);

    {
        let mut point = Point::new(
            (WIDTH - font.line_width_utf8(&page.title).cast_signed()) / 2,
            1 + h,
        );
        if false {
            point.x += 1;
            point.y += 1;
        }
        draw_text(&page.title, &font, point, theme.accent);
    };

    let mut point = Point::new(5, 1 + h * 2);
    for block in &page.content {
        match block {
            Block::H2(_) => todo!(),
            Block::H3(_) => todo!(),
            Block::P(inlines) => draw_inlines(&mut point, inlines, theme, &font),
            Block::Oli(inlines) => todo!(),
            Block::Uli(inlines) => todo!(),
            Block::A(_) => todo!(),
            Block::Img(_) => todo!(),
            Block::Quote(inlines) => todo!(),
            Block::Qr(_) => todo!(),
        }
    }
}

fn draw_inlines(point: &mut Point, inlines: &[Inline], theme: Theme, font: &Font) {
    let h = i32::from(font.char_height());
    for inline in inlines {
        let mut color = theme.primary;
        match inline.kind {
            InlineKind::Plain => {}
            InlineKind::Bold => color = theme.accent,
            InlineKind::Italic => color = theme.secondary,
            InlineKind::Image => {}
            InlineKind::Icon => {}
            InlineKind::Br => {
                point.y += h;
                return;
            }
        }
        draw_text(&inline.content, font, *point, color);
    }
    point.y += h;
}
