import QtQuick
import QtQuick.Controls

Item {
    id: root

    property string icon: ""
    property string tip: ""
    property bool active: false

    signal clicked()

    width: 44
    height: 40

    Rectangle {
        anchors.fill: parent
        radius: 9
        color: root.active ? "#26FB7299" : (hover.hovered ? "#10FFFFFF" : "transparent")
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Image {
        anchors.centerIn: parent
        source: root.icon
        sourceSize: Qt.size(18, 18)
        width: 18
        height: 18
        fillMode: Image.PreserveAspectFit
        smooth: true
        opacity: root.active ? 1.0 : 0.6
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
    ToolTip.delay: 400
}
