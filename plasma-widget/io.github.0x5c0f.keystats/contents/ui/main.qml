import QtQuick
import QtQuick.Layouts
import QtQuick.Window
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
            var pxPerMeter = (Screen.pixelDensity > 0) ? (Screen.pixelDensity * 1000.0) : 3780.0
            var meters = px / pxPerMeter
            if (meters >= 1000.0) return (meters / 1000.0).toFixed(2) + " km"
                if (meters >= 1.0) return meters.toFixed(1) + " m"
                    return formatNumber(px) + " px"
    }

    // function formatDistance(px) {
    //     if (isNaN(px)) return "0 px"
    //         if (px >= 3780000) return (px / 3780000).toFixed(2) + " km"
    //             if (px >= 3780) return (px / 3780).toFixed(1) + " m"
    //                 return formatNumber(px) + " px"
    // }

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
                    if (scrollMatch) root.scrollDistance = formatDistance(parseInt(scrollMatch[1], 10))
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

    function parseCompactCount(str) {
        var m = str.match(/^([\d.]+)\s*([KkMm])?$/)
        if (!m) return 0
        var num = parseFloat(m[1])
        var suffix = (m[2] || "").toUpperCase()
        if (suffix === "K") return Math.round(num * 1000)
        if (suffix === "M") return Math.round(num * 1000000)
        return Math.round(num)
    }

    function parseHistory(output) {
        var lines = output.split("\n")
        var counts = Array()
        var labels = Array()

        for (var i = 0; i < lines.length; i++) {
            var m = lines[i].match(/^(\d{2}-\d{2})\s+.*?([\d.]+[KkMm]?)\s*$/)
            if (m) {
                labels.push(m[1])
                counts.push(parseCompactCount(m[2]))
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

            // 7-Day History Line Chart with Auto-Scaling Y-Axis & Hover Tooltip
            Rectangle {
                id: historyBox
                Layout.fillWidth: true
                Layout.preferredHeight: 120
                radius: Kirigami.Units.smallSpacing
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.04)

                property int padLeft: 36
                property int padRight: 10
                property int hoveredIndex: -1

                function niceMax(rawMax) {
                    if (rawMax <= 10) return 10
                    var magnitude = Math.pow(10, Math.floor(Math.log(rawMax) / Math.LN10))
                    var residual = rawMax / magnitude
                    var nice
                    if (residual <= 1.0) nice = 1.0
                    else if (residual <= 2.0) nice = 2.0
                    else if (residual <= 2.5) nice = 2.5
                    else if (residual <= 5.0) nice = 5.0
                    else nice = 10.0
                    return Math.ceil(nice * magnitude)
                }

                function formatAxisVal(v) {
                    if (v >= 1000000) {
                        var m = v / 1000000
                        return (m % 1 === 0 ? m.toFixed(0) : m.toFixed(1)) + "M"
                    }
                    if (v >= 1000) {
                        var k = v / 1000
                        return (k % 1 === 0 ? k.toFixed(0) : k.toFixed(1)) + "k"
                    }
                    return Math.round(v).toString()
                }

                ColumnLayout {
                    id: chartCol
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.mediumSpacing
                    spacing: 4

                    Canvas {
                        id: historyCanvas
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        property var pointsData: root.historyData
                        property int activeIdx: historyBox.hoveredIndex
                        onPointsDataChanged: requestPaint()
                        onActiveIdxChanged: requestPaint()
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()

                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)

                            var data = root.historyData
                            if (!data || data.length < 7) return

                            var rawMax = Math.max.apply(null, data) || 0
                            var maxVal = historyBox.niceMax(rawMax)

                            var padLeft = historyBox.padLeft
                            var padRight = historyBox.padRight
                            var padTop = 8
                            var padBottom = 6
                            var plotW = width - padLeft - padRight
                            var plotH = height - padTop - padBottom
                            var stepX = plotW / 6

                            ctx.font = "10px sans-serif"
                            ctx.textAlign = "right"
                            ctx.textBaseline = "middle"

                            for (var g = 0; g <= 2; g++) {
                                var gy = padTop + (plotH * g / 2)
                                var tickVal = maxVal * (1.0 - g / 2)

                                ctx.strokeStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.09)
                                ctx.lineWidth = 1
                                ctx.beginPath()
                                ctx.moveTo(padLeft, gy)
                                ctx.lineTo(width - padRight, gy)
                                ctx.stroke()

                                ctx.fillStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.65)
                                ctx.fillText(historyBox.formatAxisVal(tickVal), padLeft - 6, gy)
                            }

                            var pts = Array()
                            for (var i = 0; i < 7; i++) {
                                var val = data[i] || 0
                                var ratio = Math.min(1.0, val / maxVal)
                                pts.push({
                                    x: padLeft + stepX * i,
                                    y: padTop + plotH * (1.0 - ratio)
                                })
                            }

                            var grad = ctx.createLinearGradient(0, padTop, 0, padTop + plotH)
                            grad.addColorStop(0, Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.28))
                            grad.addColorStop(1, Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.0))

                            ctx.beginPath()
                            ctx.moveTo(pts[0].x, padTop + plotH)
                            for (var j = 0; j < pts.length; j++) {
                                ctx.lineTo(pts[j].x, pts[j].y)
                            }
                            ctx.lineTo(pts[6].x, padTop + plotH)
                            ctx.closePath()
                            ctx.fillStyle = grad
                            ctx.fill()

                            ctx.beginPath()
                            ctx.moveTo(pts[0].x, pts[0].y)
                            for (var k = 1; k < pts.length; k++) {
                                ctx.lineTo(pts[k].x, pts[k].y)
                            }
                            ctx.strokeStyle = Kirigami.Theme.highlightColor
                            ctx.lineWidth = 2
                            ctx.lineJoin = "round"
                            ctx.lineCap = "round"
                            ctx.stroke()

                            if (historyBox.hoveredIndex >= 0 && historyBox.hoveredIndex < pts.length) {
                                var hx = pts[historyBox.hoveredIndex].x
                                ctx.strokeStyle = Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.28)
                                ctx.lineWidth = 1
                                ctx.beginPath()
                                ctx.moveTo(hx, padTop)
                                ctx.lineTo(hx, padTop + plotH)
                                ctx.stroke()
                            }

                            for (var d = 0; d < pts.length; d++) {
                                var isToday = (d === 6)
                                var isHovered = (d === historyBox.hoveredIndex)
                                if (isHovered) {
                                    ctx.beginPath()
                                    ctx.arc(pts[d].x, pts[d].y, 6.5, 0, Math.PI * 2)
                                    ctx.fillStyle = Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.25)
                                    ctx.fill()
                                }
                                ctx.beginPath()
                                ctx.arc(pts[d].x, pts[d].y, isHovered ? 4.5 : (isToday ? 4 : 2.5), 0, Math.PI * 2)
                                ctx.fillStyle = (isToday || isHovered)
                                    ? Kirigami.Theme.highlightColor
                                    : Kirigami.Theme.textColor
                                ctx.fill()
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 14

                        Repeater {
                            model: 7
                            delegate: PlasmaComponents.Label {
                                property real plotW: parent.width - historyBox.padLeft - historyBox.padRight
                                property real centerX: historyBox.padLeft + (plotW / 6) * index

                                x: centerX - width / 2
                                text: root.dayLabels[index] || ""
                                font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.78
                                font.bold: index === historyBox.hoveredIndex
                                opacity: (index === 6 || index === historyBox.hoveredIndex) ? 1.0 : 0.6
                            }
                        }
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    onPositionChanged: function(mouse) {
                        var margin = Kirigami.Units.mediumSpacing
                        var localX = mouse.x - margin - historyBox.padLeft
                        var plotW = historyBox.width - margin * 2 - historyBox.padLeft - historyBox.padRight
                        if (plotW <= 0) return
                        var idx = Math.round(localX / (plotW / 6))
                        historyBox.hoveredIndex = Math.max(0, Math.min(6, idx))
                    }
                    onExited: historyBox.hoveredIndex = -1
                }

                Rectangle {
                    id: hoverTooltip
visible: historyBox.hoveredIndex >= 0 && historyBox.hoveredIndex < root.historyData.length
z: 20
radius: 4
color: Kirigami.Theme.backgroundColor
border.color: Kirigami.Theme.highlightColor
border.width: 1
implicitWidth: tooltipLbl.implicitWidth + 10
implicitHeight: tooltipLbl.implicitHeight + 4
property real margin: Kirigami.Units.mediumSpacing
property real plotW: historyBox.width - margin * 2 - historyBox.padLeft - historyBox.padRight
property real targetX: margin + historyBox.padLeft + (plotW / 6) * Math.max(0, historyBox.hoveredIndex)
x: Math.max(4, Math.min(historyBox.width - width - 4, targetX - width / 2))
y: -height - 2
PlasmaComponents.Label {
id: tooltipLbl
anchors.centerIn: parent
font.pointSize: Kirigami.Theme.smallFont.pointSize * 0.85
font.bold: true
text: {
var idx = historyBox.hoveredIndex
if (idx < 0 || idx >= root.historyData.length) return ""
var day = root.dayLabels[idx] || ""
var val = formatNumber(root.historyData[idx] || 0)
var unit = root.historyMode === "keys" ? "keys" : "clicks"
return day + ": " + val + " " + unit
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
