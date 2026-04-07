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
    // ...
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
    firefly_ui::draw_bg(theme);
}
