use crate::*;
use alloc::string::String;
use core::mem::MaybeUninit;
use firefly_rust::*;
use firefly_types::Encode;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub input: firefly_ui::InputManager,

    /// Full ID of the target app.
    ///
    /// Used for delayed loading of assets.
    target: Option<(String, String)>,

    /// The parsed game manual.
    ///
    /// If None, the game has no manual.
    pub manual: Option<Manual>,

    /// If true, the manual has only one page.
    ///
    /// Use to hide the Table of Contents.
    pub single_page: bool,

    /// If true, the Table of Contents is currently open.
    ///
    /// If false, the selected page is open instead.
    pub toc: bool,

    /// The index of the currently selected page.
    pub page: usize,

    /// The wrapped lines of the currently open page.
    pub lines: Option<Lines>,

    /// OY offset from the page top to the screen top.
    ///
    /// In other words, this is the scroll position for the open page.
    pub offset: i32,
}

impl State {
    /// Open the currently selected page.
    pub fn open_page(&mut self) {
        let Some(manual) = self.manual.as_ref() else {
            log_error("trying to open page without manual");
            return;
        };
        self.toc = false;
        let font = self.font.as_font();
        let page = &manual.pages[self.page];
        let target = self.target.as_ref();
        let target = target.map(|(a, b)| (a.as_str(), b.as_str()));
        let lines = wrap_lines(page, &font, target);
        self.lines = Some(lines);
    }
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe {
        STATE.assume_init_mut()
    }
}

pub fn load_state() {
    let settings = get_settings(get_me());
    let encoding = settings.language.encoding();
    let font = load_file_buf(encoding).unwrap_or_else(|| load_file_buf("ascii").unwrap());
    let target = load_target();
    let manual = match &target {
        Some((author_id, app_id)) => load_manual(author_id, app_id),
        None => None,
    };

    if target.is_none() {
        log_error("failed to load target");
    } else if manual.is_none() {
        log_error("app has no manual");
    }
    let n_pages = if let Some(manual) = manual.as_ref() {
        manual.pages.len()
    } else {
        quit();
        0
    };

    let mut state = State {
        font,
        settings,
        manual,
        target,

        single_page: n_pages == 1,
        toc: true,
        page: 0,
        input: firefly_ui::InputManager::new(),
        lines: None,
        offset: 0,
    };
    if state.single_page {
        state.open_page();
    }
    #[allow(static_mut_refs)]
    unsafe {
        STATE.write(state)
    };
}

fn load_manual(author_id: &str, app_id: &str) -> Option<Manual> {
    let manual_path = alloc::format!("roms/{author_id}/{app_id}/_manual");
    let raw = sudo::load_file_buf(&manual_path)?;
    // TODO: hide pages based on badge and score conditions.
    Manual::decode(raw.as_bytes()).ok()
}

/// Read the ID of the app to be removed.
fn load_target() -> Option<(String, String)> {
    let raw = load_file_buf("target")?;
    let raw = raw.as_bytes();
    let raw = raw.trim_ascii();
    let raw = alloc::str::from_utf8(raw).ok()?;
    let (author, app) = split_by(raw, '.')?;
    let target = (String::from(author), String::from(&app[1..]));
    Some(target)
}

/// Split the string once at the given character.
fn split_by(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut split_at = None;
    let sep: u8 = sep.try_into().unwrap();
    for (i, ch) in input.bytes().enumerate() {
        if ch == sep {
            split_at = Some(i);
            break;
        }
    }
    let split_at = split_at?;
    Some(input.split_at(split_at))
}
