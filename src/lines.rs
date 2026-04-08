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
}

pub struct Word {
    pub point: Point,
    pub kind: InlineKind,
    pub content: String,
}

pub fn wrap_lines(page: &Page, font: &Font) -> Lines {
    let mut lines = Lines::new();
    let w = i32::from(font.char_width());
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
                let word = Word {
                    point: Point::new(LEFT, point.y),
                    kind: InlineKind::Bold,
                    content: alloc::format!("{}", number),
                };
                lines.push(Line {
                    point: word.point,
                    block: Block::P(Paragraph::new()),
                    words: Some(vec![word]),
                });
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
            Block::Img(_) => point.y += h, // TODO
            Block::Qr(url) => {
                lines.push(Line {
                    point: Point::new(LEFT, point.y - h),
                    block: Block::Qr(url.clone()),
                    words: None,
                });
                point.y += 50 - h;
            }
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
