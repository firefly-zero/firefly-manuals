use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use firefly_rust::*;
use firefly_types::manuals::*;

const LEFT: i32 = 5;

pub type Lines = Vec<Line>;

pub struct Line {
    pub point: Point,
    pub block: Block,
    pub words: Option<Vec<Word>>,
    pub image: Option<FileBuf>,
}

impl Line {
    pub fn new(point: Point, block: Block) -> Self {
        Self {
            point,
            block,
            words: None,
            image: None,
        }
    }
}

pub struct Word {
    pub point: Point,
    pub kind: InlineKind,
    pub content: String,
}

pub fn wrap_lines(page: &Page, font: &Font, target: Option<(&str, &str)>) -> Lines {
    let mut lines = Lines::new();
    let w = i32::from(font.char_width());
    let h = i32::from(font.char_height());
    let mut point = Point::new(LEFT, h);

    lines.push(Line::new(
        Point::new(
            (WIDTH - font.line_width_utf8(&page.title) as i32) / 2,
            point.y,
        ),
        Block::H2(page.title.clone()),
    ));
    point.y += h * 2;

    let mut number = 0;
    let mut unordered = false;
    for block in &page.content {
        if matches!(block, Block::Oli(_)) {
            number += 1;
        } else if number != 0 {
            number = 0;
            point.y += h;
        }
        if matches!(block, Block::Uli(_)) {
            unordered = true;
        } else if unordered {
            unordered = false;
            point.y += h;
        }

        match block {
            Block::H2(_) | Block::H3(_) | Block::A(_) => {
                point.x = LEFT;
                let line = Line::new(point, block.clone());
                lines.push(line);
                point.y += h * 2;
            }
            Block::P(inlines) => {
                point.x = LEFT;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Oli(inlines) => {
                let word = Word {
                    point: Point::new(LEFT, point.y),
                    kind: InlineKind::Bold,
                    content: alloc::format!("{}", number),
                };
                let mut line = Line::new(word.point, Block::P(Paragraph::new()));
                line.words = Some(vec![word]);
                lines.push(line);
                point.x = LEFT + w * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
            }
            Block::Uli(inlines) => {
                point.x = LEFT * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
            }
            Block::Quote(inlines) => {
                point.x = LEFT * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Img(name) => {
                if let Some((author_id, app_id)) = target {
                    let path = alloc::format!("roms/{author_id}/{app_id}/{name}");
                    if let Some(img) = sudo::load_file_buf(&path) {
                        let img_h = i32::from(img.as_image().height());
                        let mut line = Line::new(point, Block::Img(String::new()));
                        line.image = Some(img);
                        lines.push(line);
                        point.y += h + img_h;
                    }
                }
            }
            Block::Qr(url) => {
                lines.push(Line::new(
                    Point::new(LEFT, point.y - h),
                    Block::Qr(url.clone()),
                ));
                point.y += 50 - h;
            }
        }
    }

    lines
}

fn wrap_line(lines: &mut Lines, block: &Block, point: &mut Point, inlines: &[Inline], font: &Font) {
    let h = i32::from(font.char_height());
    let w = i32::from(font.char_width());
    let mut line = Line::new(*point, block.clone());
    let mut words = Vec::new();

    for inline in inlines {
        let mut first = true;
        for word in inline.content.split(|c: char| c.is_ascii_whitespace()) {
            // Handle whitespaces. There is always exatly one space
            // between every `word`. The `word` might be empty if
            // there are two spaces in a row or there is a trailing space.
            // We don't check line wrap for empty words, which means
            // spaces don't get wrapped around.
            if !first {
                point.x += w;
            }
            first = false;
            if word.is_empty() {
                continue;
            }

            let word_w = font.line_width_utf8(word) as i32;
            if point.x + word_w > WIDTH {
                line.words = Some(words);
                words = Vec::new();
                point.x = line.point.x;
                lines.push(line);
                line = Line::new(*point, block.clone());
                point.y += h;
            }
            words.push(Word {
                point: *point,
                kind: inline.kind,
                content: word.to_string(),
            });
            point.x += word_w;
        }
    }

    line.words = Some(words);
    lines.push(line);
    point.y += h;
}
