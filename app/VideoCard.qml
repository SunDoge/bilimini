import QtQuick

Item {
    id: root

    property var item: null
    property real cardWidth: 220

    signal clicked()

    width: cardWidth
    height: coverFrame.height + 62

    Rectangle {
        id: coverFrame
        width: parent.width
        height: Math.round(width * 9 / 16)
        radius: Theme.radiusSmall
        color: Theme.card
        clip: true

        MockCover {
            anchors.fill: parent
            radius: Theme.radiusSmall
            seed: root.item ? root.item.seed : 1
            showGlyph: false
            glyphSize: 32
        }

        Rectangle {
            anchors.fill: parent
            radius: Theme.radiusSmall
            color: hover.hovered ? "#1A000000" : "transparent"
            Behavior on color { ColorAnimation { duration: 120 } }
        }

        Image {
            anchors.centerIn: parent
            visible: hover.hovered
            source: "icons/play.svg"
            sourceSize: Qt.size(30, 30)
            width: 30
            height: 30
            opacity: 0.9
        }

        Rectangle {
            anchors {
                right: parent.right
                bottom: parent.bottom
                margins: 6
            }
            width: durationText.width + 10
            height: 18
            radius: 4
            color: "#B3000000"

            Text {
                id: durationText
                anchors.centerIn: parent
                text: root.item ? Theme.fmtTime(root.item.duration) : ""
                color: "white"
                font.pixelSize: 10
            }
        }
    }

    Text {
        id: titleText
        anchors {
            top: coverFrame.bottom
            topMargin: 8
            left: parent.left
            right: parent.right
        }
        height: font.pixelSize * 1.3 * 2
        text: root.item ? root.item.title : ""
        color: hover.hovered ? Theme.accent : Theme.text
        font.pixelSize: 13
        lineHeight: 1.3
        wrapMode: Text.Wrap
        maximumLineCount: 2
        elide: Text.ElideRight
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Text {
        anchors {
            top: titleText.bottom
            topMargin: 3
            left: parent.left
            right: parent.right
        }
        text: root.item ? (root.item.up + " · " + Theme.fmtViews(root.item.views) + " 播放") : ""
        color: Theme.textFaint
        font.pixelSize: 11
        elide: Text.ElideRight
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        onTapped: root.clicked()
    }
}
