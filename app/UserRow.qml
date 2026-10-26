import QtQuick

// UP主 search result row: avatar, name, fans / video count, signature.
Item {
    id: root

    property var user: null

    signal clicked()

    width: parent ? parent.width : 0
    height: 64

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusSmall
        color: hover.hovered ? "#14FFFFFF" : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }
    }

    Rectangle {
        id: avatar
        x: 6
        anchors.verticalCenter: parent.verticalCenter
        width: 44
        height: 44
        radius: 8
        color: Theme.card
        clip: true

        Image {
            anchors.fill: parent
            source: root.user && root.user.avatar ? root.user.avatar : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
        }
    }

    Column {
        anchors {
            left: avatar.right
            leftMargin: 10
            right: parent.right
            rightMargin: 8
            verticalCenter: parent.verticalCenter
        }
        spacing: 3

        Text {
            width: parent.width
            text: root.user && root.user.name ? root.user.name : ""
            color: Theme.text
            font.pixelSize: 13
            font.bold: true
            elide: Text.ElideRight
            maximumLineCount: 1
        }

        Text {
            width: parent.width
            text: root.user ? ("粉丝 " + Theme.fmtViews(root.user.fans || 0) + " · 视频 " + (root.user.videos || 0)) : ""
            color: Theme.textFaint
            font.pixelSize: 11
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.user ? (root.user.sign || "") : ""
            color: Theme.textMuted
            font.pixelSize: 11
            elide: Text.ElideRight
            maximumLineCount: 1
        }
    }

    HoverHandler { id: hover }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
