import QtQuick
import QtCore
import QtQuick.Layouts
import QtQuick.Window
import org.kde.kirigami as Kirigami
import org.kde.plasma.components as PlasmaComponents
import org.kde.plasma.plasma5support as Plasma5Support
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore

PlasmoidItem {
    id: root

    // Live metric state
    property string keyPresses: "0"
    property string totalClicks: "0"
    property string leftClicks: "0"
    property string rightClicks: "0"
    property string middleClicks: "0"
    property string mouseDistance: "0 px"
    property string scrollDistance: "0 px"
    property string kpsText: "0"
    property string kpsPeak: "0"
    property string cpsText: "0"
    property string cpsPeak: "0"
    property bool daemonConnected: false
    // History chart state
    property int historyDays: 7
    // 7 | 30
    property string chartStyle: "line"
    // "line" | "bar"
    property string historyMode: "keys"
    // "keys" | "clicks" | "distance" | "scroll"
    property var historyData: [0, 0, 0, 0, 0, 0, 0]
    property var dayLabels: ["-", "-", "-", "-", "-", "-", "Today"]
    property string historyTotalText: "0"
    // Top 15 keys (3 columns x 5 rows)
    property var topKeys: []
    readonly property string ctlBin: "$(command -v keystatsctl || echo /usr/local/bin/keystatsctl)"
    readonly property string dbPath: "$HOME/.local/state/keystats/stats.sqlite3"
    readonly property string statusCmd: "sh -c \"" + ctlBin + " status\""
    readonly property string stateDbPath: StandardPaths.writableLocation(StandardPaths.GenericStateLocation).toString().replace(/^file:\/\//, "") + "/keystats/stats.sqlite3"
    readonly property string clickBreakdownCmd: "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT left_clicks, right_clicks, middle_clicks FROM daily_stats WHERE date = date(\x27now\x27, \x27localtime\x27) LIMIT 1;\""
    readonly property string keysCmd: "sh -c \"" + ctlBin + " keys --limit 15\""

    readonly property string historyCmd: {
        var expr = "key_presses"
        if (historyMode === "clicks") {
            expr = "(left_clicks + right_clicks + middle_clicks + side_back_clicks + side_forward_clicks)"
        } else if (historyMode === "distance") {
            expr = "CAST(ROUND(mouse_distance) AS INTEGER)"
        } else if (historyMode === "scroll") {
            expr = "CAST(ROUND(scroll_distance) AS INTEGER)"
        }
        var offset = "-" + (historyDays - 1) + " days"
        return "sqlite3 file:" + stateDbPath + "?mode=ro \"SELECT strftime(\x27%m-%d\x27, date), " + expr + " FROM daily_stats WHERE date >= date(\x27now\x27, \x27localtime\x27, \x27" + offset + "\x27) ORDER BY date ASC;\""
    }

    function formatNumber(n) {
        if (isNaN(n))
            return "0";

        return Math.round(n).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    }

    function formatCompact(n) {
        if (isNaN(n) || n <= 0)
            return "0";

        if (n >= 1e+06) {
            var m = n / 1e+06;
            return (m >= 10 ? m.toFixed(0) : m.toFixed(1)) + "M";
        }
        if (n >= 1000) {
            var k = n / 1000;
            return (k >= 100 ? k.toFixed(0) : k.toFixed(1)) + "k";
        }
        return Math.round(n).toString();
    }

    function pxPerMeter() {
        return (Screen.pixelDensity > 0) ? (Screen.pixelDensity * 1000) : 3780;
    }

    function formatDistance(px) {
        if (isNaN(px) || px <= 0)
            return "0 m";

        var meters = px / pxPerMeter();
        if (meters >= 1000)
            return (meters / 1000).toFixed(2) + " km";

        if (meters >= 1)
            return meters.toFixed(1) + " m";

        return formatNumber(px) + " px";
    }

    function shortKeyLabel(name) {
        var map = {
            "LeftControl": "LCtrl",
            "RightControl": "RCtrl",
            "LeftShift": "LShift",
            "RightShift": "RShift",
            "LeftAlt": "LAlt",
            "RightAlt": "RAlt",
            "LeftMeta": "Super",
            "RightMeta": "RSuper",
            "Backspace": "Backspace",
            "CapsLock": "CapsLk",
            "PageUp": "PgUp",
            "PageDown": "PgDn",
            "Delete": "Del",
            "Escape": "Esc"
        };
        if (map[name])
            return map[name];

        if (name.length > 8)
            return name.slice(0, 7) + "...";

        return name;
    }

    function parseCompactCount(str) {
        var m = str.match(/^([\d.]+)\s*([KkMm])?$/);
        if (!m)
            return 0;

        var num = parseFloat(m[1]);
        var suffix = (m[2] || "").toUpperCase();
        if (suffix === "K")
            return Math.round(num * 1000);

        if (suffix === "M")
            return Math.round(num * 1e+06);

        return Math.round(num);
    }

    function parseStatus(output) {
        root.daemonConnected = output.indexOf("connected") !== -1 && output.indexOf("disconnected") === -1;
        var keysMatch = output.match(/Key presses:\s*(\d+)/);
        var clicksMatch = output.match(/Total clicks:\s*(\d+)/);
        var distMatch = output.match(/Mouse distance:\s*([\d.]+)/);
        var scrollMatch = output.match(/Scroll distance:\s*([\d.]+)/);
        var kpsMatch = output.match(/KPS:\s*(\d+)\s*\(peak\s*(\d+)\)/);
        var cpsMatch = output.match(/CPS:\s*(\d+)\s*\(peak\s*(\d+)\)/);
        if (keysMatch)
            root.keyPresses = formatNumber(parseInt(keysMatch[1], 10));

        if (clicksMatch) {
            root.totalClicks = formatNumber(parseInt(clicksMatch[1], 10));
            if (root.leftClicks === "0" && root.rightClicks === "0")
                root.leftClicks = root.totalClicks;

        }
        if (distMatch)
            root.mouseDistance = formatDistance(parseFloat(distMatch[1]));

        if (scrollMatch)
            root.scrollDistance = formatDistance(parseFloat(scrollMatch[1]));

        if (kpsMatch) {
            root.kpsText = kpsMatch[1];
            root.kpsPeak = kpsMatch[2];
        }
        if (cpsMatch) {
            root.cpsText = cpsMatch[1];
            root.cpsPeak = cpsMatch[2];
        }
    }

    function parseClickBreakdown(output) {
        var line = output.trim();
        if (!line)
            return ;

        var parts = line.split("|");
        if (parts.length >= 3) {
            root.leftClicks = formatNumber(parseInt(parts[0], 10) || 0);
            root.rightClicks = formatNumber(parseInt(parts[1], 10) || 0);
            root.middleClicks = formatNumber(parseInt(parts[2], 10) || 0);
        }
    }

    function parseHistory(output) {
        var lines = output.split("
")
        var counts = []
        var labels = []
        var isDistMode = (root.historyMode === "distance" || root.historyMode === "scroll")

        var byDate = {}
        for (var i = 0; i < lines.length; i++) {
            var p = lines[i].trim().split("|")
            if (p.length === 2) {
                byDate[p[0]] = parseInt(p[1], 10) || 0
            }
        }

        var now = new Date()
        for (var d = root.historyDays - 1; d >= 0; d--) {
            var dt = new Date(now.getFullYear(), now.getMonth(), now.getDate() - d)
            var mm = String(dt.getMonth() + 1).padStart(2, "0")
            var dd = String(dt.getDate()).padStart(2, "0")
            var key = mm + "-" + dd
            labels.push(key)
            counts.push(byDate[key] || 0)
        }

        if (counts.length > 0) {
            labels[labels.length - 1] = "Today"
            root.dayLabels = labels
            root.historyData = counts

            var sum = 0
            for (var s = 0; s < counts.length; s++) sum += (counts[s] || 0)
            root.historyTotalText = isDistMode ? formatDistance(sum) : formatNumber(sum)
        }
    }

    function parseTopKeys(output) {
        var lines = output.split("\n");
        var items = [];
        for (var i = 0; i < lines.length; i++) {
            var m = lines[i].match(/^(\S+)\s+.*?([\d.]+[KkMm]?)\s*$/);
            if (m && m[1] !== "Key")
                items.push({
                    "keyName": shortKeyLabel(m[1]),
                    "fullName": m[1],
                    "count": m[2]
                });

            if (items.length >= 15)
                break;

        }
        root.topKeys = items;
    }

    function refreshAll() {
        execSource.connectSource(statusCmd);
        if (root.expanded) {
            execSource.connectSource(clickBreakdownCmd);
            execSource.connectSource(historyCmd);
            execSource.connectSource(keysCmd);
        }
    }

    onExpandedChanged: {
        if (expanded)
            refreshAll();

    }
    Plasmoid.contextualActions: [
        PlasmaCore.Action {
            text: "Keyboard Heatmap..."
            icon.name: "input-keyboard"
            onTriggered: {
                heatmapWin.show()
                heatmapWin.raise()
                heatmapWin.requestActivate()
            }
        }
    ]

    HeatmapWindow {
        id: heatmapWin
    }

    Plasmoid.status: PlasmaCore.Types.ActiveStatus
    toolTipMainText: "KeyStats"
    toolTipSubText: root.daemonConnected ? ("Keys: " + root.keyPresses + "  |  Clicks: " + root.totalClicks + "\nDistance: " + root.mouseDistance + "  |  Scroll: " + root.scrollDistance) : "Daemon disconnected"

    Plasma5Support.DataSource {
        id: execSource

        engine: "executable"
        connectedSources: []
        onNewData: function(sourceName, data) {
            var stdout = data["stdout"] || "";
            if (sourceName === root.statusCmd)
                root.parseStatus(stdout);
            else if (sourceName === root.clickBreakdownCmd)
                root.parseClickBreakdown(stdout);
            else if (sourceName === root.historyCmd)
                root.parseHistory(stdout);
            else if (sourceName === root.keysCmd)
                root.parseTopKeys(stdout);
            disconnectSource(sourceName);
        }
    }

    Timer {
        id: pollTimer

        interval: root.expanded ? 1500 : 3000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: root.refreshAll()
    }

    // Compact Panel Indicator (Adaptive: stacks on thick panels, inline on thin panels)
    compactRepresentation: MouseArea {
        id: compactRoot

        readonly property bool isVertical: Plasmoid.formFactor === PlasmaCore.Types.Vertical
        readonly property bool isStacked: !isVertical && height >= 36

        implicitWidth: isVertical ? parent.width : (isStacked ? stackedCol.implicitWidth + 10 : inlineRow.implicitWidth + 16)
        Layout.minimumWidth: implicitWidth
        implicitHeight: isVertical ? stackedCol.implicitHeight + 8 : parent.height
        onClicked: root.expanded = !root.expanded

        RowLayout {
            id: inlineRow

            visible: !compactRoot.isStacked && !compactRoot.isVertical
            anchors.centerIn: parent
            spacing: 5

            Kirigami.Icon {
                source: "input-keyboard"
                Layout.preferredWidth: Kirigami.Units.iconSizes.small
                Layout.preferredHeight: Kirigami.Units.iconSizes.small
                opacity: root.daemonConnected ? 1 : 0.45
            }

            PlasmaComponents.Label {
                text: root.keyPresses
                font.pointSize: Kirigami.Theme.smallFont.pointSize
                font.bold: true
            }

            Kirigami.Icon {
                source: "input-mouse"
                Layout.preferredWidth: Kirigami.Units.iconSizes.small
                Layout.preferredHeight: Kirigami.Units.iconSizes.small
                opacity: root.daemonConnected ? 1 : 0.45
            }

            PlasmaComponents.Label {
                text: root.totalClicks
                font.pointSize: Kirigami.Theme.smallFont.pointSize
                font.bold: true
            }

        }

        ColumnLayout {
            id: stackedCol

            visible: compactRoot.isStacked || compactRoot.isVertical
            anchors.centerIn: parent
            spacing: 0

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 4

                Kirigami.Icon {
                    source: "input-keyboard"
                    Layout.preferredWidth: 16
                    Layout.preferredHeight: 16
                    opacity: root.daemonConnected ? 1 : 0.45
                }

                PlasmaComponents.Label {
                    text: root.keyPresses
                    font.pointSize: Kirigami.Theme.smallFont.pointSize * 1.2
                    font.bold: true
                    lineHeight: 0.9
                }

            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 4

                Kirigami.Icon {
                    source: "input-mouse"
                    Layout.preferredWidth: 16
                    Layout.preferredHeight: 16
                    opacity: root.daemonConnected ? 1 : 0.45
                }

                PlasmaComponents.Label {
                    text: root.totalClicks
                    font.pointSize: Kirigami.Theme.smallFont.pointSize * 1.2
                    font.bold: true
                    lineHeight: 0.9
                }

            }

        }

    }

    // Full Popup Dashboard (Windows KeyStats-inspired layout)
    fullRepresentation: Item {
        Layout.minimumWidth: 360
        Layout.minimumHeight: 600
        Layout.preferredWidth: 375
        Layout.preferredHeight: 625

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: Kirigami.Units.largeSpacing
            spacing: Kirigami.Units.mediumSpacing

            // Header + Live Rate Badges
            RowLayout {
                Layout.fillWidth: true

                PlasmaComponents.Label {
                    text: "Today's Stats"
                    font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.15
                    font.bold: true
                }

                Item {
                    Layout.fillWidth: true
                }

                Rectangle {
                    radius: 4
                    color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.15)
                    implicitWidth: kpsLbl.implicitWidth + 12
                    implicitHeight: kpsLbl.implicitHeight + 6

                    PlasmaComponents.Label {
                        id: kpsLbl

                        anchors.centerIn: parent
                        text: "KPS " + root.kpsText + " (" + root.kpsPeak + ")"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                        font.bold: true
                        color: Kirigami.Theme.highlightColor
                    }

                }

                Rectangle {
                    radius: 4
                    color: Qt.rgba(Kirigami.Theme.positiveTextColor.r, Kirigami.Theme.positiveTextColor.g, Kirigami.Theme.positiveTextColor.b, 0.15)
                    implicitWidth: cpsLbl.implicitWidth + 12
                    implicitHeight: cpsLbl.implicitHeight + 6

                    PlasmaComponents.Label {
                        id: cpsLbl

                        anchors.centerIn: parent
                        text: "CPS " + root.cpsText + " (" + root.cpsPeak + ")"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                        font.bold: true
                        color: Kirigami.Theme.positiveTextColor
                    }

                }

            }

            // Full-Width Keystrokes Hero Card
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 58
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Kirigami.Units.largeSpacing
                    anchors.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.largeSpacing

                    Rectangle {
                        width: 32
                        height: 32
                        radius: 6
                        color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.12)

                        Kirigami.Icon {
                            anchors.centerIn: parent
                            source: "input-keyboard"
                            width: 18
                            height: 18
                        }

                    }

                    ColumnLayout {
                        spacing: 1

                        PlasmaComponents.Label {
                            text: "Keystrokes"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.9
                            opacity: 0.7
                        }

                        PlasmaComponents.Label {
                            text: root.keyPresses
                            font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.35
                            font.bold: true
                        }

                    }

                    Item {
                        Layout.fillWidth: true
                    }

                }

            }

            // 2x2 Stat Cards Grid (Left Clicks, Right Clicks, Mouse Distance, Scroll Distance)
            GridLayout {
                Layout.fillWidth: true
                columns: 2
                rowSpacing: Kirigami.Units.smallSpacing
                columnSpacing: Kirigami.Units.smallSpacing

                // Card 1: Left Clicks
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.mediumSpacing
                        spacing: Kirigami.Units.mediumSpacing

                        Rectangle {
                            width: 28
                            height: 28
                            radius: 6
                            color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.12)

                            Kirigami.Icon {
                                anchors.centerIn: parent
                                source: "input-mouse"
                                width: 16
                                height: 16
                            }

                        }

                        ColumnLayout {
                            spacing: 1

                            PlasmaComponents.Label {
                                text: "Left Clicks"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                                opacity: 0.7
                            }

                            PlasmaComponents.Label {
                                text: root.leftClicks
                                font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.15
                                font.bold: true
                            }

                        }

                        Item {
                            Layout.fillWidth: true
                        }

                    }

                }

                // Card 2: Right Clicks (+ Middle badge if > 0)
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.mediumSpacing
                        spacing: Kirigami.Units.mediumSpacing

                        Rectangle {
                            width: 28
                            height: 28
                            radius: 6
                            color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.12)

                            Kirigami.Icon {
                                anchors.centerIn: parent
                                source: "input-mouse"
                                width: 16
                                height: 16
                            }

                        }

                        ColumnLayout {
                            spacing: 1

                            PlasmaComponents.Label {
                                text: root.middleClicks !== "0" ? ("Right (" + root.middleClicks + " mid)") : "Right Clicks"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                                opacity: 0.7
                            }

                            PlasmaComponents.Label {
                                text: root.rightClicks
                                font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.15
                                font.bold: true
                            }

                        }

                        Item {
                            Layout.fillWidth: true
                        }

                    }

                }

                // Card 3: Mouse Distance
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.mediumSpacing
                        spacing: Kirigami.Units.mediumSpacing

                        Rectangle {
                            width: 28
                            height: 28
                            radius: 6
                            color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.12)

                            Kirigami.Icon {
                                anchors.centerIn: parent
                                source: "transform-move"
                                width: 16
                                height: 16
                            }

                        }

                        ColumnLayout {
                            spacing: 1

                            PlasmaComponents.Label {
                                text: "Mouse Distance"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                                opacity: 0.7
                            }

                            PlasmaComponents.Label {
                                text: root.mouseDistance
                                font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.15
                                font.bold: true
                            }

                        }

                        Item {
                            Layout.fillWidth: true
                        }

                    }

                }

                // Card 4: Scroll Distance
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.mediumSpacing
                        spacing: Kirigami.Units.mediumSpacing

                        Rectangle {
                            width: 28
                            height: 28
                            radius: 6
                            color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.12)

                            Kirigami.Icon {
                                anchors.centerIn: parent
                                source: "go-up"
                                width: 16
                                height: 16
                            }

                        }

                        ColumnLayout {
                            spacing: 1

                            PlasmaComponents.Label {
                                text: "Scroll Distance"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.88
                                opacity: 0.7
                            }

                            PlasmaComponents.Label {
                                text: root.scrollDistance
                                font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.15
                                font.bold: true
                            }

                        }

                        Item {
                            Layout.fillWidth: true
                        }

                    }

                }

            }

            // Key Breakdown Section (15 keys in 3-column x 5-row grid)
            PlasmaComponents.Label {
                text: "Key Breakdown"
                font.bold: true
                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.98
                opacity: 0.85
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 132
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.04)
                border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                border.width: 1

                GridLayout {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.mediumSpacing
                    columns: 3
                    flow: GridLayout.TopToBottom
                    rows: 5
                    columnSpacing: Kirigami.Units.largeSpacing
                    rowSpacing: 3

                    Repeater {
                        model: root.topKeys

                        delegate: RowLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Rectangle {
                                radius: 3
                                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.07)
                                border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
                                border.width: 1
                                implicitWidth: 64
                                implicitHeight: 19

                                PlasmaComponents.Label {
                                    anchors.centerIn: parent
                                    text: modelData.keyName
                                    font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.82
                                    font.bold: true
                                }

                            }

                            PlasmaComponents.Label {
                                text: modelData.count
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                                opacity: 0.8
                                horizontalAlignment: Text.AlignRight
                                Layout.fillWidth: true
                            }

                        }

                    }

                    PlasmaComponents.Label {
                        visible: root.topKeys.length === 0
                        text: "No keystrokes recorded yet today"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize
                        opacity: 0.55
                    }

                }

            }
            // History Header (Title + Line / Bar toggle)

            RowLayout {
                Layout.fillWidth: true

                PlasmaComponents.Label {
                    text: "History"
                    font.bold: true
                    font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.98
                    opacity: 0.85
                }

                Item {
                    Layout.fillWidth: true
                }

                RowLayout {
                    spacing: 2

                    PlasmaComponents.ToolButton {
                        text: "Line"
                        checked: root.chartStyle === "line"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                        onClicked: {
                            root.chartStyle = "line";
                            historyCanvas.requestPaint();
                        }
                    }

                    PlasmaComponents.ToolButton {
                        text: "Bar"
                        checked: root.chartStyle === "bar"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                        onClicked: {
                            root.chartStyle = "bar";
                            historyCanvas.requestPaint();
                        }
                    }

                }

            }
            // History Card (7D/30D + Keys/Clicks/Distance/Scroll + Canvas + Total)

            Rectangle {
                id: historyBox

                property int padLeft: 36
                property int padRight: 10
                property int hoveredIndex: -1

                function niceMax(rawMax) {
                    if (rawMax <= 10)
                        return 10;

                    var magnitude = Math.pow(10, Math.floor(Math.log(rawMax) / Math.LN10));
                    var residual = rawMax / magnitude;
                    var nice;
                    if (residual <= 1)
                        nice = 1;
                    else if (residual <= 2)
                        nice = 2;
                    else if (residual <= 2.5)
                        nice = 2.5;
                    else if (residual <= 5)
                        nice = 5;
                    else
                        nice = 10;
                    return Math.ceil(nice * magnitude);
                }

                function formatAxisVal(v) {
                    if (root.historyMode === "distance" || root.historyMode === "scroll") {
                        var meters = v / root.pxPerMeter();
                        if (meters >= 1000)
                            return (meters / 1000).toFixed(1) + "km";

                        if (meters >= 1)
                            return Math.round(meters) + "m";

                        return Math.round(v) + "px";
                    }
                    if (v >= 1e+06) {
                        var m = v / 1e+06;
                        return (m % 1 === 0 ? m.toFixed(0) : m.toFixed(1)) + "M";
                    }
                    if (v >= 1000) {
                        var k = v / 1000;
                        return (k % 1 === 0 ? k.toFixed(0) : k.toFixed(1)) + "k";
                    }
                    return Math.round(v).toString();
                }

                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 150
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.04)
                border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                border.width: 1

                ColumnLayout {
                    id: chartCol

                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.mediumSpacing
                    spacing: 4

                    // Filter Bar: 7D | 30D   and   Keys | Clicks | Distance | Scroll
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        PlasmaComponents.ToolButton {
                            text: "7D"
                            checked: root.historyDays === 7
                            font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.82
                            onClicked: {
                                root.historyDays = 7;
                                historyBox.hoveredIndex = -1;
                                execSource.connectSource(root.historyCmd);
                            }
                        }

                        PlasmaComponents.ToolButton {
                            text: "30D"
                            checked: root.historyDays === 30
                            font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.82
                            onClicked: {
                                root.historyDays = 30;
                                historyBox.hoveredIndex = -1;
                                execSource.connectSource(root.historyCmd);
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        Repeater {
                            model: [{
                                "id": "keys",
                                "label": "Keys"
                            }, {
                                "id": "clicks",
                                "label": "Clicks"
                            }, {
                                "id": "distance",
                                "label": "Dist"
                            }, {
                                "id": "scroll",
                                "label": "Scroll"
                            }]

                            delegate: PlasmaComponents.ToolButton {
                                text: modelData.label
                                checked: root.historyMode === modelData.id
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.82
                                onClicked: {
                                    root.historyMode = modelData.id;
                                    historyBox.hoveredIndex = -1;
                                    execSource.connectSource(root.historyCmd);
                                }
                            }

                        }

                    }

                    Canvas {
                        id: historyCanvas

                        property var pointsData: root.historyData
                        property int activeIdx: historyBox.hoveredIndex
                        property string styleMode: root.chartStyle

                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        onPointsDataChanged: requestPaint()
                        onActiveIdxChanged: requestPaint()
                        onStyleModeChanged: requestPaint()
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()
                        onPaint: {
                            var ctx = getContext("2d");
                            ctx.clearRect(0, 0, width, height);
                            var data = root.historyData;
                            var n = data ? data.length : 0;
                            if (n < 2)
                                return ;

                            var rawMax = Math.max.apply(null, data) || 0;
                            var maxVal = historyBox.niceMax(rawMax);
                            var padLeft = historyBox.padLeft;
                            var padRight = historyBox.padRight;
                            var padTop = 8;
                            var padBottom = 6;
                            var plotW = width - padLeft - padRight;
                            var plotH = height - padTop - padBottom;
                            var stepX = plotW / (n - 1);
                            ctx.font = "10px sans-serif";
                            ctx.textAlign = "right";
                            ctx.textBaseline = "middle";
                            for (var g = 0; g <= 2; g++) {
                                var gy = padTop + (plotH * g / 2);
                                var tickVal = maxVal * (1 - g / 2);
                                ctx.strokeStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.09);
                                ctx.lineWidth = 1;
                                ctx.beginPath();
                                ctx.moveTo(padLeft, gy);
                                ctx.lineTo(width - padRight, gy);
                                ctx.stroke();
                                ctx.fillStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.65);
                                ctx.fillText(historyBox.formatAxisVal(tickVal), padLeft - 6, gy);
                            }
                            var pts = [];
                            for (var i = 0; i < n; i++) {
                                var val = data[i] || 0;
                                var ratio = Math.min(1, val / maxVal);
                                pts.push({
                                    "x": padLeft + stepX * i,
                                    "y": padTop + plotH * (1 - ratio),
                                    "h": Math.max(2, plotH * ratio)
                                });
                            }
                            if (root.chartStyle === "bar") {
                                var barW = Math.max(3, Math.min(18, (plotW / n) * 0.62));
                                for (var b = 0; b < n; b++) {
                                    var isTodayB = (b === n - 1);
                                    var isHoveredB = (b === historyBox.hoveredIndex);
                                    ctx.fillStyle = (isTodayB || isHoveredB) ? Kirigami.Theme.highlightColor : Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.45);
                                    ctx.fillRect(pts[b].x - barW / 2, padTop + plotH - pts[b].h, barW, pts[b].h);
                                }
                            } else {
                                var grad = ctx.createLinearGradient(0, padTop, 0, padTop + plotH);
                                grad.addColorStop(0, Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.28));
                                grad.addColorStop(1, Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0));
                                ctx.beginPath();
                                ctx.moveTo(pts[0].x, padTop + plotH);
                                for (var j = 0; j < n; j++) {
                                    ctx.lineTo(pts[j].x, pts[j].y);
                                }
                                ctx.lineTo(pts[n - 1].x, padTop + plotH);
                                ctx.closePath();
                                ctx.fillStyle = grad;
                                ctx.fill();
                                ctx.beginPath();
                                ctx.moveTo(pts[0].x, pts[0].y);
                                for (var k = 1; k < n; k++) {
                                    ctx.lineTo(pts[k].x, pts[k].y);
                                }
                                ctx.strokeStyle = Kirigami.Theme.highlightColor;
                                ctx.lineWidth = 2;
                                ctx.lineJoin = "round";
                                ctx.lineCap = "round";
                                ctx.stroke();
                                if (historyBox.hoveredIndex >= 0 && historyBox.hoveredIndex < n) {
                                    var hx = pts[historyBox.hoveredIndex].x;
                                    ctx.strokeStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.28);
                                    ctx.lineWidth = 1;
                                    ctx.beginPath();
                                    ctx.moveTo(hx, padTop);
                                    ctx.lineTo(hx, padTop + plotH);
                                    ctx.stroke();
                                }
                                for (var d = 0; d < n; d++) {
                                    var isToday = (d === n - 1);
                                    var isHovered = (d === historyBox.hoveredIndex);
                                    if (n <= 10 || isToday || isHovered || (data[d] || 0) > 0) {
                                        if (isHovered) {
                                            ctx.beginPath();
                                            ctx.arc(pts[d].x, pts[d].y, 6.5, 0, Math.PI * 2);
                                            ctx.fillStyle = Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.25);
                                            ctx.fill();
                                        }
                                        ctx.beginPath();
                                        ctx.arc(pts[d].x, pts[d].y, isHovered ? 4.5 : (isToday ? 3.8 : 2.3), 0, Math.PI * 2);
                                        ctx.fillStyle = (isToday || isHovered) ? Kirigami.Theme.highlightColor : Kirigami.Theme.textColor;
                                        ctx.fill();
                                    }
                                }
                            }
                        }
                    }
                    // Sparse X-axis date ticks (avoids crowding in 30D view)

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 14

                        Repeater {
                            model: root.dayLabels.length

                            delegate: PlasmaComponents.Label {
                                property int totalPts: root.dayLabels.length
                                property int stride: totalPts <= 7 ? 2 : 7
                                property real plotW: parent.width - historyBox.padLeft - historyBox.padRight
                                property real centerX: historyBox.padLeft + (totalPts > 1 ? (plotW / (totalPts - 1)) * index : 0)

                                visible: index === totalPts - 1 || (index % stride === 0 && (totalPts - 1 - index) >= Math.floor(stride / 2))
                                x: Math.max(0, Math.min(parent.width - width, centerX - width / 2))
                                text: root.dayLabels[index] || ""
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.78
                                font.bold: index === totalPts - 1
                                opacity: index === totalPts - 1 ? 1 : 0.6
                            }

                        }

                    }
                    // Footer Total Summary (matches Windows "Total: 13.2k" footer)

                    PlasmaComponents.Label {
                        text: "Total (" + root.historyDays + "d): " + root.historyTotalText
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                        opacity: 0.7
                    }

                }

                MouseArea {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.top: parent.top
                    anchors.topMargin: 36
                    hoverEnabled: true
                    onPositionChanged: function(mouse) {
                        var n = root.historyData.length;
                        if (n < 2)
                            return ;

                        var margin = Kirigami.Units.mediumSpacing;
                        var localX = mouse.x - margin - historyBox.padLeft;
                        var plotW = historyBox.width - margin * 2 - historyBox.padLeft - historyBox.padRight;
                        if (plotW <= 0)
                            return ;

                        var idx = Math.round(localX / (plotW / (n - 1)));
                        historyBox.hoveredIndex = Math.max(0, Math.min(n - 1, idx));
                    }
                    onExited: historyBox.hoveredIndex = -1
                }

                Rectangle {
                    id: hoverTooltip

                    property int n: Math.max(2, root.historyData.length)
                    property real margin: Kirigami.Units.mediumSpacing
                    property real plotW: historyBox.width - margin * 2 - historyBox.padLeft - historyBox.padRight
                    property real targetX: margin + historyBox.padLeft + (plotW / (n - 1)) * Math.max(0, historyBox.hoveredIndex)

                    visible: historyBox.hoveredIndex >= 0 && historyBox.hoveredIndex < root.historyData.length
                    z: 20
                    radius: 4
                    color: Kirigami.Theme.backgroundColor
                    border.color: Kirigami.Theme.highlightColor
                    border.width: 1
                    implicitWidth: tooltipLbl.implicitWidth + 10
                    implicitHeight: tooltipLbl.implicitHeight + 4
                    x: Math.max(4, Math.min(historyBox.width - width - 4, targetX - width / 2))
                    y: 28

                    PlasmaComponents.Label {
                        id: tooltipLbl

                        anchors.centerIn: parent
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                        font.bold: true
                        text: {
                            var idx = historyBox.hoveredIndex;
                            if (idx < 0 || idx >= root.historyData.length)
                                return "";

                            var day = root.dayLabels[idx] || "";
                            var raw = root.historyData[idx] || 0;
                            if (root.historyMode === "distance" || root.historyMode === "scroll")
                                return day + ": " + root.formatDistance(raw);

                            var unit = root.historyMode === "keys" ? "keys" : "clicks";
                            return day + ": " + formatNumber(raw) + " " + unit;
                        }
                    }

                }

            }

        }

    }

}
