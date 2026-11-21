import QtCore
import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import QtQuick.Window
import org.kde.kirigami as Kirigami
import org.kde.plasma.plasma5support as Plasma5Support

Window {
    id: win

    property string rangeMode: "day" // "day" | "7d" | "30d"
    property string layoutMode: "ansi_us"

    Settings {
        id: heatmapSettings
        category: "KeyboardHeatmap"
        property alias layoutMode: win.layoutMode
    }

    property bool isoEnterHovered: false // "ansi_us" | "iso_uk" | "iso_de" | "iso_fr"
    property var availableDates: []
    property int dateIndex: 0
    property string selectedDate: ""
    property var keyTotals: ({
    })
    property var keyDetails: ({
    })
    property int totalKeys: 0
    property int activeKeys: 0
    property int maxKeyCount: 1
    property string hoveredSummary: "Hover over any keycap to inspect exact counts and combos"
    readonly property string stateDbPath: StandardPaths.writableLocation(StandardPaths.GenericStateLocation).toString().replace(/^file:\/\//, "") + "/keystats/stats.sqlite3"
    readonly property string datesCmd: "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT DISTINCT date FROM key_counts ORDER BY date ASC;\""
    readonly property string countsCmd: {
        if (rangeMode === "7d")
            return "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT key_name, SUM(count) FROM key_counts WHERE date >= date('now', 'localtime', '-6 days') GROUP BY key_name;\"";

        if (rangeMode === "30d")
            return "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT key_name, SUM(count) FROM key_counts WHERE date >= date('now', 'localtime', '-29 days') GROUP BY key_name;\"";

        var d = selectedDate !== "" ? selectedDate : "now";
        var whereClause = (d === "now") ? "date = date('now', 'localtime')" : ("date = '" + d + "'");
        return "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT key_name, SUM(count) FROM key_counts WHERE " + whereClause + " GROUP BY key_name;\"";
    }
    readonly property bool isIsoLayout: layoutMode !== "ansi_us"
    readonly property var keyboardRows: {
        var row0 = [{
            "id": "Escape",
            "top": "",
            "main": "esc",
            "w": 1.25
        }, {
            "id": "F1",
            "top": "",
            "main": "F1",
            "w": 1
        }, {
            "id": "F2",
            "top": "",
            "main": "F2",
            "w": 1
        }, {
            "id": "F3",
            "top": "",
            "main": "F3",
            "w": 1
        }, {
            "id": "F4",
            "top": "",
            "main": "F4",
            "w": 1
        }, {
            "id": "F5",
            "top": "",
            "main": "F5",
            "w": 1
        }, {
            "id": "F6",
            "top": "",
            "main": "F6",
            "w": 1
        }, {
            "id": "F7",
            "top": "",
            "main": "F7",
            "w": 1
        }, {
            "id": "F8",
            "top": "",
            "main": "F8",
            "w": 1
        }, {
            "id": "F9",
            "top": "",
            "main": "F9",
            "w": 1
        }, {
            "id": "F10",
            "top": "",
            "main": "F10",
            "w": 1
        }, {
            "id": "F11",
            "top": "",
            "main": "F11",
            "w": 1
        }, {
            "id": "F12",
            "top": "",
            "main": "F12",
            "w": 1
        }, {
            "id": "Delete",
            "top": "",
            "main": "del",
            "w": 1.75
        }];
        var row5 = [{
            "id": "LeftControl",
            "top": "",
            "main": "ctrl",
            "w": 1.25
        }, {
            "id": "LeftMeta",
            "top": "",
            "main": "super",
            "w": 1.25
        }, {
            "id": "LeftAlt",
            "top": "",
            "main": "alt",
            "w": 1.25
        }, {
            "id": "Space",
            "top": "",
            "main": "",
            "w": 6.25
        }, {
            "id": "RightAlt",
            "top": "",
            "main": layoutMode === "iso_de" || layoutMode === "iso_fr" ? "alt gr" : "alt",
            "w": 1
        }, {
            "id": "RightControl",
            "top": "",
            "main": "ctrl",
            "w": 1
        }, {
            "id": "Left",
            "top": "",
            "main": "◀",
            "w": 1
        }, {
            "id": "UpDown",
            "top": "",
            "main": "",
            "w": 1,
            "splitUpDown": true
        }, {
            "id": "Right",
            "top": "",
            "main": "▶",
            "w": 1
        }];
        if (layoutMode === "iso_uk")
            return [row0, [{
                "id": "`",
                "top": "¬",
                "main": "`",
                "w": 1
            }, {
                "id": "1",
                "top": "!",
                "main": "1",
                "w": 1
            }, {
                "id": "2",
                "top": "\"",
                "main": "2",
                "w": 1
            }, {
                "id": "3",
                "top": "£",
                "main": "3",
                "w": 1
            }, {
                "id": "4",
                "top": "$",
                "main": "4",
                "w": 1
            }, {
                "id": "5",
                "top": "%",
                "main": "5",
                "w": 1
            }, {
                "id": "6",
                "top": "^",
                "main": "6",
                "w": 1
            }, {
                "id": "7",
                "top": "&",
                "main": "7",
                "w": 1
            }, {
                "id": "8",
                "top": "*",
                "main": "8",
                "w": 1
            }, {
                "id": "9",
                "top": "(",
                "main": "9",
                "w": 1
            }, {
                "id": "0",
                "top": ")",
                "main": "0",
                "w": 1
            }, {
                "id": "-",
                "top": "_",
                "main": "-",
                "w": 1
            }, {
                "id": "=",
                "top": "+",
                "main": "=",
                "w": 1
            }, {
                "id": "Backspace",
                "top": "",
                "main": "backspace",
                "w": 2
            }], [{
                "id": "Tab",
                "top": "",
                "main": "tab",
                "w": 1.5
            }, {
                "id": "Q",
                "top": "",
                "main": "Q",
                "w": 1
            }, {
                "id": "W",
                "top": "",
                "main": "W",
                "w": 1
            }, {
                "id": "E",
                "top": "",
                "main": "E",
                "w": 1
            }, {
                "id": "R",
                "top": "",
                "main": "R",
                "w": 1
            }, {
                "id": "T",
                "top": "",
                "main": "T",
                "w": 1
            }, {
                "id": "Y",
                "top": "",
                "main": "Y",
                "w": 1
            }, {
                "id": "U",
                "top": "",
                "main": "U",
                "w": 1
            }, {
                "id": "I",
                "top": "",
                "main": "I",
                "w": 1
            }, {
                "id": "O",
                "top": "",
                "main": "O",
                "w": 1
            }, {
                "id": "P",
                "top": "",
                "main": "P",
                "w": 1
            }, {
                "id": "[",
                "top": "{",
                "main": "[",
                "w": 1
            }, {
                "id": "]",
                "top": "}",
                "main": "]",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "return",
                "w": 1.5,
                "isoEnterTop": true
            }], [{
                "id": "CapsLock",
                "top": "",
                "main": "caps lock",
                "w": 1.75
            }, {
                "id": "A",
                "top": "",
                "main": "A",
                "w": 1
            }, {
                "id": "S",
                "top": "",
                "main": "S",
                "w": 1
            }, {
                "id": "D",
                "top": "",
                "main": "D",
                "w": 1
            }, {
                "id": "F",
                "top": "",
                "main": "F",
                "w": 1
            }, {
                "id": "G",
                "top": "",
                "main": "G",
                "w": 1
            }, {
                "id": "H",
                "top": "",
                "main": "H",
                "w": 1
            }, {
                "id": "J",
                "top": "",
                "main": "J",
                "w": 1
            }, {
                "id": "K",
                "top": "",
                "main": "K",
                "w": 1
            }, {
                "id": "L",
                "top": "",
                "main": "L",
                "w": 1
            }, {
                "id": ";",
                "top": ":",
                "main": ";",
                "w": 1
            }, {
                "id": "'",
                "top": "@",
                "main": "'",
                "w": 1
            }, {
                "id": "\\",
                "top": "~",
                "main": "#",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "",
                "w": 1.25,
                "isoEnterBottom": true
            }], [{
                "id": "LeftShift",
                "top": "",
                "main": "shift",
                "w": 1.25
            }, {
                "id": "Key_86",
                "top": "|",
                "main": "\\",
                "w": 1
            }, {
                "id": "Z",
                "top": "",
                "main": "Z",
                "w": 1
            }, {
                "id": "X",
                "top": "",
                "main": "X",
                "w": 1
            }, {
                "id": "C",
                "top": "",
                "main": "C",
                "w": 1
            }, {
                "id": "V",
                "top": "",
                "main": "V",
                "w": 1
            }, {
                "id": "B",
                "top": "",
                "main": "B",
                "w": 1
            }, {
                "id": "N",
                "top": "",
                "main": "N",
                "w": 1
            }, {
                "id": "M",
                "top": "",
                "main": "M",
                "w": 1
            }, {
                "id": ",",
                "top": "<",
                "main": ",",
                "w": 1
            }, {
                "id": ".",
                "top": ">",
                "main": ".",
                "w": 1
            }, {
                "id": "/",
                "top": "?",
                "main": "/",
                "w": 1
            }, {
                "id": "RightShift",
                "top": "",
                "main": "shift",
                "w": 2.75
            }], row5];

        if (layoutMode === "iso_de")
            return [row0, [{
                "id": "",
                "top": "°",
                "main": "^",
                "w": 1
            }, {
                "id": "1",
                "top": "!",
                "main": "1",
                "w": 1
            }, {
                "id": "2",
                "top": "\"",
                "main": "2",
                "w": 1
            }, {
                "id": "3",
                "top": "§",
                "main": "3",
                "w": 1
            }, {
                "id": "4",
                "top": "$",
                "main": "4",
                "w": 1
            }, {
                "id": "5",
                "top": "%",
                "main": "5",
                "w": 1
            }, {
                "id": "6",
                "top": "&",
                "main": "6",
                "w": 1
            }, {
                "id": "7",
                "top": "/",
                "main": "7",
                "w": 1
            }, {
                "id": "8",
                "top": "(",
                "main": "8",
                "w": 1
            }, {
                "id": "9",
                "top": ")",
                "main": "9",
                "w": 1
            }, {
                "id": "0",
                "top": "=",
                "main": "0",
                "w": 1
            }, {
                "id": "-",
                "top": "?",
                "main": "ß",
                "w": 1
            }, {
                "id": "=",
                "top": "",
                "main": "´",
                "w": 1
            }, {
                "id": "Backspace",
                "top": "",
                "main": "backspace",
                "w": 2
            }], [{
                "id": "Tab",
                "top": "",
                "main": "tab",
                "w": 1.5
            }, {
                "id": "Q",
                "top": "",
                "main": "Q",
                "w": 1
            }, {
                "id": "W",
                "top": "",
                "main": "W",
                "w": 1
            }, {
                "id": "E",
                "top": "",
                "main": "E",
                "w": 1
            }, {
                "id": "R",
                "top": "",
                "main": "R",
                "w": 1
            }, {
                "id": "T",
                "top": "",
                "main": "T",
                "w": 1
            }, {
                "id": "Y",
                "top": "",
                "main": "Z",
                "w": 1
            }, {
                "id": "U",
                "top": "",
                "main": "U",
                "w": 1
            }, {
                "id": "I",
                "top": "",
                "main": "I",
                "w": 1
            }, {
                "id": "O",
                "top": "",
                "main": "O",
                "w": 1
            }, {
                "id": "P",
                "top": "",
                "main": "P",
                "w": 1
            }, {
                "id": "[",
                "top": "",
                "main": "Ü",
                "w": 1
            }, {
                "id": "]",
                "top": "*",
                "main": "+",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "return",
                "w": 1.5,
                "isoEnterTop": true
            }], [{
                "id": "CapsLock",
                "top": "",
                "main": "caps lock",
                "w": 1.75
            }, {
                "id": "A",
                "top": "",
                "main": "A",
                "w": 1
            }, {
                "id": "S",
                "top": "",
                "main": "S",
                "w": 1
            }, {
                "id": "D",
                "top": "",
                "main": "D",
                "w": 1
            }, {
                "id": "F",
                "top": "",
                "main": "F",
                "w": 1
            }, {
                "id": "G",
                "top": "",
                "main": "G",
                "w": 1
            }, {
                "id": "H",
                "top": "",
                "main": "H",
                "w": 1
            }, {
                "id": "J",
                "top": "",
                "main": "J",
                "w": 1
            }, {
                "id": "K",
                "top": "",
                "main": "K",
                "w": 1
            }, {
                "id": "L",
                "top": "",
                "main": "L",
                "w": 1
            }, {
                "id": ";",
                "top": "",
                "main": "Ö",
                "w": 1
            }, {
                "id": "'",
                "top": "",
                "main": "Ä",
                "w": 1
            }, {
                "id": "\\",
                "top": "'",
                "main": "#",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "",
                "w": 1.25,
                "isoEnterBottom": true
            }], [{
                "id": "LeftShift",
                "top": "",
                "main": "shift",
                "w": 1.25
            }, {
                "id": "Key_86",
                "top": ">",
                "main": "<",
                "w": 1
            }, {
                "id": "Z",
                "top": "",
                "main": "Y",
                "w": 1
            }, {
                "id": "X",
                "top": "",
                "main": "X",
                "w": 1
            }, {
                "id": "C",
                "top": "",
                "main": "C",
                "w": 1
            }, {
                "id": "V",
                "top": "",
                "main": "V",
                "w": 1
            }, {
                "id": "B",
                "top": "",
                "main": "B",
                "w": 1
            }, {
                "id": "N",
                "top": "",
                "main": "N",
                "w": 1
            }, {
                "id": "M",
                "top": "",
                "main": "M",
                "w": 1
            }, {
                "id": ",",
                "top": ";",
                "main": ",",
                "w": 1
            }, {
                "id": ".",
                "top": ":",
                "main": ".",
                "w": 1
            }, {
                "id": "/",
                "top": "_",
                "main": "-",
                "w": 1
            }, {
                "id": "RightShift",
                "top": "",
                "main": "shift",
                "w": 2.75
            }], row5];

        if (layoutMode === "iso_fr")
            return [row0, [{
                "id": "`",
                "top": "",
                "main": "²",
                "w": 1
            }, {
                "id": "1",
                "top": "1",
                "main": "&",
                "w": 1
            }, {
                "id": "2",
                "top": "2",
                "main": "é",
                "w": 1
            }, {
                "id": "3",
                "top": "3",
                "main": "\"",
                "w": 1
            }, {
                "id": "4",
                "top": "4",
                "main": "'",
                "w": 1
            }, {
                "id": "5",
                "top": "5",
                "main": "(",
                "w": 1
            }, {
                "id": "6",
                "top": "6",
                "main": "-",
                "w": 1
            }, {
                "id": "7",
                "top": "7",
                "main": "è",
                "w": 1
            }, {
                "id": "8",
                "top": "8",
                "main": "_",
                "w": 1
            }, {
                "id": "9",
                "top": "9",
                "main": "ç",
                "w": 1
            }, {
                "id": "0",
                "top": "0",
                "main": "à",
                "w": 1
            }, {
                "id": "-",
                "top": "°",
                "main": ")",
                "w": 1
            }, {
                "id": "=",
                "top": "+",
                "main": "=",
                "w": 1
            }, {
                "id": "Backspace",
                "top": "",
                "main": "backspace",
                "w": 2
            }], [{
                "id": "Tab",
                "top": "",
                "main": "tab",
                "w": 1.5
            }, {
                "id": "Q",
                "top": "",
                "main": "A",
                "w": 1
            }, {
                "id": "W",
                "top": "",
                "main": "Z",
                "w": 1
            }, {
                "id": "E",
                "top": "",
                "main": "E",
                "w": 1
            }, {
                "id": "R",
                "top": "",
                "main": "R",
                "w": 1
            }, {
                "id": "T",
                "top": "",
                "main": "T",
                "w": 1
            }, {
                "id": "Y",
                "top": "",
                "main": "Y",
                "w": 1
            }, {
                "id": "U",
                "top": "",
                "main": "U",
                "w": 1
            }, {
                "id": "I",
                "top": "",
                "main": "I",
                "w": 1
            }, {
                "id": "O",
                "top": "",
                "main": "O",
                "w": 1
            }, {
                "id": "P",
                "top": "",
                "main": "P",
                "w": 1
            }, {
                "id": "[",
                "top": "¨",
                "main": "^",
                "w": 1
            }, {
                "id": "]",
                "top": "£",
                "main": "$",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "return",
                "w": 1.5,
                "isoEnterTop": true
            }], [{
                "id": "CapsLock",
                "top": "",
                "main": "caps lock",
                "w": 1.75
            }, {
                "id": "A",
                "top": "",
                "main": "Q",
                "w": 1
            }, {
                "id": "S",
                "top": "",
                "main": "S",
                "w": 1
            }, {
                "id": "D",
                "top": "",
                "main": "D",
                "w": 1
            }, {
                "id": "F",
                "top": "",
                "main": "F",
                "w": 1
            }, {
                "id": "G",
                "top": "",
                "main": "G",
                "w": 1
            }, {
                "id": "H",
                "top": "",
                "main": "H",
                "w": 1
            }, {
                "id": "J",
                "top": "",
                "main": "J",
                "w": 1
            }, {
                "id": "K",
                "top": "",
                "main": "K",
                "w": 1
            }, {
                "id": "L",
                "top": "",
                "main": "L",
                "w": 1
            }, {
                "id": ";",
                "top": "",
                "main": "M",
                "w": 1
            }, {
                "id": "'",
                "top": "%",
                "main": "ù",
                "w": 1
            }, {
                "id": "\\",
                "top": "µ",
                "main": "*",
                "w": 1
            }, {
                "id": "Enter",
                "top": "",
                "main": "",
                "w": 1.25,
                "isoEnterBottom": true
            }], [{
                "id": "LeftShift",
                "top": "",
                "main": "shift",
                "w": 1.25
            }, {
                "id": "Key_86",
                "top": ">",
                "main": "<",
                "w": 1
            }, {
                "id": "Z",
                "top": "",
                "main": "W",
                "w": 1
            }, {
                "id": "X",
                "top": "",
                "main": "X",
                "w": 1
            }, {
                "id": "C",
                "top": "",
                "main": "C",
                "w": 1
            }, {
                "id": "V",
                "top": "",
                "main": "V",
                "w": 1
            }, {
                "id": "B",
                "top": "",
                "main": "B",
                "w": 1
            }, {
                "id": "N",
                "top": "",
                "main": "N",
                "w": 1
            }, {
                "id": "M",
                "top": "?",
                "main": ",",
                "w": 1
            }, {
                "id": ",",
                "top": ".",
                "main": ";",
                "w": 1
            }, {
                "id": ".",
                "top": "/",
                "main": ":",
                "w": 1
            }, {
                "id": "/",
                "top": "§",
                "main": "!",
                "w": 1
            }, {
                "id": "RightShift",
                "top": "",
                "main": "shift",
                "w": 2.75
            }], row5];

        // Default: ANSI (US)
        return [row0, [{
            "id": "",
            "top": "~",
            "main": "",
            "w": 1
        }, {
            "id": "1",
            "top": "!",
            "main": "1",
            "w": 1
        }, {
            "id": "2",
            "top": "@",
            "main": "2",
            "w": 1
        }, {
            "id": "3",
            "top": "#",
            "main": "3",
            "w": 1
        }, {
            "id": "4",
            "top": "$",
            "main": "4",
            "w": 1
        }, {
            "id": "5",
            "top": "%",
            "main": "5",
            "w": 1
        }, {
            "id": "6",
            "top": "^",
            "main": "6",
            "w": 1
        }, {
            "id": "7",
            "top": "&",
            "main": "7",
            "w": 1
        }, {
            "id": "8",
            "top": "*",
            "main": "8",
            "w": 1
        }, {
            "id": "9",
            "top": "(",
            "main": "9",
            "w": 1
        }, {
            "id": "0",
            "top": ")",
            "main": "0",
            "w": 1
        }, {
            "id": "-",
            "top": "_",
            "main": "-",
            "w": 1
        }, {
            "id": "=",
            "top": "+",
            "main": "=",
            "w": 1
        }, {
            "id": "Backspace",
            "top": "",
            "main": "backspace",
            "w": 2
        }], [{
            "id": "Tab",
            "top": "",
            "main": "tab",
            "w": 1.5
        }, {
            "id": "Q",
            "top": "",
            "main": "Q",
            "w": 1
        }, {
            "id": "W",
            "top": "",
            "main": "W",
            "w": 1
        }, {
            "id": "E",
            "top": "",
            "main": "E",
            "w": 1
        }, {
            "id": "R",
            "top": "",
            "main": "R",
            "w": 1
        }, {
            "id": "T",
            "top": "",
            "main": "T",
            "w": 1
        }, {
            "id": "Y",
            "top": "",
            "main": "Y",
            "w": 1
        }, {
            "id": "U",
            "top": "",
            "main": "U",
            "w": 1
        }, {
            "id": "I",
            "top": "",
            "main": "I",
            "w": 1
        }, {
            "id": "O",
            "top": "",
            "main": "O",
            "w": 1
        }, {
            "id": "P",
            "top": "",
            "main": "P",
            "w": 1
        }, {
            "id": "[",
            "top": "{",
            "main": "[",
            "w": 1
        }, {
            "id": "]",
            "top": "}",
            "main": "]",
            "w": 1
        }, {
            "id": "\\",
            "top": "|",
            "main": "\\",
            "w": 1.5
        }], [{
            "id": "CapsLock",
            "top": "",
            "main": "caps lock",
            "w": 1.75
        }, {
            "id": "A",
            "top": "",
            "main": "A",
            "w": 1
        }, {
            "id": "S",
            "top": "",
            "main": "S",
            "w": 1
        }, {
            "id": "D",
            "top": "",
            "main": "D",
            "w": 1
        }, {
            "id": "F",
            "top": "",
            "main": "F",
            "w": 1
        }, {
            "id": "G",
            "top": "",
            "main": "G",
            "w": 1
        }, {
            "id": "H",
            "top": "",
            "main": "H",
            "w": 1
        }, {
            "id": "J",
            "top": "",
            "main": "J",
            "w": 1
        }, {
            "id": "K",
            "top": "",
            "main": "K",
            "w": 1
        }, {
            "id": "L",
            "top": "",
            "main": "L",
            "w": 1
        }, {
            "id": ";",
            "top": ":",
            "main": ";",
            "w": 1
        }, {
            "id": "'",
            "top": "\"",
            "main": "'",
            "w": 1
        }, {
            "id": "Enter",
            "top": "",
            "main": "return",
            "w": 2.25
        }], [{
            "id": "LeftShift",
            "top": "",
            "main": "shift",
            "w": 2.25
        }, {
            "id": "Z",
            "top": "",
            "main": "Z",
            "w": 1
        }, {
            "id": "X",
            "top": "",
            "main": "X",
            "w": 1
        }, {
            "id": "C",
            "top": "",
            "main": "C",
            "w": 1
        }, {
            "id": "V",
            "top": "",
            "main": "V",
            "w": 1
        }, {
            "id": "B",
            "top": "",
            "main": "B",
            "w": 1
        }, {
            "id": "N",
            "top": "",
            "main": "N",
            "w": 1
        }, {
            "id": "M",
            "top": "",
            "main": "M",
            "w": 1
        }, {
            "id": ",",
            "top": "<",
            "main": ",",
            "w": 1
        }, {
            "id": ".",
            "top": ">",
            "main": ".",
            "w": 1
        }, {
            "id": "/",
            "top": "?",
            "main": "/",
            "w": 1
        }, {
            "id": "RightShift",
            "top": "",
            "main": "shift",
            "w": 2.75
        }], row5];
    }

    function canonicalKey(rawName) {
        var shiftToBase = {
            "~": "`",
            "!": "1",
            "@": "2",
            "#": "3",
            "$": "4",
            "%": "5",
            "^": "6",
            "&": "7",
            "*": "8",
            "(": "9",
            ")": "0",
            "_": "-",
            "+": "=",
            "{": "[",
            "}": "]",
            "|": "\\",
            ":": ";",
            "\"": "'",
            "<": ",",
            ">": ".",
            "?": "/",
            "£": "3",
            "¬": "`",
            "¦": "`",
            "§": "3",
            "°": "`",
            "€": "E"
        };
        if (shiftToBase[rawName])
            return shiftToBase[rawName];

        var modComboMap = {
            "Ctrl+Shift": "LeftShift",
            "Ctrl+Alt": "LeftAlt",
            "Shift+Alt": "LeftAlt",
            "Ctrl+Shift+Alt": "LeftShift"
        };
        if (modComboMap[rawName])
            return modComboMap[rawName];

        if (rawName.indexOf("+") !== -1) {
            var parts = rawName.split("+");
            var tail = parts[parts.length - 1];
            if (shiftToBase[tail])
                return shiftToBase[tail];

            return tail;
        }
        return rawName;
    }

    function formatBadge(n) {
        if (n >= 10000)
            return (n / 1000).toFixed(0) + "k";

        if (n >= 1000)
            return (n / 1000).toFixed(1) + "k";

        return n.toString();
    }

    function formatNumber(n) {
        return Math.round(n).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    }

    function parseDates(output) {
        var lines = output.trim().split("\n");
        var list = [];
        for (var i = 0; i < lines.length; i++) {
            var d = lines[i].trim();
            if (d.length === 10)
                list.push(d);

        }
        if (list.length > 0) {
            win.availableDates = list;
            if (win.selectedDate === "" || list.indexOf(win.selectedDate) === -1) {
                win.dateIndex = list.length - 1;
                win.selectedDate = list[win.dateIndex];
            } else {
                win.dateIndex = list.indexOf(win.selectedDate);
            }
        }
        dbSource.connectSource(win.countsCmd);
    }

    function parseCounts(output) {
        var lines = output.trim().split("\n");
        var totals = {
        };
        var details = {
        };
        var sumAll = 0;
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].trim();
            if (!line)
                continue;

            var sep = line.lastIndexOf("|");
            if (sep <= 0)
                continue;

            var rawKey = line.slice(0, sep);
            var cnt = parseInt(line.slice(sep + 1), 10) || 0;
            if (cnt <= 0)
                continue;

            sumAll += cnt;
            var canon = canonicalKey(rawKey);
            totals[canon] = (totals[canon] || 0) + cnt;
            if (!details[canon])
                details[canon] = [];

            details[canon].push(rawKey + ": " + formatNumber(cnt));
        }
        var maxC = 1;
        var active = 0;
        for (var k in totals) {
            if (totals[k] > 0) {
                active += 1;
                if (totals[k] > maxC)
                    maxC = totals[k];

            }
        }
        win.keyTotals = totals;
        win.keyDetails = details;
        win.totalKeys = sumAll;
        win.activeKeys = active;
        win.maxKeyCount = maxC;
    }

    function reload() {
        dbSource.connectSource(datesCmd);
    }

    title: "KeyStats — Keyboard Heatmap"
    width: 960
    height: 495
    minimumWidth: 820
    minimumHeight: 450
    color: Kirigami.Theme.backgroundColor
    onVisibleChanged: {
        if (visible)
            reload();

    }

    Plasma5Support.DataSource {
        id: dbSource

        engine: "executable"
        connectedSources: []
        onNewData: function(sourceName, data) {
            var stdout = data["stdout"] || "";
            if (sourceName === win.datesCmd)
                win.parseDates(stdout);
            else if (sourceName === win.countsCmd)
                win.parseCounts(stdout);
            disconnectSource(sourceName);
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 12

        RowLayout {
            Layout.fillWidth: true

            ColumnLayout {
                spacing: 2

                QQC2.Label {
                    text: "Keyboard Heatmap"
                    font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.45
                    font.bold: true
                }

                QQC2.Label {
                    text: "Virtual keyboard view for daily, 7-day, and 30-day key intensity."
                    font.pointSize: Kirigami.Theme.smallFont.pointSize
                    opacity: 0.68
                }

            }

            Item {
                Layout.fillWidth: true
            }

            QQC2.Label {
                text: "Layout:"
                font.pointSize: Kirigami.Theme.smallFont.pointSize
                opacity: 0.75
            }

            QQC2.ComboBox {
                id: layoutCombo

                implicitWidth: 135
                model: [{
                    "id": "ansi_us",
                    "label": "ANSI (US)"
                }, {
                    "id": "iso_uk",
                    "label": "ISO (UK)"
                }, {
                    "id": "iso_de",
                    "label": "ISO (DE)"
                }, {
                    "id": "iso_fr",
                    "label": "ISO (FR)"
                }]
                textRole: "label"
                valueRole: "id"
                Component.onCompleted: {
                    var idx = indexOfValue(win.layoutMode)
                    if (idx >= 0) currentIndex = idx
                }
                onActivated: {
                    win.layoutMode = currentValue;
                }
            }

        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            RowLayout {
                spacing: 4

                Repeater {
                    model: [{
                        "id": "day",
                        "label": "Day"
                    }, {
                        "id": "7d",
                        "label": "7D"
                    }, {
                        "id": "30d",
                        "label": "30D"
                    }]

                    delegate: QQC2.Button {
                        text: modelData.label
                        checkable: true
                        checked: win.rangeMode === modelData.id
                        onClicked: {
                            win.rangeMode = modelData.id;
                            dbSource.connectSource(win.countsCmd);
                        }
                    }

                }

            }

            RowLayout {
                visible: win.rangeMode === "day"
                spacing: 4

                QQC2.Button {
                    text: "<"
                    implicitWidth: 32
                    enabled: win.dateIndex > 0
                    onClicked: {
                        if (win.dateIndex > 0) {
                            win.dateIndex -= 1;
                            win.selectedDate = win.availableDates[win.dateIndex];
                            dbSource.connectSource(win.countsCmd);
                        }
                    }
                }

                Rectangle {
                    radius: 6
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.06)
                    implicitWidth: dateLbl.implicitWidth + 20
                    implicitHeight: 30

                    QQC2.Label {
                        id: dateLbl

                        anchors.centerIn: parent
                        font.bold: true
                        text: {
                            if (win.availableDates.length === 0)
                                return "Today";

                            var pos = (win.dateIndex + 1) + "/" + win.availableDates.length;
                            return win.selectedDate + "  (" + pos + ")";
                        }
                    }

                }

                QQC2.Button {
                    text: ">"
                    implicitWidth: 32
                    enabled: win.dateIndex < win.availableDates.length - 1
                    onClicked: {
                        if (win.dateIndex < win.availableDates.length - 1) {
                            win.dateIndex += 1;
                            win.selectedDate = win.availableDates[win.dateIndex];
                            dbSource.connectSource(win.countsCmd);
                        }
                    }
                }

            }

            Item {
                Layout.fillWidth: true
            }

            QQC2.Label {
                text: "Total " + win.formatNumber(win.totalKeys) + " keys · " + win.activeKeys + " active keys"
                font.pointSize: Kirigami.Theme.smallFont.pointSize * 1.05
                opacity: 0.75
            }

        }

        Rectangle {
            id: kbFrame

            readonly property real gap: 5
            readonly property real pad: 10
            readonly property real unitW: (width - pad * 2 - gap * 14) / 15
            readonly property real rowH: (height - pad * 2 - gap * 5) / 6

            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: 12
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.035)
            border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
            border.width: 1

            Column {
                anchors.fill: parent
                anchors.margins: kbFrame.pad
                spacing: kbFrame.gap

                Repeater {
                    model: win.keyboardRows

                    delegate: Row {
                        property var rowKeys: modelData

                        spacing: kbFrame.gap

                        Repeater {
                            model: rowKeys

                            delegate: Item {
                                id: keyCell

                                property var kInfo: modelData
                                property int keyCount: win.keyTotals[kInfo.id] || 0
                                property real intensity: keyCount > 0 ? Math.min(1, Math.sqrt(keyCount / win.maxKeyCount)) : 0
                                property bool isIsoHalf: !!kInfo.isoEnterTop || !!kInfo.isoEnterBottom

                                width: kInfo.w * kbFrame.unitW + (kInfo.w - 1) * kbFrame.gap
                                height: kbFrame.rowH

                                MouseArea {
                                    visible: keyCell.isIsoHalf
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    onEntered: {
                                        win.isoEnterHovered = true
                                        if (keyCell.keyCount === 0) {
                                            win.hoveredSummary = "return — 0 presses"
                                        } else {
                                            var breakdown = (win.keyDetails["Enter"] || []).join(", ")
                                            win.hoveredSummary = "return — " + win.formatNumber(keyCell.keyCount) + " total (" + breakdown + ")"
                                        }
                                    }
                                    onExited: win.isoEnterHovered = false
                                }

                                Rectangle {
                                    visible: !kInfo.splitUpDown && !keyCell.isIsoHalf
                                    anchors.fill: parent
                                    // Extend top ISO Enter half down across the row gap to connect seamlessly with the bottom half
                                    anchors.bottomMargin: kInfo.isoEnterTop ? -(kbFrame.gap + 2) : 0
                                    z: keyCell.isIsoHalf ? 2 : 1
                                    radius: 6
                                    color: keyCell.keyCount > 0 ? Qt.rgba(0.32, 0.72, 0.34, 0.22 + 0.68 * keyCell.intensity) : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.06)
                                    border.color: keyHover.containsMouse ? Kirigami.Theme.highlightColor : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.14)
                                    border.width: keyHover.containsMouse ? 1.5 : 1

                                    Rectangle {
                                        visible: keyCell.keyCount > 0 && !kInfo.isoEnterBottom
                                        x: 4
                                        y: 4
                                        radius: height / 2
                                        width: Math.max(17, badgeText.implicitWidth + 7)
                                        height: 16
                                        color: Qt.rgba(0.18, 0.45, 0.2, 0.85)

                                        QQC2.Label {
                                            id: badgeText

                                            anchors.centerIn: parent
                                            text: win.formatBadge(keyCell.keyCount)
                                            color: "#ffffff"
                                            font.pixelSize: 9
                                            font.bold: true
                                        }

                                    }

                                    QQC2.Label {
                                        visible: kInfo.top !== ""
                                        anchors.right: parent.right
                                        anchors.top: parent.top
                                        anchors.rightMargin: 6
                                        anchors.topMargin: 4
                                        text: kInfo.top
                                        font.pixelSize: 10
                                        opacity: 0.65
                                    }

                                    QQC2.Label {
                                        visible: kInfo.main !== ""
                                        anchors.right: parent.right
                                        anchors.bottom: parent.bottom
                                        anchors.rightMargin: 6
                                        anchors.bottomMargin: kInfo.isoEnterTop ? (kbFrame.gap + 6) : 4
                                        text: kInfo.main
                                        font.pixelSize: kInfo.main.length > 2 ? 10 : 12
                                        font.bold: keyCell.keyCount > 0
                                        opacity: 0.88
                                    }

                                    MouseArea {
                                        id: keyHover

                                        anchors.fill: parent
                                        hoverEnabled: true
                                        onEntered: {
                                            var label = kInfo.top !== "" ? (kInfo.main + " / " + kInfo.top) : (kInfo.main || kInfo.id);
                                            if (keyCell.keyCount === 0) {
                                                win.hoveredSummary = label + " — 0 presses";
                                            } else {
                                                var breakdown = (win.keyDetails[kInfo.id] || []).join(", ");
                                                win.hoveredSummary = label + " — " + win.formatNumber(keyCell.keyCount) + " total (" + breakdown + ")";
                                            }
                                        }
                                    }

                                }

                                Column {
                                    visible: !!kInfo.splitUpDown
                                    anchors.fill: parent
                                    spacing: 3

                                    Repeater {
                                        model: [{
                                            "id": "Up",
                                            "arrow": "▲"
                                        }, {
                                            "id": "Down",
                                            "arrow": "▼"
                                        }]

                                        delegate: Rectangle {
                                            property int arrowCnt: win.keyTotals[modelData.id] || 0
                                            property real arrowInt: arrowCnt > 0 ? Math.min(1, Math.sqrt(arrowCnt / win.maxKeyCount)) : 0

                                            width: parent.width
                                            height: (parent.height - 3) / 2
                                            radius: 4
                                            color: arrowCnt > 0 ? Qt.rgba(0.32, 0.72, 0.34, 0.22 + 0.68 * arrowInt) : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.06)
                                            border.color: arrowHover.containsMouse ? Kirigami.Theme.highlightColor : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.14)
                                            border.width: 1

                                            RowLayout {
                                                anchors.fill: parent
                                                anchors.leftMargin: 4
                                                anchors.rightMargin: 5
                                                spacing: 2

                                                Rectangle {
                                                    visible: arrowCnt > 0
                                                    radius: 6
                                                    width: Math.max(14, arrowBadge.implicitWidth + 5)
                                                    height: 12
                                                    color: Qt.rgba(0.18, 0.45, 0.2, 0.85)

                                                    QQC2.Label {
                                                        id: arrowBadge

                                                        anchors.centerIn: parent
                                                        text: win.formatBadge(arrowCnt)
                                                        color: "#ffffff"
                                                        font.pixelSize: 8
                                                        font.bold: true
                                                    }

                                                }

                                                Item {
                                                    Layout.fillWidth: true
                                                }

                                                QQC2.Label {
                                                    text: modelData.arrow
                                                    font.pixelSize: 9
                                                    opacity: 0.85
                                                }

                                            }

                                            MouseArea {
                                                id: arrowHover

                                                anchors.fill: parent
                                                hoverEnabled: true
                                                onEntered: {
                                                    win.hoveredSummary = modelData.id + " Arrow — " + win.formatNumber(arrowCnt) + " presses";
                                                }
                                            }

                                        }

                                    }

                                }

                            }

                        }

                    }

                }

            }

            // Unified seamless L-shaped ISO Return keycap
            Item {
                visible: win.isIsoLayout
                anchors.fill: parent
                anchors.margins: kbFrame.pad
                z: 5

                property int enterCount: win.keyTotals["Enter"] || 0
                property real enterIntensity: enterCount > 0 ? Math.min(1, Math.sqrt(enterCount / win.maxKeyCount)) : 0
                property bool hovered: win.isoEnterHovered

                onEnterCountChanged: isoCanvas.requestPaint()
                onEnterIntensityChanged: isoCanvas.requestPaint()
                onHoveredChanged: isoCanvas.requestPaint()

                property real topW: 1.5 * kbFrame.unitW + 0.5 * kbFrame.gap
                property real botW: 1.25 * kbFrame.unitW + 0.25 * kbFrame.gap
                property real notchW: topW - botW
                property real topY: 2 * (kbFrame.rowH + kbFrame.gap)
                property real midY: topY + kbFrame.rowH
                property real botY: topY + 2 * kbFrame.rowH + kbFrame.gap
                property real rightX: width
                property real topLeftX: rightX - topW
                property real botLeftX: rightX - botW

                Canvas {
                    id: isoCanvas
                    anchors.fill: parent
                    onWidthChanged: requestPaint()
                    onHeightChanged: requestPaint()

                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)

                        var r = 6
                        var x0 = parent.topLeftX
                        var x1 = parent.botLeftX
                        var x2 = parent.rightX
                        var y0 = parent.topY
                        var y1 = parent.midY
                        var y2 = parent.botY

                        ctx.beginPath()
                        ctx.moveTo(x0 + r, y0)
                        ctx.lineTo(x2 - r, y0)
                        ctx.arcTo(x2, y0, x2, y0 + r, r)
                        ctx.lineTo(x2, y2 - r)
                        ctx.arcTo(x2, y2, x2 - r, y2, r)
                        ctx.lineTo(x1 + r, y2)
                        ctx.arcTo(x1, y2, x1, y2 - r, r)
                        ctx.lineTo(x1, y1 + r)
                        ctx.arcTo(x1, y1, x1 - r, y1, r)
                        ctx.lineTo(x0 + r, y1)
                        ctx.arcTo(x0, y1, x0, y1 - r, r)
                        ctx.lineTo(x0, y0 + r)
                        ctx.arcTo(x0, y0, x0 + r, y0, r)
                        ctx.closePath()

                        ctx.fillStyle = parent.enterCount > 0
                            ? Qt.rgba(0.32, 0.72, 0.34, 0.22 + 0.68 * parent.enterIntensity)
                            : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.06)
                        ctx.fill()

                        ctx.lineWidth = parent.hovered ? 1.5 : 1
                        ctx.strokeStyle = parent.hovered
                            ? Kirigami.Theme.highlightColor
                            : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.14)
                        ctx.stroke()
                    }
                }

                Rectangle {
                    visible: parent.enterCount > 0
                    x: parent.topLeftX + 5
                    y: parent.topY + 5
                    radius: height / 2
                    width: Math.max(17, isoBadgeText.implicitWidth + 7)
                    height: 16
                    color: Qt.rgba(0.18, 0.45, 0.2, 0.85)

                    QQC2.Label {
                        id: isoBadgeText
                        anchors.centerIn: parent
                        text: win.formatBadge(parent.parent.enterCount)
                        color: "#ffffff"
                        font.pixelSize: 9
                        font.bold: true
                    }
                }

                QQC2.Label {
                    x: parent.rightX - width - 7
                    y: parent.botY - height - 5
                    text: "return"
                    font.pixelSize: 10
                    font.bold: parent.enterCount > 0
                    opacity: 0.88
                }
            }
        }

        QQC2.Label {
            Layout.fillWidth: true
            text: win.hoveredSummary
            font.pointSize: Kirigami.Theme.smallFont.pointSize
            opacity: 0.75
            elide: Text.ElideRight
        }

    }

}
