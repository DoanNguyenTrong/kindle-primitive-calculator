use calculator_core::tools::{convert, simplify, solve, unit_names, Polynomial, GROUP_NAMES};
use primitive_app::Change;
use primitive_ui::{
    back_button, header, keyboard_key, screen_rect, Canvas, Editor, Input, Layout, Press, Rect,
    TextStyle, BLACK, BODY, CAPTION,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Menu,
    Convert,
    Steps,
    Graph,
}

#[derive(Clone, Copy)]
enum Picker {
    Group,
    From,
    To,
}

pub struct View {
    panel: Panel,
    fields: [Editor; 3],
    editing: Option<usize>,
    group: usize,
    from: usize,
    to: usize,
    picker: Option<Picker>,
    equation: bool,
    result: Option<String>,
    steps: Vec<String>,
    step: usize,
    graph: Option<Polynomial>,
    table: bool,
    zoom: usize,
    error: Option<&'static str>,
}

const SCALES: [i32; 6] = [2, 5, 10, 20, 50, 100];

impl Default for View {
    fn default() -> Self {
        Self {
            panel: Panel::Menu,
            fields: std::array::from_fn(|_| Editor::new(24)),
            editing: None,
            group: 0,
            from: 0,
            to: 1,
            picker: None,
            equation: false,
            result: None,
            steps: Vec::new(),
            step: 0,
            graph: None,
            table: false,
            zoom: 2,
            error: None,
        }
    }
}

impl View {
    fn select(&mut self, panel: Panel) -> Change {
        self.panel = panel;
        self.error = None;
        self.editing = None;
        self.picker = None;
        self.clear_result();
        self.fill_fields(match panel {
            Panel::Menu => ["", "", ""],
            Panel::Convert => ["1", "", ""],
            Panel::Steps => ["6", "8", ""],
            Panel::Graph => ["1", "0", "0"],
        });
        self.equation = false;
        self.table = false;
        self.zoom = 2;
        Change::Full
    }

    fn fill_fields(&mut self, values: [&str; 3]) {
        for (field, value) in self.fields.iter_mut().zip(values) {
            *field = Editor::new(24);
            field.text = value.chars().collect();
            field.caret = field.text.len();
        }
    }

    fn clear_result(&mut self) {
        self.result = None;
        self.steps.clear();
        self.step = 0;
        self.graph = None;
        self.error = None;
    }

    fn back(&mut self) -> Change {
        if self.editing.take().is_some() || self.picker.take().is_some() {
            Change::Full
        } else if self.panel == Panel::Menu {
            Change::Exit
        } else {
            self.select(Panel::Menu)
        }
    }

    fn calculate(&mut self) -> Change {
        let [a, b, c] = self.fields.each_ref().map(Editor::string);
        self.error = None;
        match self.panel {
            Panel::Menu => {}
            Panel::Convert => match convert(&a, self.group, self.from, self.to) {
                Ok(text) => self.result = Some(text),
                Err(error) => self.error = Some(error),
            },
            Panel::Steps => match if self.equation {
                solve(&a, &b, &c)
            } else {
                simplify(&a, &b)
            } {
                Ok(steps) => {
                    self.steps = steps;
                    self.step = 0;
                }
                Err(error) => self.error = Some(error),
            },
            Panel::Graph => {
                self.graph = Polynomial::parse(&a, &b, &c);
                if self.graph.is_none() {
                    self.error = Some("Enter numbers for a, b and c.");
                }
            }
        }
        Change::Partial
    }

    fn picker_names(&self, picker: Picker) -> Vec<&'static str> {
        match picker {
            Picker::Group => GROUP_NAMES.to_vec(),
            Picker::From | Picker::To => unit_names(self.group).collect(),
        }
    }

    fn handle_picker(&mut self, picker: Picker, x: f32, y: f32) -> Change {
        let w = primitive_app::WIDTH as i32;
        let Some(index) =
            (0..self.picker_names(picker).len()).find(|&index| pick_row(w, index).contains(x, y))
        else {
            return Change::None;
        };
        match picker {
            Picker::Group => {
                self.group = index;
                self.from = 0;
                self.to = 1;
            }
            Picker::From => self.from = index,
            Picker::To => self.to = index,
        }
        self.clear_result();
        self.picker = None;
        Change::Full
    }

    fn input_rows(&self) -> usize {
        match self.panel {
            Panel::Menu => 0,
            Panel::Convert => 1,
            Panel::Steps if !self.equation => 2,
            _ => 3,
        }
    }

    fn field_rect(&self, w: i32, i: usize) -> Rect {
        match self.panel {
            Panel::Convert => rect(w, 40, 415, 1184, 113),
            Panel::Steps => rect(w, 40, 304 + 123 * i as i32, 1184, 107),
            Panel::Graph => rect(w, 40 + 400 * i as i32, 260, 384, 107),
            Panel::Menu => rect(w, 40, 310, 1184, 108),
        }
    }

    pub fn input(&mut self, input: Input) -> Change {
        let (x, y) = match input {
            Input::Tap(x, y) => (x, y),
            Input::PageBack => return self.back(),
            _ => return Change::None,
        };
        let w = primitive_app::WIDTH as i32;
        if back_button(w).contains(x, y) {
            return self.back();
        }
        if let Some(picker) = self.picker {
            return self.handle_picker(picker, x, y);
        }
        if let Some(i) = self.editing {
            let layout = Layout::new(1264, 1680, 4, 5);
            if let Some(key) = keyboard_key(&layout, &self.fields[i], "Done", x, y) {
                return match self.fields[i].press(key) {
                    Press::Go => {
                        self.editing = None;
                        Change::Full
                    }
                    Press::Nothing => Change::None,
                    Press::Edited => {
                        self.clear_result();
                        Change::Partial
                    }
                    _ => Change::Partial,
                };
            }
        }
        if let Some(i) = (0..self.input_rows()).find(|&i| self.field_rect(w, i).contains(x, y)) {
            self.editing = Some(i);
            return Change::Full;
        }
        if self.editing.is_some() {
            return Change::None;
        }

        match self.panel {
            Panel::Menu => {
                for (i, panel) in [Panel::Convert, Panel::Steps, Panel::Graph]
                    .into_iter()
                    .enumerate()
                {
                    if menu_row(w, i).contains(x, y) {
                        return self.select(panel);
                    }
                }
            }
            Panel::Convert => {
                for (i, picker) in [Picker::Group, Picker::From, Picker::To]
                    .into_iter()
                    .enumerate()
                {
                    if convert_button(w, i).contains(x, y) {
                        self.picker = Some(picker);
                        return Change::Full;
                    }
                }
            }
            Panel::Steps => {
                if pill(w, 0).contains(x, y) || pill(w, 1).contains(x, y) {
                    let equation = pill(w, 1).contains(x, y);
                    if equation != self.equation {
                        self.equation = equation;
                        self.fill_fields(if equation {
                            ["2", "3", "11"]
                        } else {
                            ["6", "8", ""]
                        });
                        self.clear_result();
                        return Change::Full;
                    }
                }
                if !self.steps.is_empty() && step_button(w, false).contains(x, y) && self.step > 0 {
                    self.step -= 1;
                    return Change::Partial;
                }
                if step_button(w, true).contains(x, y) && self.step + 1 < self.steps.len() {
                    self.step += 1;
                    return Change::Partial;
                }
            }
            Panel::Graph => {
                if graph_toolbar(w, 1).contains(x, y) && self.graph.is_some() {
                    self.table = !self.table;
                    return Change::Partial;
                }
                if self.graph.is_some() && !self.table {
                    if graph_toolbar(w, 2).contains(x, y) {
                        self.zoom = self.zoom.saturating_sub(1);
                        return Change::Partial;
                    }
                    if graph_toolbar(w, 3).contains(x, y) {
                        self.zoom = (self.zoom + 1).min(SCALES.len() - 1);
                        return Change::Partial;
                    }
                }
            }
        }

        if self.panel != Panel::Menu && self.calculate_rect(w).contains(x, y) {
            return self.calculate();
        }
        Change::None
    }

    fn calculate_rect(&self, w: i32) -> Rect {
        match self.panel {
            Panel::Convert => rect(w, 40, 584, 1184, 108),
            Panel::Steps => rect(w, 40, 705, 1184, 108),
            Panel::Graph => graph_toolbar(w, 0),
            Panel::Menu => rect(w, 40, 580, 1184, 108),
        }
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        if let Some(picker) = self.picker {
            header(
                canvas,
                match picker {
                    Picker::Group => "Conversion category",
                    Picker::From => "Convert from",
                    Picker::To => "Convert to",
                },
                "Back",
            );
            for (i, name) in self.picker_names(picker).into_iter().enumerate() {
                canvas.button(pick_row(w, i), name, false, false);
            }
            return;
        }
        let title = match self.panel {
            Panel::Menu => "Calculator tools",
            Panel::Convert => "Convert",
            Panel::Steps => "Steps",
            Panel::Graph => "Graph",
        };
        header(canvas, title, "Back");

        if self.panel == Panel::Menu {
            for (i, (label, detail)) in [
                (
                    "Convert",
                    "Common units for length, mass, volume, time and temperature.",
                ),
                ("Steps", "Simplify a fraction or solve a linear equation."),
                ("Graph", "Plot a line or quadratic and inspect its values."),
            ]
            .into_iter()
            .enumerate()
            {
                let bounds = menu_row(w, i);
                canvas.card(bounds);
                canvas.text(
                    rect(w, 80, 228 + 200 * i as i32, 1104, 58),
                    label,
                    TextStyle::new(45).bold(),
                );
                canvas.text(rect(w, 80, 304 + 200 * i as i32, 1104, 60), detail, CAPTION);
            }
            return;
        }

        match self.panel {
            Panel::Convert => {
                let names: Vec<_> = unit_names(self.group).collect();
                for (i, label) in [
                    GROUP_NAMES[self.group].to_owned(),
                    format!("From {}", names[self.from]),
                    format!("To {}", names[self.to]),
                ]
                .iter()
                .enumerate()
                {
                    canvas.button(convert_button(w, i), label, false, false);
                }
            }
            Panel::Steps => {
                canvas.button(pill(w, 0), "Fraction", !self.equation, false);
                canvas.button(pill(w, 1), "Linear equation", self.equation, false);
                if self.equation {
                    canvas.text(
                        rect(w, 40, 269, 1184, 32),
                        "a × x + b = c",
                        CAPTION.center(),
                    );
                }
            }
            Panel::Graph => canvas.text(
                rect(w, 40, 154, 1184, 80),
                "y = a x² + b x + c",
                BODY.center(),
            ),
            Panel::Menu => {}
        }

        let labels = match self.panel {
            Panel::Convert => ["Value", "", ""],
            Panel::Steps if !self.equation => ["Numerator", "Denominator", ""],
            _ => ["a", "b", "c"],
        };
        for (i, label) in labels.into_iter().enumerate().take(self.input_rows()) {
            let bounds = self.field_rect(w, i);
            let value = self.fields[i].string();
            canvas.panel(bounds, 16, if self.editing == Some(i) { 6 } else { 3 }, 255);
            canvas.text(bounds.inset(20), &format!("{label}   {value}"), BODY);
        }

        if let Some(index) = self.editing {
            canvas.keyboard(self.fields[index].layer, "Done", None);
            return;
        }

        let button = match self.panel {
            Panel::Convert => "Convert",
            Panel::Steps => "Show steps",
            Panel::Graph => "Plot",
            Panel::Menu => "",
        };
        canvas.button(self.calculate_rect(w), button, true, false);
        if let Some(error) = self.error {
            canvas.text(rect(w, 40, 1530, 1184, 120), error, BODY);
        }
        match self.panel {
            Panel::Convert => {
                if let Some(result) = &self.result {
                    canvas.card(rect(w, 40, 759, 1184, 224));
                    canvas.text(
                        rect(w, 78, 812, 1108, 120),
                        result,
                        TextStyle::new(48).bold().center(),
                    );
                }
            }
            Panel::Steps => {
                if let Some(step) = self.steps.get(self.step) {
                    canvas.card(rect(w, 40, 895, 1184, 370));
                    canvas.text(
                        rect(w, 75, 932, 1114, 89),
                        &format!("Step {} of {}", self.step + 1, self.steps.len()),
                        CAPTION,
                    );
                    canvas.text(rect(w, 75, 1025, 1114, 194), step, TextStyle::new(46));
                    if self.step > 0 {
                        canvas.button(step_button(w, false), "Previous", false, false);
                    }
                    if self.step + 1 < self.steps.len() {
                        canvas.button(step_button(w, true), "Next step", false, false);
                    }
                }
            }
            Panel::Graph => {
                if let Some(graph) = self.graph {
                    canvas.button(
                        graph_toolbar(w, 1),
                        if self.table { "View graph" } else { "Table" },
                        false,
                        false,
                    );
                    let scale = SCALES[self.zoom];
                    if self.table {
                        draw_table(canvas, w, graph, scale);
                    } else {
                        canvas.button(graph_toolbar(w, 2), "Zoom in", false, false);
                        canvas.button(graph_toolbar(w, 3), "Zoom out", false, false);
                        draw_plot(canvas, w, graph, scale);
                    }
                }
            }
            Panel::Menu => {}
        }
    }
}

fn draw_table(canvas: &mut Canvas, w: i32, graph: Polynomial, scale: i32) {
    let half = scale / 2;
    let mut y = 570;
    for (left, right) in std::iter::once(("x".into(), "y".into())).chain(
        [-scale, -half, 0, half, scale]
            .into_iter()
            .map(|x| (x.to_string(), graph.sample(i128::from(x)))),
    ) {
        let bounds = rect(w, 40, y, 1184, 120);
        canvas.card(bounds);
        canvas.text(rect(w, 50, y + 8, 440, 104), &left, BODY.center());
        canvas.text(rect(w, 490, y + 8, 714, 104), &right, BODY.center());
        y += 133;
    }
}

fn draw_plot(canvas: &mut Canvas, w: i32, graph: Polynomial, scale: i32) {
    let plot = rect(w, 162, 566, 940, 940);
    canvas.panel(plot, 0, 3, 255);
    let mid = (plot.w - 1) / 2;
    canvas.fill(Rect::new(plot.x + mid, plot.y + 2, 2, plot.h - 4), BLACK);
    canvas.fill(Rect::new(plot.x + 2, plot.y + mid, plot.w - 4, 2), BLACK);
    let limit = f64::from(scale);
    let point = |column| {
        let x = 2.0 * limit * f64::from(column) / f64::from(plot.w - 1) - limit;
        graph.at(x)
    };
    let row = |y: f64| ((limit - y) * f64::from(plot.h - 1) / (2.0 * limit)) as i32;
    let mut previous = point(1);
    for col in 2..plot.w - 2 {
        let value = point(col);
        let high = previous.max(value);
        let low = previous.min(value);
        if high >= -limit && low <= limit {
            let top = row(high.min(limit)).clamp(2, plot.h - 3);
            let bottom = row(low.max(-limit)).clamp(top, plot.h - 3);
            canvas.fill(
                Rect::new(plot.x + col, plot.y + top, 2, bottom - top + 2),
                BLACK,
            );
        }
        previous = value;
    }
    canvas.text(
        rect(w, 40, 1530, 1184, 100),
        &format!("Both axes range from -{scale} to {scale}."),
        CAPTION.center(),
    );
}

fn rect(w: i32, x: i32, y: i32, width: i32, height: i32) -> Rect {
    screen_rect(w, x, y, width, height)
}
fn menu_row(w: i32, i: usize) -> Rect {
    rect(w, 40, 196 + 200 * i as i32, 1184, 168)
}
fn pick_row(w: i32, i: usize) -> Rect {
    rect(w, 40, 170 + 130 * i as i32, 1184, 110)
}
fn pill(w: i32, i: i32) -> Rect {
    rect(w, 40 + 600 * i, 160, 580, 108)
}
fn convert_button(w: i32, i: usize) -> Rect {
    rect(w, 40 + 400 * i as i32, 180, 384, 124)
}
fn step_button(w: i32, next: bool) -> Rect {
    rect(w, if next { 646 } else { 40 }, 1313, 578, 110)
}
fn graph_toolbar(w: i32, i: i32) -> Rect {
    rect(w, 40 + 300 * i, 403, 284, 106)
}
