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
        139 => "Menu",
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
        99 => "PrtSc",      // KEY_SYSRQ
        210 => "PrtSc",     // KEY_PRINT (ACPI Print)
        119 => "Pause",
        127 => "Compose",

        // Media keys (common on laptops)
        113 => "Mute",
        114 => "VolDown",
        115 => "VolUp",
        116 => "Power",
        142 => "Sleep",
        143 => "WakeUp",
        150 => "WWW",
        155 => "Mail",
        156 => "Bookmarks",
        157 => "Computer",
        158 => "Back",
        159 => "Forward",
        163 => "NextSong",
        164 => "PlayPause",
        165 => "PrevSong",
        166 => "StopCD",
        167 => "Record",
        168 => "Rewind",
        171 => "Config",
        172 => "HomePage",
        173 => "Refresh",
        174 => "Exit",

        // Function keys (F13-F24)
        183 => "F13",
        184 => "F14",
        185 => "F15",
        186 => "F16",
        187 => "F17",
        188 => "F18",
        189 => "F19",
        190 => "F20",
        191 => "F21",
        192 => "F22",
        193 => "F23",
        194 => "F24",

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

/// Modifier key evdev codes (Shift, Ctrl, Alt, Meta).
#[allow(dead_code)]
const MODIFIER_CODES: [u16; 8] = [
    42,  // LeftShift
    54,  // RightShift
    29,  // LeftControl
    97,  // RightControl
    56,  // LeftAlt
    100, // RightAlt
    125, // LeftMeta
    126, // RightMeta
];

/// Shift key evdev codes.
const SHIFT_CODES: [u16; 2] = [42, 54];

/// Control key evdev codes.
const CTRL_CODES: [u16; 2] = [29, 97];

/// Alt key evdev codes.
const ALT_CODES: [u16; 2] = [56, 100];

/// Meta/Super key evdev codes.
const META_CODES: [u16; 2] = [125, 126];

/// Returns `true` if the key code is a modifier key.
#[allow(dead_code)]
pub fn is_modifier(code: u16) -> bool {
    MODIFIER_CODES.contains(&code)
}

/// Returns `true` if the key code is a Shift key.
pub fn is_shift(code: u16) -> bool {
    SHIFT_CODES.contains(&code)
}

/// Returns `true` if the key code is a Control key.
pub fn is_ctrl(code: u16) -> bool {
    CTRL_CODES.contains(&code)
}

/// Returns `true` if the key code is an Alt key.
pub fn is_alt(code: u16) -> bool {
    ALT_CODES.contains(&code)
}

/// Returns `true` if the key code is a Meta/Super key.
pub fn is_meta(code: u16) -> bool {
    META_CODES.contains(&code)
}

/// Map evdev key codes to their Shift-modified display names.
///
/// Returns `None` if the key has no shifted variant.
pub fn shifted_key_name(code: u16) -> Option<&'static str> {
    match code {
        // Numbers → symbols
        2 => Some("!"),
        3 => Some("@"),
        4 => Some("#"),
        5 => Some("$"),
        6 => Some("%"),
        7 => Some("^"),
        8 => Some("&"),
        9 => Some("*"),
        10 => Some("("),
        11 => Some(")"),
        // Punctuation shifted
        12 => Some("_"),  // -
        13 => Some("+"),  // =
        26 => Some("{"),  // [
        27 => Some("}"),  // ]
        39 => Some(":"),  // ;
        40 => Some("\""), // '
        41 => Some("~"),  // `
        43 => Some("|"),  // \
        51 => Some("<"),  // ,
        52 => Some(">"),  // .
        53 => Some("?"),  // /
        _ => None,
    }
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
        // PrtSc variants
        assert_eq!(key_name(99), "PrtSc");   // KEY_SYSRQ
        assert_eq!(key_name(210), "PrtSc");  // KEY_PRINT
        // Media keys
        assert_eq!(key_name(113), "Mute");
        assert_eq!(key_name(114), "VolDown");
        assert_eq!(key_name(115), "VolUp");
        assert_eq!(key_name(164), "PlayPause");
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
