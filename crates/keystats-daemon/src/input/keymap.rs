/// Map evdev key codes (u16) to stable display names.
/// These names match the macOS KeyStats naming convention where possible.
pub fn key_name(code: u16) -> String {
    match code {
        // Letters
        16 => "Q",
        17 => "W",
        18 => "E",
        19 => "R",
        20 => "T",
        21 => "Y",
        22 => "U",
        23 => "I",
        24 => "O",
        25 => "P",
        30 => "A",
        31 => "S",
        32 => "D",
        33 => "F",
        34 => "G",
        35 => "H",
        36 => "J",
        37 => "K",
        38 => "L",
        44 => "Z",
        45 => "X",
        46 => "C",
        47 => "V",
        48 => "B",
        49 => "N",
        50 => "M",

        // Numbers
        2 => "1",
        3 => "2",
        4 => "3",
        5 => "4",
        6 => "5",
        7 => "6",
        8 => "7",
        9 => "8",
        10 => "9",
        11 => "0",

        // Modifiers
        42 => "LeftShift",
        54 => "RightShift",
        29 => "LeftControl",
        97 => "RightControl",
        56 => "LeftAlt",
        100 => "RightAlt",
        125 => "LeftMeta",
        126 => "RightMeta",
        58 => "CapsLock",

        // Navigation
        103 => "Up",
        108 => "Down",
        105 => "Left",
        106 => "Right",
        104 => "PageUp",
        109 => "PageDown",
        102 => "Home",
        107 => "End",
        110 => "Insert",
        111 => "Delete",

        // Special keys
        1 => "Escape",
        28 => "Enter",
        57 => "Space",
        14 => "Backspace",
        15 => "Tab",
        59 => "F1",
        60 => "F2",
        61 => "F3",
        62 => "F4",
        63 => "F5",
        64 => "F6",
        65 => "F7",
        66 => "F8",
        67 => "F9",
        68 => "F10",
        87 => "F11",
        88 => "F12",
        69 => "NumLock",
        70 => "ScrollLock",

        // Numpad
        71 => "Num7",
        72 => "Num8",
        73 => "Num9",
        75 => "Num4",
        76 => "Num5",
        77 => "Num6",
        79 => "Num1",
        80 => "Num2",
        81 => "Num3",
        82 => "Num0",
        83 => "NumDot",
        96 => "NumEnter",
        74 => "NumMinus",
        78 => "NumPlus",
        55 => "NumStar",
        98 => "NumSlash",

        // Punctuation
        12 => "-",
        13 => "=",
        26 => "[",
        27 => "]",
        39 => ";",
        40 => "'",
        41 => "`",
        43 => "\\",
        51 => ",",
        52 => ".",
        53 => "/",

        _ => return format!("Key_{}", code),
    }
    .to_string()
}

/// Identify the button role for mouse button codes.
pub fn button_role(code: u16) -> Option<&'static str> {
    match code {
        0x110 => Some("left"),         // BTN_LEFT
        0x111 => Some("right"),        // BTN_RIGHT
        0x112 => Some("middle"),       // BTN_MIDDLE
        0x113 => Some("side_back"),    // BTN_SIDE
        0x114 => Some("side_forward"), // BTN_EXTRA
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_are_stable() {
        assert_eq!(key_name(30), "A");
        assert_eq!(key_name(57), "Space");
        assert_eq!(key_name(28), "Enter");
        assert_eq!(key_name(1), "Escape");
    }

    #[test]
    fn unknown_key_code_returns_key_number() {
        assert_eq!(key_name(999), "Key_999");
    }

    #[test]
    fn button_roles_are_correct() {
        assert_eq!(button_role(0x110), Some("left"));
        assert_eq!(button_role(0x111), Some("right"));
        assert_eq!(button_role(0x113), Some("side_back"));
        assert_eq!(button_role(0x114), Some("side_forward"));
    }

    #[test]
    fn non_mouse_button_is_none() {
        assert_eq!(button_role(30), None); // KEY_A
    }
}
