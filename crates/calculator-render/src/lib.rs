use std::fs;
use std::io;
use std::path::PathBuf;

use calculator_core::{Calculator, Key, Op};
use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BODY, CAPTION,
};

const KEYS: [&[(Key, &str)]; 5] = [
    &[
        (Key::Clear, "Clear"),
        (Key::Backspace, "Del"),
        (Key::Negate, "±"),
        (Key::Operator(Op::Divide), "÷"),
    ],
    &[
        (Key::Digit(7), "7"),
        (Key::Digit(8), "8"),
        (Key::Digit(9), "9"),
        (Key::Operator(Op::Multiply), "×"),
    ],
    &[
        (Key::Digit(4), "4"),
        (Key::Digit(5), "5"),
        (Key::Digit(6), "6"),
        (Key::Operator(Op::Subtract), "-"),
    ],
    &[
        (Key::Digit(1), "1"),
        (Key::Digit(2), "2"),
        (Key::Digit(3), "3"),
        (Key::Operator(Op::Add), "+"),
    ],
    &[(Key::Digit(0), "0"), (Key::Point, "."), (Key::Equals, "=")],
];
const PER_PAGE: usize = 6;

pub struct View {
    pub calculator: Calculator,
    directory: PathBuf,
    history: Option<usize>,
    notice: Option<String>,
}

impl View {
    pub fn open(directory: PathBuf) -> Self {
        let calculator = fs::read(directory.join("calculator.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Calculator>(&bytes).ok())
            .map(Calculator::restored)
            .unwrap_or_default();
        Self {
            calculator,
            directory,
            history: None,
            notice: None,
        }
    }

    fn draw_history(&self, canvas: &mut Canvas, width: i32, page: usize) {
        header(canvas, "History", "Back");
        if self.calculator.history.is_empty() {
            canvas.text(
                rect(width, 40, 299, 1184, 110),
                "No calculations yet",
                BODY.center(),
            );
            return;
        }
        for (index, record) in self
            .calculator
            .history
            .iter()
            .skip(page * PER_PAGE)
            .take(PER_PAGE)
            .enumerate()
        {
            let y = 202 + index as i32 * 174;
            canvas.card(rect(width, 40, y, 1184, 150));
            canvas.text(
                rect(width, 75, y + 24, 1114, 52),
                &record.expression,
                CAPTION,
            );
            let label = if record.rounded {
                format!("{} (rounded)", record.result)
            } else {
                record.result.clone()
            };
            canvas.text(rect(width, 75, y + 75, 1114, 61), &label, BODY.right());
        }
        let pages = self.calculator.history.len().div_ceil(PER_PAGE);
        if page > 0 {
            canvas.button(previous(width), "Previous", false, false);
        }
        if page + 1 < pages {
            canvas.button(next(width), "Next", false, false);
        }
        if pages > 1 {
            canvas.text(
                rect(width, 406, 1417, 452, 56),
                &format!("{} / {pages}", page + 1),
                CAPTION.center(),
            );
        }
    }

    fn draw_calculator(&self, canvas: &mut Canvas, width: i32) {
        header(canvas, "Calculator", "Done");
        canvas.card(rect(width, 40, 185, 1184, 355));
        canvas.text(
            rect(width, 80, 242, 1104, 66),
            &self.calculator.status(),
            CAPTION.right(),
        );
        let entry = self.calculator.entry();
        let size = if entry.len() > 14 { 76 } else { 110 };
        canvas.text(
            rect(width, 80, 329, 1104, 150),
            entry,
            TextStyle::new(size).bold().right(),
        );
        canvas.button(history_button(width), "History", false, false);
        canvas.button(copy_button(width), "Copy result", false, false);
        if let Some(notice) = &self.notice {
            canvas.text(rect(width, 40, 696, 1184, 51), notice, CAPTION.center());
        }
        for (row, keys) in KEYS.iter().enumerate() {
            for (column, (key, label)) in keys.iter().enumerate() {
                canvas.button(
                    key_rect(width, row, column),
                    label,
                    *key == Key::Equals,
                    false,
                );
            }
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = primitive_app::WIDTH as i32;
        if back_button(width).contains(x, y) {
            return self.back();
        }
        if self.history.is_some() {
            if previous(width).contains(x, y) {
                return self.page(-1);
            }
            if next(width).contains(x, y) {
                return self.page(1);
            }
            return Change::None;
        }
        if history_button(width).contains(x, y) {
            self.history = Some(0);
            return Change::Full;
        }
        if copy_button(width).contains(x, y) {
            self.notice = Some(self.copy());
            return Change::Partial;
        }
        for (row, keys) in KEYS.iter().enumerate() {
            for (column, &(key, _)) in keys.iter().enumerate() {
                if key_rect(width, row, column).contains(x, y) {
                    self.notice = None;
                    self.calculator.press(key);
                    if matches!(key, Key::Equals | Key::Operator(_)) && self.save().is_err() {
                        self.notice = Some("Could not save calculation history.".into());
                    }
                    return Change::Partial;
                }
            }
        }
        Change::None
    }

    fn copy(&self) -> String {
        let Some(value) = self.calculator.copy() else {
            return "There is no result to copy.".into();
        };
        let line = format!("{value}\n");
        if primitive_app::save(&self.directory.join("result.txt"), line.as_bytes()).is_err() {
            return "Could not copy the result.".into();
        }
        "Copied to result.txt".into()
    }

    fn back(&mut self) -> Change {
        if self.history.take().is_some() {
            Change::Full
        } else {
            Change::Exit
        }
    }

    fn page(&mut self, change: isize) -> Change {
        let Some(page) = self.history else {
            return Change::None;
        };
        let last = self
            .calculator
            .history
            .len()
            .div_ceil(PER_PAGE)
            .saturating_sub(1);
        let selected = page.saturating_add_signed(change).min(last);
        if selected == page {
            return Change::None;
        }
        self.history = Some(selected);
        Change::Partial
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let width = canvas.bounds().w;
        if let Some(page) = self.history {
            self.draw_history(canvas, width, page);
        } else {
            self.draw_calculator(canvas, width);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if self.history.is_some_and(|page| page > 0) => self.page(-1),
            Input::PageBack => self.back(),
            Input::PageForward => self.page(1),
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        primitive_app::save(
            &self.directory.join("calculator.json"),
            &serde_json::to_vec(&self.calculator)?,
        )
    }
}

fn rect(width: i32, x: i32, y: i32, w: i32, h: i32) -> Rect {
    screen_rect(width, x, y, w, h)
}

fn history_button(width: i32) -> Rect {
    rect(width, 40, 565, 570, 106)
}

fn copy_button(width: i32) -> Rect {
    rect(width, 654, 565, 570, 106)
}

fn key_rect(width: i32, row: usize, column: usize) -> Rect {
    let x = 40 + column as i32 * 300;
    let y = 768 + row as i32 * 167;
    let key_width = if row == 4 && column == 2 { 584 } else { 284 };
    rect(width, x, y, key_width, 150)
}

fn previous(width: i32) -> Rect {
    rect(width, 40, 1386, 350, 106)
}

fn next(width: i32) -> Rect {
    rect(width, 874, 1386, 350, 106)
}
