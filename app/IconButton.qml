import QtQuick
import QtQuick.Controls

Item {
    id: root

    property string icon: ""
    property string tip: ""
    property int size: 28
    property int iconSize: 15
    property bool active: false
    property real iconOpacity: 0.85

    signal clicked()

    implicitWidth: size
    implicitHeight: size
    opacity: enabled ? 1.0 : 0.35

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: root.active ? Theme.accentSoft : (hover.hovered ? "#18FFFFFF" : "transparent")
        Behavior on color { ColorAnimation { duration: 110 } }
    }

    Image {
        anchors.centerIn: parent
        source: root.icon
        sourceSize: Qt.size(root.iconSize, root.iconSize)
        width: root.iconSize
        height: root.iconSize
        fillMode: Image.PreserveAspectFit
        smooth: true
        opacity: root.active ? 1.0 : root.iconOpacity
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        onTapped: root.clicked()
    }

    ToolTip.visible: hover.hovered && root.tip !== ""
    ToolTip.text: root.tip
    ToolTip.delay: 650
}
