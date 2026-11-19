pub mod tools;

use std::fmt;

use serde::{Deserialize, Serialize};

const LIMIT: i128 = 1_000_000_000_000_000_000;
const FRACTION_LIMIT: u32 = 12;
const HISTORY_LIMIT: usize = 100;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Op {
    pub fn label(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "×",
            Self::Divide => "÷",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Digit(u8),
    Point,
    Negate,
    Backspace,
    Clear,
    Operator(Op),
    Equals,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Decimal {
    coefficient: i128,
    scale: u32,
}

impl Decimal {
    fn new(mut coefficient: i128, mut scale: u32) -> Result<Self, &'static str> {
        while scale > 0 && coefficient % 10 == 0 {
            coefficient /= 10;
            scale -= 1;
        }
        if coefficient.unsigned_abs() >= LIMIT.cast_unsigned() {
            return Err("Number is too large.");
        }
        if scale > FRACTION_LIMIT {
            return Err("Result has too many decimal places.");
        }
        Ok(Self { coefficient, scale })
    }

    fn parse(text: &str) -> Option<Self> {
        let (integer, fraction) = text.split_once('.').unwrap_or((text, ""));
        let digits = integer.strip_prefix('-').unwrap_or(integer);
        if digits.is_empty()
            || !digits.bytes().all(|ch| ch.is_ascii_digit())
            || !fraction.bytes().all(|ch| ch.is_ascii_digit())
        {
            return None;
        }
        let coefficient = text.replace('.', "").parse().ok()?;
        Self::new(coefficient, fraction.len() as u32).ok()
    }

    fn calculate(self, op: Op, right: Self) -> Result<(Self, bool), &'static str> {
        let common_scale = self.scale.max(right.scale);
        let result = match op {
            Op::Add | Op::Subtract => {
                let left = self.coefficient * 10_i128.pow(common_scale - self.scale);
                let right = right.coefficient * 10_i128.pow(common_scale - right.scale);
                let sum = if op == Op::Add {
                    left + right
                } else {
                    left - right
                };
                Self::new(sum, common_scale)
            }
            Op::Multiply => Self::new(
                self.coefficient * right.coefficient,
                self.scale + right.scale,
            ),
            Op::Divide => return self.divide(right),
        }?;
        Ok((result, false))
    }

    fn divide(self, right: Self) -> Result<(Self, bool), &'static str> {
        let dividend = self.coefficient.abs() * 10_i128.pow(right.scale);
        let divisor = right.coefficient.abs() * 10_i128.pow(self.scale);
        if divisor == 0 {
            return Err("Cannot divide by zero.");
        }
        let mut digits = dividend / divisor;
        let mut remaining = dividend % divisor;
        let mut scale = 0;
        while remaining != 0 && scale < FRACTION_LIMIT && digits < LIMIT / 10 {
            remaining *= 10;
            digits = digits * 10 + remaining / divisor;
            remaining %= divisor;
            scale += 1;
        }
        let rounded = remaining != 0;
        if remaining * 2 >= divisor {
            digits += 1;
        }
        if (self.coefficient < 0) != (right.coefficient < 0) {
            digits = -digits;
        }
        Ok((Self::new(digits, scale)?, rounded))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale == 0 {
            return write!(output, "{}", self.coefficient);
        }
        let digits = format!(
            "{:0>width$}",
            self.coefficient.abs(),
            width = self.scale as usize + 1
        );
        let point = digits.len() - self.scale as usize;
        let (integer, fraction) = digits.split_at(point);
        let negative = if self.coefficient < 0 { "-" } else { "" };
        write!(output, "{negative}{integer}.{fraction}")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct History {
    pub expression: String,
    pub result: String,
    pub rounded: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Calculator {
    entry: String,
    pending: Option<(Decimal, Op)>,
    repeat: Option<(Op, Decimal)>,
    fresh: bool,
    #[serde(skip)]
    error: Option<&'static str>,
    rounded: bool,
    pub history: Vec<History>,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            entry: "0".into(),
            pending: None,
            repeat: None,
            fresh: false,
            error: None,
            rounded: false,
            history: Vec::new(),
        }
    }
}

impl Calculator {
    pub fn restored(mut self) -> Self {
        self.history.truncate(HISTORY_LIMIT);
        let valid = |value: Decimal| {
            value.scale <= FRACTION_LIMIT && value.coefficient.unsigned_abs() < LIMIT as u128
        };
        if self.entry.len() > 20
            || Decimal::parse(&self.entry).is_none()
            || self.pending.is_some_and(|(value, _)| !valid(value))
            || self.repeat.is_some_and(|(_, value)| !valid(value))
        {
            self.press(Key::Clear);
        }
        self
    }

    pub fn entry(&self) -> &str {
        &self.entry
    }

    pub fn status(&self) -> String {
        self.error
            .map(str::to_owned)
            .or_else(|| {
                self.pending
                    .map(|(left, operation)| format!("{left} {}", operation.label()))
            })
            .unwrap_or_else(|| {
                if self.rounded {
                    "Rounded result".into()
                } else {
                    String::new()
                }
            })
    }

    pub fn copy(&self) -> Option<String> {
        if self.error.is_some() {
            return None;
        }
        Decimal::parse(&self.entry).map(|value| value.to_string())
    }

    pub fn press(&mut self, key: Key) {
        if matches!(key, Key::Clear) {
            let history = std::mem::take(&mut self.history);
            *self = Self::default();
            self.history = history;
            return;
        }
        if self.error.is_some() {
            if matches!(key, Key::Digit(_) | Key::Point) {
                self.press(Key::Clear);
            } else {
                return;
            }
        }
        match key {
            Key::Digit(digit) if digit <= 9 => self.edit_digit(digit),
            Key::Point => {
                self.begin_entry();
                if !self.entry.contains('.') {
                    self.entry.push('.');
                }
            }
            Key::Negate => {
                self.consume_unset_operand();
                if self.entry.starts_with('-') {
                    self.entry.remove(0);
                } else {
                    self.entry.insert(0, '-');
                }
                self.rounded = false;
                self.fresh = false;
            }
            Key::Backspace => {
                self.consume_unset_operand();
                self.entry.pop();
                if self.entry.is_empty() || self.entry == "-" {
                    self.entry = "0".into();
                }
                self.fresh = false;
                self.rounded = false;
            }
            Key::Operator(op) => self.operator(op),
            Key::Equals => self.equals(),
            Key::Digit(_) | Key::Clear => {}
        }
    }

    fn edit_digit(&mut self, digit: u8) {
        self.begin_entry();
        if matches!(self.entry.as_str(), "0" | "-0") {
            self.entry.pop();
        }
        let ch = char::from(b'0' + digit);
        self.entry.push(ch);
        if Decimal::parse(&self.entry).is_none() || self.entry.len() > 20 {
            self.entry.pop();
        }
    }

    fn begin_entry(&mut self) {
        if self.fresh {
            self.entry = "0".into();
            self.fresh = false;
        }
        self.repeat = None;
        self.rounded = false;
    }

    fn consume_unset_operand(&mut self) {
        if self.pending.is_some() && self.fresh {
            self.entry = "0".into();
        }
        self.repeat = None;
    }

    fn operator(&mut self, op: Op) {
        if let Some((left, previous)) = self.pending {
            if self.fresh {
                self.pending = Some((left, op));
                return;
            }
            if let Some(right) = Decimal::parse(&self.entry) {
                self.evaluate(left, previous, right);
            }
        }
        if self.error.is_none() {
            self.pending = Decimal::parse(&self.entry).map(|value| (value, op));
            self.fresh = true;
            self.repeat = None;
        }
    }

    fn equals(&mut self) {
        let Some(current) = Decimal::parse(&self.entry) else {
            return;
        };
        if let Some((left, op)) = self.pending.take() {
            self.evaluate(left, op, current);
        } else if let Some((op, right)) = self.repeat {
            self.evaluate(current, op, right);
        }
    }

    fn evaluate(&mut self, left: Decimal, op: Op, right: Decimal) {
        match left.calculate(op, right) {
            Ok((value, rounded)) => {
                self.entry = value.to_string();
                self.repeat = Some((op, right));
                self.fresh = true;
                self.rounded = rounded;
                self.history.insert(
                    0,
                    History {
                        expression: format!("{left} {} {right}", op.label()),
                        result: self.entry.clone(),
                        rounded,
                    },
                );
                self.history.truncate(HISTORY_LIMIT);
            }
            Err(error) => {
                self.error = Some(error);
                self.fresh = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(left: &str, op: Op, right: &str) -> (String, bool) {
        let (value, rounded) = Decimal::parse(left)
            .unwrap()
            .calculate(op, Decimal::parse(right).unwrap())
            .unwrap();
        (value.to_string(), rounded)
    }

    #[test]
    fn fractional_operations_are_exact_when_they_fit() {
        assert_eq!(evaluate("0.1", Op::Add, "0.2"), ("0.3".into(), false));
        assert_eq!(evaluate("1.4", Op::Subtract, "2.3"), ("-0.9".into(), false));
        assert_eq!(
            evaluate("-0.025", Op::Multiply, "0.2"),
            ("-0.005".into(), false)
        );
        assert_eq!(evaluate("1", Op::Divide, "8"), ("0.125".into(), false));
    }

    #[test]
    fn nonterminating_division_is_rounded_and_labelled() {
        assert_eq!(
            evaluate("2", Op::Divide, "3"),
            ("0.666666666667".into(), true)
        );
        assert_eq!(
            evaluate("-1", Op::Divide, "6"),
            ("-0.166666666667".into(), true)
        );
    }

    #[test]
    fn repeated_equals_and_clear_history() {
        let mut calculator = Calculator::default();
        for key in [
            Key::Digit(5),
            Key::Operator(Op::Add),
            Key::Digit(2),
            Key::Equals,
            Key::Equals,
        ] {
            calculator.press(key);
        }
        assert_eq!(calculator.entry(), "9");
        assert_eq!(calculator.history[0].expression, "7 + 2");
        assert_eq!(calculator.copy().as_deref(), Some("9"));
        calculator.press(Key::Clear);
        assert_eq!(calculator.entry(), "0");
        assert_eq!(calculator.history.len(), 2);
    }

    #[test]
    fn mistakes_recover_without_losing_calculation_history() {
        let mut calculator = Calculator::default();
        for key in [
            Key::Digit(2),
            Key::Operator(Op::Divide),
            Key::Digit(0),
            Key::Equals,
        ] {
            calculator.press(key);
        }
        assert_eq!(calculator.status(), "Cannot divide by zero.");
        assert_eq!(calculator.copy(), None);
        calculator.press(Key::Digit(4));
        calculator.press(Key::Negate);
        calculator.press(Key::Backspace);
        assert_eq!(calculator.entry(), "0");
    }

    #[test]
    fn pending_operator_can_be_corrected_and_chaining_is_immediate() {
        let mut calculator = Calculator::default();
        for key in [
            Key::Digit(2),
            Key::Operator(Op::Subtract),
            Key::Operator(Op::Add),
            Key::Digit(3),
            Key::Operator(Op::Multiply),
            Key::Digit(4),
            Key::Equals,
        ] {
            calculator.press(key);
        }
        assert_eq!(calculator.entry(), "20");
        assert_eq!(calculator.history[1].result, "5");
    }
}
