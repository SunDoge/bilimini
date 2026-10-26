import QtQuick

Item {
    id: root

    property real value: 0
    property real maximum: 1
    property bool interactive: true

    signal seekTo(real ratio)

    implicitHeight: 16

    Rectangle {
        id: track
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: (hover.hovered || drag.pressed) ? 4 : 3
        radius: height / 2
        color: "#24FFFFFF"
        Behavior on height { NumberAnimation { duration: 100 } }

        Rectangle {
            width: parent.width * (root.maximum > 0
                                   ? Math.max(0, Math.min(1, root.value / root.maximum)) : 0)
            height: parent.height
            radius: parent.radius
            color: Theme.accent
        }
    }

    Rectangle {
        id: knob
        width: 10
        height: 10
        radius: 5
        color: "white"
        visible: hover.hovered || drag.pressed
        x: Math.max(0, Math.min(root.width - width,
                                root.width * (root.maximum > 0
                                              ? Math.max(0, Math.min(1, root.value / root.maximum)) : 0)
                                - width / 2))
        anchors.verticalCenter: parent.verticalCenter
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    MouseArea {
        id: drag
        anchors.fill: parent
        enabled: root.interactive
        onPressed: function (mouse) {
            root.seekTo(Math.max(0, Math.min(1, mouse.x / width)));
        }
        onPositionChanged: function (mouse) {
            if (pressed)
                root.seekTo(Math.max(0, Math.min(1, mouse.x / width)));
        }
    }
}
