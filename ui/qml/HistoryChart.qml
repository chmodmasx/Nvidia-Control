import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: card

    property string heading: ""
    property string metricKey: ""
    property string unit: ""
    property int minutes: 5
    property int decimals: 0
    property bool percentage: false

    implicitWidth: 380
    implicitHeight: 262
    radius: 14
    color: "#2c3039"
    border.color: "#454b57"
    border.width: 1

    function formatReading(value) {
        if (value === undefined || value === null || !isFinite(Number(value)))
            return "—"
        return Number(value).toFixed(decimals) + " " + unit
    }

    function repaint() {
        if (card.visible)
            graph.requestPaint()
    }

    onMinutesChanged: repaint()
    onMetricKeyChanged: repaint()
    onWidthChanged: repaint()
    onVisibleChanged: repaint()
    Component.onCompleted: repaint()

    Connections {
        target: telemetryHistory
        function onUpdated() { card.repaint() }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 5

        RowLayout {
            Layout.fillWidth: true

            Label {
                text: card.heading
                color: "#e5edf7"
                font.pixelSize: 15
                font.weight: Font.DemiBold
                Layout.fillWidth: true
                elide: Text.ElideRight
            }

            Label {
                id: currentReading
                color: "#9dcee9"
                font.pixelSize: 13
                text: "—"
            }
        }

        Canvas {
            id: graph
            Layout.fillWidth: true
            Layout.fillHeight: true
            renderTarget: Canvas.Image

            onPaint: {
                const ctx = getContext("2d")
                const w = width
                const h = height
                ctx.clearRect(0, 0, w, h)

                const points = telemetryHistory.points(card.metricKey, card.minutes)
                const cutoff = Date.now() - card.minutes * 60000
                const left = 10
                const right = w - 12
                const top = 22
                const bottom = h - 14
                const usableWidth = Math.max(1, right - left)
                const usableHeight = Math.max(1, bottom - top)

                ctx.strokeStyle = "#434c5a"
                ctx.lineWidth = 1
                ctx.beginPath()
                for (let i = 0; i <= 4; ++i) {
                    const y = top + usableHeight * i / 4
                    ctx.moveTo(left, y)
                    ctx.lineTo(right, y)
                }
                ctx.stroke()

                let low = Infinity
                let high = -Infinity
                let latest = null

                for (let i = 0; i < points.length; ++i) {
                    const v = points[i].v
                    if (v === null || v === undefined || !isFinite(Number(v)))
                        continue
                    const value = Number(v)
                    low = Math.min(low, value)
                    high = Math.max(high, value)
                    latest = value
                }

                currentReading.text = card.formatReading(latest)
                if (latest === null) {
                    ctx.fillStyle = "#8998aa"
                    ctx.textAlign = "center"
                    ctx.font = "12px sans-serif"
                    ctx.fillText("Esperando lecturas…", w / 2, h / 2)
                    return
                }

                if (card.percentage) {
                    low = 0
                    high = 100
                } else {
                    const padding = Math.max((high - low) * 0.12, Math.max(1, high * 0.03))
                    low = Math.max(0, low - padding)
                    high += padding
                }
                if (high <= low)
                    high = low + 1

                ctx.fillStyle = "#8393a8"
                ctx.font = "11px sans-serif"
                ctx.textAlign = "left"
                ctx.fillText(high.toFixed(card.decimals) + " " + card.unit, left, 12)
                ctx.fillText(low.toFixed(card.decimals) + " " + card.unit, left, h - 1)

                ctx.strokeStyle = "#51a2da"
                ctx.lineWidth = 2
                ctx.lineJoin = "round"
                ctx.beginPath()
                let previousTime = -1
                let hasSegment = false

                for (let i = 0; i < points.length; ++i) {
                    const p = points[i]
                    const v = p.v
                    if (v === null || v === undefined || !isFinite(Number(v))) {
                        hasSegment = false
                        previousTime = -1
                        continue
                    }

                    const timestamp = Number(p.t)
                    const x = left + Math.max(0, Math.min(1,
                                       (timestamp - cutoff) / (card.minutes * 60000))) * usableWidth
                    const y = bottom - (Number(v) - low) / (high - low) * usableHeight

                    // An interruption is not a continuous reading. Never draw
                    // a line over disconnected periods or missing sensors.
                    if (!hasSegment || timestamp - previousTime > 3500)
                        ctx.moveTo(x, y)
                    else
                        ctx.lineTo(x, y)
                    hasSegment = true
                    previousTime = timestamp
                }

                ctx.stroke()
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Label {
                text: "Hace " + card.minutes + " min"
                font.pixelSize: 11
                color: "#8192a6"
            }
            Item { Layout.fillWidth: true }
            Label {
                text: "Ahora"
                font.pixelSize: 11
                color: "#8192a6"
            }
        }
    }
}
