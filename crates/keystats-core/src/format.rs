/// Compact number formatting (K/M suffixes) matching macOS KeyStats behavior.
pub fn format_count(n: u64) -> String {
    match n {
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1_000_000.0),
        n if n >= 1_000 => format!("{:.1}K", n as f64 / 1_000.0),
        _ => n.to_string(),
    }
}

pub fn format_distance(d: f64) -> String {
    if d >= 1000.0 {
        format!("{:.1}K", d / 1000.0)
    } else {
        format!("{:.0}", d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_small_numbers() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
    }

    #[test]
    fn format_thousands() {
        assert_eq!(format_count(1_000), "1.0K");
        assert_eq!(format_count(1_500), "1.5K");
        assert_eq!(format_count(9_999), "10.0K");
    }

    #[test]
    fn format_millions() {
        assert_eq!(format_count(1_000_000), "1.0M");
        assert_eq!(format_count(1_500_000), "1.5M");
    }

    #[test]
    fn format_distance_basic() {
        assert_eq!(format_distance(0.0), "0");
        assert_eq!(format_distance(42.0), "42");
        assert_eq!(format_distance(1500.0), "1.5K");
    }
}
