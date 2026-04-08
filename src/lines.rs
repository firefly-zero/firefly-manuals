use alloc::{
    string::{String, ToString},
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
}

pub struct Word {
    pub point: Point,
    pub kind: InlineKind,
    pub content: String,
}

pub fn wrap_lines(page: &Page, font: &Font) -> Lines {
    let mut lines = Lines::new();
    let h = i32::from(font.char_height());
    let mut point = Point::new(LEFT, h);

    lines.push(Line {
        point: Point::new(
            (WIDTH - font.line_width_utf8(&page.title) as i32) / 2,
            point.y,
        ),
        block: Block::H2(page.title.clone()),
        words: None,
    });
    point.y += h * 2;

    for block in &page.content {
        match block {
            Block::H2(_) | Block::H3(_) | Block::A(_) => {
                point.x = LEFT;
                let line = Line {
                    point,
                    block: block.clone(),
                    words: None,
                };
                lines.push(line);
                point.y += h * 2;
            }
            Block::P(inlines) => {
                point.x = LEFT;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Oli(inlines) => {
                point.x = LEFT * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Uli(inlines) => {
                point.x = LEFT * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Quote(inlines) => {
                point.x = LEFT * 2;
                wrap_line(&mut lines, block, &mut point, inlines, font);
                point.y += h;
            }
            Block::Img(_) => point.y += h, // TODO
            Block::Qr(_) => point.y += h,  // TODO
        }
    }

    lines
}

fn wrap_line(lines: &mut Lines, block: &Block, point: &mut Point, inlines: &[Inline], font: &Font) {
    let h = i32::from(font.char_height());
    let w = i32::from(font.char_width());
    let mut line = Line {
        point: *point,
        block: block.clone(),
        words: None,
    };
    let mut words = Vec::new();

    for inline in inlines {
        for word in inline.content.split_ascii_whitespace() {
            let word_w = font.line_width_utf8(word) as i32;
            if point.x + word_w > WIDTH {
                line.words = Some(words);
                words = Vec::new();
                point.x = line.point.x;
                lines.push(line);
                line = Line {
                    point: *point,
                    block: block.clone(),
                    words: None,
                };
                point.y += h;
            }
            words.push(Word {
                point: *point,
                kind: inline.kind,
                content: word.to_string(),
            });
            point.x += word_w + w;
        }
    }

    line.words = Some(words);
    lines.push(line);
    point.y += h;
}
