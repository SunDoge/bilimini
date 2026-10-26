import QtQuick

Item {
    id: root

    property string label: ""
    property bool active: false

    signal clicked()

    implicitHeight: 30
    implicitWidth: labelText.implicitWidth + 24

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: root.active ? Theme.accentSoft : (hover.hovered ? "#12FFFFFF" : "transparent")
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Text {
        id: labelText
        anchors.centerIn: parent
        text: root.label
        color: root.active ? Theme.accent : Theme.textMuted
        font.pixelSize: 12
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        onTapped: root.clicked()
    }
}
