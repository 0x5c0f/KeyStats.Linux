import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents
import org.kde.plasma.plasma5support as Plasma5Support
import org.kde.kirigami as Kirigami

PlasmoidItem {
    id: root

    property int keyCount: 0
    property int clickCount: 0
    property string mouseDistance: "0 m"
    property string scrollDistance: "0 px"
    property real currentKps: 0.0
    property real peakKps: 0.0
    property real currentCps: 0.0
    property real peakCps: 0.0
    property string historyMode: "keys"
    property var historyData: [0, 0, 0, 0, 0, 0, 0]
    property var dayLabels: ["-", "-", "-", "-", "-", "-", "Today"]
    property var topKeys: []
    property int tick: 0

    preferredRepresentation: compactRepresentation

    Plasma5Support.DataSource {
        id: executable
        engine: "executable"
        connectedSources: []

        onNewData: function(sourceName, data) {
            var stdout = data["stdout"] || ""
            if (stdout.length > 0) {
                if (sourceName.indexOf("history --") !== -1) {
                    parseHistory(stdout)
                } else if (sourceName.indexOf("keys --limit") !== -1) {
                    parseTopKeys(stdout)
                } else {
                    parseStats(stdout)
                }
            }
            disconnectSource(sourceName)
        }
    }

    function refreshHistory() {
        root.tick += 1
        executable.connectSource("sh -c '$(command -v keystatsctl || echo /usr/local/bin/keystatsctl) history --" + root.historyMode + "' # " + root.tick)
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            root.tick += 1
            executable.connectSource("sh -c '$(command -v keystatsctl || echo /usr/local/bin/keystatsctl) status' # " + root.tick)

            if (root.tick === 1 || root.tick % 10 === 0) {
                executable.connectSource("sh -c '$(command -v keystatsctl || echo /usr/local/bin/keystatsctl) keys --limit 5' # " + root.tick)
            }
            if (root.tick === 1 || root.tick % 30 === 0) {
                executable.connectSource("sh -c '$(command -v keystatsctl || echo /usr/local/bin/keystatsctl) history --" + root.historyMode + "' # " + root.tick)
            }
        }
    }

    function formatDistance(px) {
        if (isNaN(px)) return "0 px"
            if (px >= 3780000) return (px / 3780000).toFixed(2) + " km"
                if (px >= 3780) return (px / 3780).toFixed(1) + " m"
                    return formatNumber(px) + " px"
    }

    function parseStats(output) {
        var keysMatch = output.match(/Key presses:\s*(\d+)/i)
        var clicksMatch = output.match(/Total clicks:\s*(\d+)/i)
        var distMatch = output.match(/Mouse distance:\s*(\d+)/i)
        var scrollMatch = output.match(/Scroll distance:\s*(\d+)/i)
        var kpsMatch = output.match(/KPS:\s*([\d.]+)\s*\(peak\s*([\d.]+)\)/i)
        var cpsMatch = output.match(/CPS:\s*([\d.]+)\s*\(peak\s*([\d.]+)\)/i)

        if (keysMatch) root.keyCount = parseInt(keysMatch[1], 10)
            if (clicksMatch) root.clickCount = parseInt(clicksMatch[1], 10)
                if (distMatch) root.mouseDistance = formatDistance(parseInt(distMatch[1], 10))
                    if (scrollMatch) root.scrollDistance = formatNumber(parseInt(scrollMatch[1], 10)) + " px"
                        if (kpsMatch) {
                            root.currentKps = parseFloat(kpsMatch[1])
                            root.peakKps = parseFloat(kpsMatch[2])
                        }
                        if (cpsMatch) {
                            root.currentCps = parseFloat(cpsMatch[1])
                            root.peakCps = parseFloat(cpsMatch[2])
                        }

                        var updated = root.historyData.slice(0, 6)
                        updated.push(root.historyMode === "keys" ? root.keyCount : root.clickCount)
                        root.historyData = updated
    }

    function parseHistory(output) {
        var lines = output.split("\n")
        var counts = []
        var labels = []

        for (var i = 0; i < lines.length; i++) {
            var m = lines[i].match(/^(\d{2}-\d{2})\s+.*?(\d+)\s*$/)
            if (m) {
                labels.push(m[1])
                counts.push(parseInt(m[2], 10))
            }
            if (counts.length === 7) break
        }

        if (counts.length === 7) {
            labels[6] = "Today"
            root.dayLabels = labels
            root.historyData = counts
        }
    }

    function parseTopKeys(output) {
        var lines = output.split("\n")
        var parsed = []
        var maxCount = 1

        for (var i = 0; i < lines.length; i++) {
            var m = lines[i].match(/^(\S+)\s+[^\d\s]+\s+(\d+)\s*$/)
            if (m) {
                var c = parseInt(m[2], 10)
                if (parsed.length === 0) maxCount = c || 1
                    parsed.push({ name: m[1], count: c, ratio: Math.min(1.0, c / maxCount) })
            }
            if (parsed.length === 5) break
        }

        if (parsed.length > 0) {
            root.topKeys = parsed
        }
    }

    function formatNumber(num) {
        return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",")
    }

    compactRepresentation: MouseArea {
        id: compactRoot

        // Switch to 2-row stacked layout when panel height is >= 36px
        readonly property bool isStacked: height >= 36

        Layout.minimumWidth: panelGrid.implicitWidth + Kirigami.Units.smallSpacing * 2
        Layout.preferredWidth: Layout.minimumWidth
        onClicked: root.expanded = !root.expanded

        GridLayout {
            id: panelGrid
            anchors.centerIn: parent
            columns: compactRoot.isStacked ? 2 : 5
            columnSpacing: Kirigami.Units.smallSpacing
            rowSpacing: 1

            Kirigami.Icon {
                source: "input-keyboard"
                Layout.preferredWidth: compactRoot.isStacked ? 12 : Kirigami.Units.iconSizes.small
                Layout.preferredHeight: compactRoot.isStacked ? 12 : Kirigami.Units.iconSizes.small
            }

            PlasmaComponents.Label {
                text: formatNumber(root.keyCount)
                font.family: "monospace"
                font.bold: true
                font.pointSize: Kirigami.Theme.defaultFont.pointSize
                //font.pointSize: compactRoot.isStacked
                //? Kirigami.Theme.smallFont.pointSize * 0.9
                //: Kirigami.Theme.defaultFont.pointSize
                horizontalAlignment: Text.AlignRight
                Layout.fillWidth: compactRoot.isStacked
            }

            PlasmaComponents.Label {
                text: "·"
                opacity: 0.5
                visible: !compactRoot.isStacked
            }

            Kirigami.Icon {
                source: "input-mouse"
                Layout.preferredWidth: compactRoot.isStacked ? 16 : Kirigami.Units.iconSizes.small
                Layout.preferredHeight: compactRoot.isStacked ? 16 : Kirigami.Units.iconSizes.small
            }

            PlasmaComponents.Label {
                text: formatNumber(root.clickCount)
                font.family: "monospace"
                font.bold: true
                font.pointSize: Kirigami.Theme.defaultFont.pointSize
                //font.pointSize: compactRoot.isStacked
                //? Kirigami.Theme.smallFont.pointSize * 0.9
                //: Kirigami.Theme.defaultFont.pointSize
                horizontalAlignment: Text.AlignRight
                Layout.fillWidth: compactRoot.isStacked
            }
        }
    }

    fullRepresentation: Item {
        Layout.preferredWidth: Kirigami.Units.gridUnit * 20
        Layout.preferredHeight: Kirigami.Units.gridUnit * 28
        Layout.minimumWidth: Layout.preferredWidth
        Layout.minimumHeight: Layout.preferredHeight

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: Kirigami.Units.largeSpacing
            spacing: Kirigami.Units.mediumSpacing

            // Header + KPS & CPS Badges
            RowLayout {
                Layout.fillWidth: true
                spacing: 6

                PlasmaComponents.Label {
                    text: "KeyStats"
                    font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.2
                    font.bold: true
                    Layout.fillWidth: true
                }

                Rectangle {
                    color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.15)
                    radius: Kirigami.Units.smallSpacing
                    implicitWidth: kpsLabel.implicitWidth + 10
                    implicitHeight: kpsLabel.implicitHeight + 6

                    PlasmaComponents.Label {
                        id: kpsLabel
                        anchors.centerIn: parent
                        text: root.currentKps.toFixed(1) + " KPS (Peak " + root.peakKps.toFixed(1) + ")"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.92
                        font.bold: true
                        color: Kirigami.Theme.highlightColor
                    }
                }

                Rectangle {
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                    radius: Kirigami.Units.smallSpacing
                    implicitWidth: cpsLabel.implicitWidth + 10
                    implicitHeight: cpsLabel.implicitHeight + 6

                    PlasmaComponents.Label {
                        id: cpsLabel
                        anchors.centerIn: parent
                        text: root.currentCps.toFixed(1) + " CPS (Peak " + root.peakCps.toFixed(1) + ")"
                        font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.92
                        font.bold: true
                        opacity: 0.85
                    }
                }
            }

            Kirigami.Separator { Layout.fillWidth: true }

            // 2x2 Metric Cards Grid
            GridLayout {
                columns: 2
                rowSpacing: Kirigami.Units.smallSpacing
                columnSpacing: Kirigami.Units.smallSpacing
                Layout.fillWidth: true

                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 58
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        PlasmaComponents.Label {
                            text: "Keystrokes"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize
                            opacity: 0.7
                            Layout.alignment: Qt.AlignHCenter
                        }
                        PlasmaComponents.Label {
                            text: formatNumber(root.keyCount)
                            font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.25
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 58
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        PlasmaComponents.Label {
                            text: "Mouse Clicks"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize
                            opacity: 0.7
                            Layout.alignment: Qt.AlignHCenter
                        }
                        PlasmaComponents.Label {
                            text: formatNumber(root.clickCount)
                            font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.25
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 58
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        PlasmaComponents.Label {
                            text: "Mouse Distance"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize
                            opacity: 0.7
                            Layout.alignment: Qt.AlignHCenter
                        }
                        PlasmaComponents.Label {
                            text: root.mouseDistance
                            font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.1
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: 58
                    radius: Kirigami.Units.smallSpacing
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.05)
                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        PlasmaComponents.Label {
                            text: "Scroll Distance"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize
                            opacity: 0.7
                            Layout.alignment: Qt.AlignHCenter
                        }
                        PlasmaComponents.Label {
                            text: root.scrollDistance
                            font.pointSize: Kirigami.Theme.defaultFont.pointSize * 1.1
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
            }
            // 7-Day History Header + Keys / Clicks Toggle
            RowLayout {
                Layout.fillWidth: true
                PlasmaComponents.Label {
                    text: "Last 7 Days"
                    font.pointSize: Kirigami.Theme.smallFont.pointSize
                    font.bold: true
                    opacity: 0.8
                    Layout.fillWidth: true
                }
                RowLayout {
                    spacing: 4
                    Rectangle {
                        radius: 4
                        implicitWidth: keysToggleLbl.implicitWidth + 12
                        implicitHeight: keysToggleLbl.implicitHeight + 4
                        color: root.historyMode === "keys"
                        ? Kirigami.Theme.highlightColor
                        : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                        PlasmaComponents.Label {
                            id: keysToggleLbl
                            anchors.centerIn: parent
                            text: "Keys"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                            font.bold: root.historyMode === "keys"
                            color: root.historyMode === "keys" ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.textColor
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.historyMode !== "keys") {
                                    root.historyMode = "keys"
                                    root.refreshHistory()
                                }
                            }
                        }
                    }
                    Rectangle {
                        radius: 4
                        implicitWidth: clicksToggleLbl.implicitWidth + 12
                        implicitHeight: clicksToggleLbl.implicitHeight + 4
                        color: root.historyMode === "clicks"
                        ? Kirigami.Theme.highlightColor
                        : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                        PlasmaComponents.Label {
                            id: clicksToggleLbl
                            anchors.centerIn: parent
                            text: "Clicks"
                            font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
                            font.bold: root.historyMode === "clicks"
                            color: root.historyMode === "clicks" ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.textColor
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.historyMode !== "clicks") {
                                    root.historyMode = "clicks"
                                    root.refreshHistory()
                                }
                            }
                        }
                    }
                }
            }
            // 7-Day History Bar Chart
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 110
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.04)
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.mediumSpacing
                    spacing: Kirigami.Units.smallSpacing
                    Repeater {
                        model: 7
                        delegate: ColumnLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Layout.preferredWidth: 32
                            property real maxVal: Math.max.apply(null, root.historyData) || 1
                            property real ratio: Math.min(1.0, (root.historyData[index] || 0) / maxVal)
                            Item { Layout.fillHeight: true }
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: Math.max(4, Math.round(70 * ratio))
                                radius: 3
                                color: index === 6 ? Kirigami.Theme.highlightColor : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.3)
                            }
                            PlasmaComponents.Label {
                                text: root.dayLabels[index] || ""
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.8
                                opacity: index === 6 ? 1.0 : 0.6
                                Layout.preferredWidth: 32
                                horizontalAlignment: Text.AlignHCenter
                                Layout.alignment: Qt.AlignHCenter
                            }
                        }
                    }
                }
            }
            // Top Keys Breakdown Section
            PlasmaComponents.Label {
                text: "Top Keys Today"
                font.pointSize: Kirigami.Theme.smallFont.pointSize
                font.bold: true
                opacity: 0.8
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.04)
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.mediumSpacing
                    spacing: 6
                    Repeater {
                        model: root.topKeys
                        delegate: RowLayout {
                            Layout.fillWidth: true
                            spacing: Kirigami.Units.smallSpacing
                            PlasmaComponents.Label {
                                text: modelData.name
                                font.family: "monospace"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize
                                Layout.preferredWidth: 85
                                elide: Text.ElideRight
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: 8
                                radius: 4
                                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.1)
                                Rectangle {
                                    width: Math.max(4, parent.width * modelData.ratio)
                                    height: parent.height
                                    radius: 4
                                    color: Kirigami.Theme.highlightColor
                                }
                            }
                            PlasmaComponents.Label {
                                text: formatNumber(modelData.count)
                                font.family: "monospace"
                                font.pointSize: Kirigami.Theme.smallFont.pointSize
                                opacity: 0.8
                                horizontalAlignment: Text.AlignRight
                                Layout.preferredWidth: 36
                            }
                        }
                    }
                    Item { Layout.fillHeight: true }
                }
            }
        }
    }
}
