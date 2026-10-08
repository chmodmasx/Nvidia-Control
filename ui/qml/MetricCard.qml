import QtQuick
import QtQuick.Controls

Rectangle {
    id: card
    property string label: ""
    property string value: "—"
    property string subtext: ""
    property real progress: -1

    radius: 16
    color: "#2c3039"
    border.width: 1
    border.color: "#454b57"
    implicitHeight: 134
    implicitWidth: 205

    Column {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 8

        Label {
            text: card.label
            color: "#abb7ca"
            font.pixelSize: 13
        }

        Label {
            text: card.value
            color: "#f6f6f6"
            font.pixelSize: 26
            font.weight: Font.DemiBold
        }

        Label {
            text: card.subtext
            color: "#8495ab"
            font.pixelSize: 11
            visible: text.length > 0
        }

        Rectangle {
            visible: card.progress >= 0
            width: parent.width
            height: 6
            radius: 3
            color: "#414753"

            Rectangle {
                width: parent.width * Math.min(1, Math.max(0, card.progress))
                height: parent.height
                radius: parent.radius
                color: "#51a2da"
            }
        }
    }
}
