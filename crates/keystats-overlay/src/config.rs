/// Position of the overlay on screen.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Position {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Position {
    /// Parse from a CLI string argument.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "top-left" | "topleft" => Self::TopLeft,
            "top-right" | "topright" => Self::TopRight,
            "bottom-left" | "bottomleft" => Self::BottomLeft,
            "bottom-right" | "bottomright" => Self::BottomRight,
            _ => Self::TopLeft,
        }
    }
}

/// CLI configuration for the overlay.
#[derive(Debug, Clone)]
pub struct Config {
    /// Position on screen.
    pub position: Position,
    /// Maximum number of keys to display simultaneously.
    pub max_keys: usize,
    /// Fade-out duration in milliseconds after key release.
    pub fade_duration_ms: u64,
    /// Font size in pixels.
    pub font_size: u32,
    /// Margin as percentage of screen size (1-20).
    pub margin_percent: u32,
    /// Background opacity as percentage (0-100). 0 = fully transparent, 100 = fully opaque.
    pub opacity: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            position: Position::default(),
            max_keys: 10,
            fade_duration_ms: 800,
            font_size: 16,
            margin_percent: 5,
            opacity: 40,
        }
    }
}

impl Config {
    /// Parse configuration from command-line arguments.
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut config = Self::default();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--position" => {
                    if let Some(val) = args.get(i + 1) {
                        config.position = Position::parse(val);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--max-keys" => {
                    if let Some(val) = args.get(i + 1) {
                        config.max_keys = val.parse().unwrap_or(10).clamp(1, 50);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--fade-duration" => {
                    if let Some(val) = args.get(i + 1) {
                        config.fade_duration_ms = val.parse().unwrap_or(800).clamp(50, 5000);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--font-size" => {
                    if let Some(val) = args.get(i + 1) {
                        config.font_size = val.parse().unwrap_or(16).clamp(8, 72);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--margin" => {
                    if let Some(val) = args.get(i + 1) {
                        config.margin_percent = val.parse().unwrap_or(5).clamp(1, 20);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--opacity" => {
                    if let Some(val) = args.get(i + 1) {
                        config.opacity = val.parse().unwrap_or(40).clamp(0, 100);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    i += 1;
                }
            }
        }

        config
    }
}

fn print_help() {
    println!("Usage: keystats-overlay [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --position <POSITION>      Screen position: top-left, top-right, bottom-left, bottom-right (default: top-left)");
    println!("  --max-keys <N>             Maximum keys to display (default: 10)");
    println!("  --fade-duration <MS>       Fade-out duration in milliseconds (default: 800)");
    println!("  --font-size <PX>           Font size in pixels (default: 16)");
    println!("  --margin <PERCENT>         Margin as percentage of screen size, 1-20 (default: 5)");
    println!("  --opacity <PERCENT>        Background opacity, 0-100 (default: 40)");
    println!("  -h, --help                 Show this help");
}
