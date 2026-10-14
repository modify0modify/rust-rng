use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};

/// A small command-line random number generator.
#[derive(Debug, Parser)]
#[command(name = "rng", version, about, long_about = None)]
struct Args {
    /// How many numbers to print (minimum 1)
    #[arg(
        short = 'n',
        long = "count",
        default_value_t = 1,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    count: u64,

    /// Lower bound. Inclusive for integers and floats.
    #[arg(
        short = 'm',
        long = "min",
        default_value = "1",
        allow_hyphen_values = true
    )]
    min: String,

    /// Upper bound. Inclusive for integers; exclusive for floats.
    #[arg(
        short = 'M',
        long = "max",
        default_value = "100",
        allow_hyphen_values = true
    )]
    max: String,

    /// Sample f64 values from the half-open interval [min, max).
    #[arg(short = 'f', long = "float")]
    float: bool,

    /// Seed the generator so the same options always produce the same numbers.
    #[arg(short = 's', long = "seed")]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
    if let Err(err) = write_samples(&args, &mut io::stdout()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn write_samples(args: &Args, out: &mut impl Write) -> Result<(), String> {
    if args.float {
        let min = parse_f64_bound("min", &args.min)?;
        let max = parse_f64_bound("max", &args.max)?;
        try_for_each_float(min, max, args.count, args.seed, |value| {
            writeln!(out, "{value}").map_err(|err| err.to_string())
        })
    } else {
        let min = parse_i64_bound("min", &args.min)?;
        let max = parse_i64_bound("max", &args.max)?;
        try_for_each_int(min, max, args.count, args.seed, |value| {
            writeln!(out, "{value}").map_err(|err| err.to_string())
        })
    }
}

fn make_rng(seed: Option<u64>) -> StdRng {
    match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    }
}

fn check_count(count: u64) -> Result<(), String> {
    if count < 1 {
        Err("count must be at least 1".to_string())
    } else {
        Ok(())
    }
}

/// Integer bounds must be written as i64 literals. A value such as `1.5`
/// parses as a float and is rejected so integer mode stays integral.
fn parse_i64_bound(label: &str, raw: &str) -> Result<i64, String> {
    if let Ok(value) = raw.parse::<i64>() {
        return Ok(value);
    }
    if looks_fractional(raw) && raw.parse::<f64>().is_ok() {
        return Err(format!("fractional integer bound for {label}: '{raw}'"));
    }
    Err(format!("invalid integer bound for {label}: '{raw}'"))
}

fn looks_fractional(raw: &str) -> bool {
    raw.contains('.') || raw.contains('e') || raw.contains('E')
}

fn parse_f64_bound(label: &str, raw: &str) -> Result<f64, String> {
    let value: f64 = raw
        .parse()
        .map_err(|_| format!("invalid float bound for {label}: '{raw}'"))?;
    if !value.is_finite() {
        return Err(format!("non-finite float bound for {label}: '{raw}'"));
    }
    Ok(value)
}

fn try_for_each_int<F>(
    min: i64,
    max: i64,
    count: u64,
    seed: Option<u64>,
    mut emit: F,
) -> Result<(), String>
where
    F: FnMut(i64) -> Result<(), String>,
{
    check_count(count)?;
    if min > max {
        return Err(format!(
            "inverted range: min ({min}) is greater than max ({max})"
        ));
    }
    let mut rng = make_rng(seed);
    for _ in 0..count {
        emit(rng.gen_range(min..=max))?;
    }
    Ok(())
}

fn try_for_each_float<F>(
    min: f64,
    max: f64,
    count: u64,
    seed: Option<u64>,
    mut emit: F,
) -> Result<(), String>
where
    F: FnMut(f64) -> Result<(), String>,
{
    check_count(count)?;
    if min >= max {
        return Err(format!("empty float range: [{min}, {max})"));
    }
    let mut rng = make_rng(seed);
    for _ in 0..count {
        emit(rng.gen_range(min..max))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_ints(
        min: i64,
        max: i64,
        count: u64,
        seed: Option<u64>,
    ) -> Result<Vec<i64>, String> {
        let mut values = Vec::new();
        try_for_each_int(min, max, count, seed, |value| {
            values.push(value);
            Ok(())
        })?;
        Ok(values)
    }

    fn generate_floats(
        min: f64,
        max: f64,
        count: u64,
        seed: Option<u64>,
    ) -> Result<Vec<f64>, String> {
        let mut values = Vec::new();
        try_for_each_float(min, max, count, seed, |value| {
            values.push(value);
            Ok(())
        })?;
        Ok(values)
    }

    fn render(args: &Args) -> Result<String, String> {
        let mut buf = Vec::new();
        write_samples(args, &mut buf)?;
        String::from_utf8(buf).map_err(|err| err.to_string())
    }

    #[test]
    fn defaults() {
        let args = Args::try_parse_from(["rng"]).unwrap();
        assert_eq!(args.count, 1);
        assert_eq!(args.min, "1");
        assert_eq!(args.max, "100");
        assert!(!args.float);
        assert_eq!(args.seed, None);

        assert!(Args::try_parse_from(["rng", "-n", "0"]).is_err());
        assert!(Args::try_parse_from(["rng", "--count", "0"]).is_err());

        let rendered = render(&Args::try_parse_from(["rng", "-s", "1"]).unwrap()).unwrap();
        let value: i64 = rendered.trim_end().parse().unwrap();
        assert!((1..=100).contains(&value));
        assert_eq!(rendered.lines().count(), 1);
    }

    #[test]
    fn inverted_range() {
        let err = generate_ints(10, 1, 1, None).unwrap_err();
        assert!(err.contains("inverted range"), "{err}");

        let args = Args::try_parse_from(["rng", "-m", "10", "-M", "1"]).unwrap();
        let err = render(&args).unwrap_err();
        assert!(err.contains("inverted range"), "{err}");

        let equal = generate_ints(5, 5, 3, Some(1)).unwrap();
        assert_eq!(equal, vec![5, 5, 5]);
    }

    #[test]
    fn fractional_integer_bound() {
        let err = parse_i64_bound("min", "1.5").unwrap_err();
        assert!(err.contains("fractional integer bound"), "{err}");

        let err = parse_i64_bound("max", "2.0").unwrap_err();
        assert!(err.contains("fractional integer bound"), "{err}");

        let args = Args::try_parse_from(["rng", "-m", "0", "-M", "3.5"]).unwrap();
        let err = render(&args).unwrap_err();
        assert!(err.contains("fractional integer bound"), "{err}");

        let bounds = parse_f64_bound("min", "1.5").unwrap();
        assert_eq!(bounds, 1.5);
    }

    #[test]
    fn empty_float_range() {
        let err = generate_floats(2.0, 2.0, 1, Some(1)).unwrap_err();
        assert!(err.contains("empty float range"), "{err}");

        let err = generate_floats(3.0, 1.5, 1, Some(1)).unwrap_err();
        assert!(err.contains("empty float range"), "{err}");

        let args = Args::try_parse_from(["rng", "-f", "-m", "1", "-M", "1"]).unwrap();
        let err = render(&args).unwrap_err();
        assert!(err.contains("empty float range"), "{err}");

        assert!(generate_ints(2, 2, 1, Some(1)).is_ok());
    }

    #[test]
    fn seeded_ints() {
        let a = generate_ints(1, 100, 8, Some(42)).unwrap();
        let b = generate_ints(1, 100, 8, Some(42)).unwrap();
        let c = generate_ints(1, 100, 8, Some(43)).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.iter().all(|value| (1..=100).contains(value)));

        let args =
            Args::try_parse_from(["rng", "-s", "42", "-n", "8", "-m", "1", "-M", "100"]).unwrap();
        let rendered = render(&args).unwrap();
        let expected: String = a.iter().map(|value| format!("{value}\n")).collect();
        assert_eq!(rendered, expected);

        let mut saw_min = false;
        let mut saw_max = false;
        for seed in 0..64 {
            let value = generate_ints(0, 1, 1, Some(seed)).unwrap()[0];
            saw_min |= value == 0;
            saw_max |= value == 1;
        }
        assert!(
            saw_min && saw_max,
            "inclusive integer range must reach both ends"
        );
    }

    #[test]
    fn seeded_floats() {
        let a = generate_floats(0.0, 1.0, 8, Some(42)).unwrap();
        let b = generate_floats(0.0, 1.0, 8, Some(42)).unwrap();
        let c = generate_floats(0.0, 1.0, 8, Some(99)).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.iter().all(|value| (0.0..1.0).contains(value)));

        let args = Args::try_parse_from(["rng", "-f", "-s", "42", "-n", "8", "-m", "0", "-M", "1"])
            .unwrap();
        let rendered = render(&args).unwrap();
        let expected: String = a.iter().map(|value| format!("{value}\n")).collect();
        assert_eq!(rendered, expected);
        assert_eq!(rendered.lines().count(), 8);
    }

    #[test]
    fn full_i64_domain() {
        let a = generate_ints(i64::MIN, i64::MAX, 8, Some(99)).unwrap();
        let b = generate_ints(i64::MIN, i64::MAX, 8, Some(99)).unwrap();
        assert_eq!(a, b);
        assert!(
            a.iter()
                .any(|value| *value < i32::MIN as i64 || *value > i32::MAX as i64),
            "samples stayed inside i32, so the full i64 domain was not used: {a:?}"
        );

        let args = Args::try_parse_from([
            "rng",
            "-s",
            "99",
            "-n",
            "8",
            "-m",
            &i64::MIN.to_string(),
            "-M",
            &i64::MAX.to_string(),
        ])
        .unwrap();
        let rendered = render(&args).unwrap();
        let expected: String = a.iter().map(|value| format!("{value}\n")).collect();
        assert_eq!(rendered, expected);
    }
}
