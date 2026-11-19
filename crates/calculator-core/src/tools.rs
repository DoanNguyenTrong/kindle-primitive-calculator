use std::fmt;

use crate::Decimal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Ratio {
    numerator: i128,
    denominator: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

impl Ratio {
    fn new(numerator: i128, denominator: i128) -> Self {
        let factor = gcd(numerator, denominator) * if denominator < 0 { -1 } else { 1 };
        Self {
            numerator: numerator / factor,
            denominator: denominator / factor,
        }
    }

    fn integer(n: i128) -> Self {
        Self::new(n, 1)
    }

    fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.len() > 24 {
            return None;
        }
        if let Some((top, bottom)) = text.split_once('/') {
            let n: i128 = top.trim().parse().ok()?;
            let d: i128 = bottom.trim().parse().ok()?;
            return (d != 0
                && n.unsigned_abs() <= 1_000_000_000
                && d.unsigned_abs() <= 1_000_000_000)
                .then(|| Self::new(n, d));
        }
        let padded;
        let text = if let Some(tail) = text.strip_prefix("-.") {
            padded = format!("-0.{tail}");
            &padded
        } else if text.starts_with('.') {
            padded = format!("0{text}");
            &padded
        } else {
            text
        };
        let value = Decimal::parse(text)?;
        (value.coefficient.unsigned_abs() <= 1_000_000_000_000 && value.scale <= 6)
            .then(|| Self::new(value.coefficient, 10_i128.pow(value.scale)))
    }

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.numerator * rhs.denominator + rhs.numerator * self.denominator,
            self.denominator * rhs.denominator,
        )
    }

    fn subtract(self, rhs: Self) -> Self {
        self.add(Self::new(-rhs.numerator, rhs.denominator))
    }

    fn multiply(self, rhs: Self) -> Self {
        Self::new(
            self.numerator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }

    fn divide(self, rhs: Self) -> Self {
        debug_assert_ne!(rhs.numerator, 0);
        self.multiply(Self::new(rhs.denominator, rhs.numerator))
    }

    fn real(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    fn decimal_text(self) -> String {
        // Keep the displayed rounding exact, including negative values and ties.
        let scale = 1_000_000_u128;
        let numerator = self.numerator.unsigned_abs();
        let denominator = self.denominator as u128;
        let scaled = numerator * scale;
        let approximation = !scaled.is_multiple_of(denominator);
        let rounded = (scaled + denominator / 2) / denominator;
        let fraction = format!("{:06}", rounded % scale);
        let fraction = fraction.trim_end_matches('0');
        let sign = if self.numerator < 0 && rounded != 0 {
            "-"
        } else {
            ""
        };
        let decimal = if fraction.is_empty() {
            String::new()
        } else {
            format!(".{fraction}")
        };
        let prefix = if approximation { "≈ " } else { "" };
        format!("{prefix}{sign}{}{decimal}", rounded / scale)
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(output, "{}", self.numerator)
        } else {
            write!(output, "{}/{}", self.numerator, self.denominator)
        }
    }
}

struct Unit {
    name: &'static str,
    factor: (i128, i128),
    offset: (i128, i128),
}

const fn unit(name: &'static str, num: i128, den: i128) -> Unit {
    Unit {
        name,
        factor: (num, den),
        offset: (0, 1),
    }
}

const LENGTH: &[Unit] = &[
    unit("Meters", 1, 1),
    unit("Centimeters", 1, 100),
    unit("Kilometers", 1000, 1),
    unit("Inches", 127, 5000),
    unit("Feet", 381, 1250),
    unit("Miles", 201168, 125),
];
const MASS: &[Unit] = &[
    unit("Kilograms", 1, 1),
    unit("Grams", 1, 1000),
    unit("Pounds", 45359237, 100000000),
    unit("Ounces", 45359237, 1600000000),
];
const VOLUME: &[Unit] = &[
    unit("Liters", 1, 1),
    unit("Milliliters", 1, 1000),
    unit("US gallons", 473176473, 125000000),
];
const TIME: &[Unit] = &[
    unit("Seconds", 1, 1),
    unit("Minutes", 60, 1),
    unit("Hours", 3600, 1),
    unit("Days", 86400, 1),
];
const TEMPERATURE: &[Unit] = &[
    unit("Celsius", 1, 1),
    Unit {
        name: "Fahrenheit",
        factor: (5, 9),
        offset: (-160, 9),
    },
    Unit {
        name: "Kelvin",
        factor: (1, 1),
        offset: (-27315, 100),
    },
];

pub const GROUP_NAMES: &[&str] = &["Length", "Mass", "Volume", "Time", "Temperature"];
const GROUPS: &[&[Unit]] = &[LENGTH, MASS, VOLUME, TIME, TEMPERATURE];

pub fn unit_names(group: usize) -> impl Iterator<Item = &'static str> {
    GROUPS
        .get(group)
        .into_iter()
        .flat_map(|units| units.iter().map(|unit| unit.name))
}

pub fn convert(value: &str, group: usize, from: usize, to: usize) -> Result<String, &'static str> {
    let value = Ratio::parse(value).ok_or("Enter a decimal or a fraction.")?;
    let group = GROUPS.get(group).ok_or("Choose a category.")?;
    let from = group.get(from).ok_or("Choose a source unit.")?;
    let to = group.get(to).ok_or("Choose a destination unit.")?;
    let ratio = |(n, d)| Ratio::new(n, d);
    let base = value.multiply(ratio(from.factor)).add(ratio(from.offset));
    let result = base.subtract(ratio(to.offset)).divide(ratio(to.factor));
    Ok(format!(
        "{} {}",
        result.decimal_text(),
        to.name.to_lowercase()
    ))
}

pub fn simplify(numerator: &str, denominator: &str) -> Result<Vec<String>, &'static str> {
    let integer = |text: &str| {
        text.trim()
            .parse::<i128>()
            .ok()
            .filter(|n| n.unsigned_abs() <= 1_000_000_000)
    };
    let a = integer(numerator).ok_or("Enter a whole number for the numerator.")?;
    let b = integer(denominator)
        .filter(|n| *n != 0)
        .ok_or("Enter a nonzero whole number for the denominator.")?;
    let divisor = gcd(a, b);
    let mut steps = vec![format!(
        "Start with {a}/{b}. The greatest common factor is {divisor}."
    )];
    if divisor > 1 {
        steps.push(format!(
            "Divide both parts by {divisor} to get {}/{}.",
            a / divisor,
            b / divisor
        ));
    }
    if b < 0 {
        steps.push("Move the minus sign out of the denominator.".into());
    }
    steps.push(format!("Simplified result: {}", Ratio::new(a, b)));
    Ok(steps)
}

pub fn solve(a: &str, b: &str, c: &str) -> Result<Vec<String>, &'static str> {
    let (a, b, c) = (Ratio::parse(a), Ratio::parse(b), Ratio::parse(c));
    let (Some(a), Some(b), Some(c)) = (a, b, c) else {
        return Err("Enter a number for a, b and c.");
    };
    let difference = c.subtract(b);
    let mut steps = vec![format!(
        "Subtract {b} from both sides. This gives ({a})x = {difference}."
    )];
    if a.numerator == 0 {
        steps.push(
            if difference.numerator == 0 {
                "Both sides are zero. Every value of x is a solution."
            } else {
                "Zero cannot equal a nonzero number. There is no solution."
            }
            .into(),
        );
    } else {
        steps.push(format!(
            "Divide both sides by {a}. x = {}",
            difference.divide(a)
        ));
    }
    Ok(steps)
}

#[derive(Clone, Copy)]
pub struct Polynomial {
    a: Ratio,
    b: Ratio,
    c: Ratio,
}

impl Polynomial {
    pub fn parse(a: &str, b: &str, c: &str) -> Option<Self> {
        Some(Self {
            a: Ratio::parse(a)?,
            b: Ratio::parse(b)?,
            c: Ratio::parse(c)?,
        })
    }

    pub fn at(self, x: f64) -> f64 {
        self.a
            .real()
            .mul_add(x, self.b.real())
            .mul_add(x, self.c.real())
    }

    pub fn sample(self, x: i128) -> String {
        let x = Ratio::integer(x);
        self.a
            .multiply(x)
            .add(self.b)
            .multiply(x)
            .add(self.c)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_affine_and_linear_units() {
        assert_eq!(convert("32", 4, 1, 0), Ok("0 celsius".into()));
        assert_eq!(convert("0", 4, 0, 2), Ok("273.15 kelvin".into()));
        assert_eq!(convert("1/4", 0, 0, 1), Ok("25 centimeters".into()));
        assert_eq!(convert("1", 0, 0, 3), Ok("≈ 39.370079 inches".into()));
        assert!(convert("1/0", 0, 0, 1).is_err());
    }

    #[test]
    fn explains_signs_and_degenerate_equations() {
        assert_eq!(
            simplify("12", "-18").unwrap().last().unwrap(),
            "Simplified result: -2/3"
        );
        let steps = solve("1/2", "3", "5").unwrap();
        assert!(steps[1].ends_with("x = 4"));
        assert!(solve("0", "1", "1").unwrap()[1].contains("Every"));
        assert!(solve("0", "1", "2").unwrap()[1].contains("no solution"));
    }

    #[test]
    fn evaluates_lines_and_quadratics() {
        let graph = Polynomial::parse("1/2", "-2", "1").unwrap();
        assert_eq!(graph.sample(3), "-1/2");
        assert!((graph.at(3.) + 0.5).abs() < f64::EPSILON);
        assert_eq!(Polynomial::parse("0", "4", "1").unwrap().sample(-2), "-7");
    }
}
