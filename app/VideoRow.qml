import QtQuick

// Compact horizontal card used by the browse list and the "related" list.
Item {
    id: root

    property var item: null
    property bool compact: false

    signal clicked()

    readonly property int thumbWidth: compact ? 112 : 144
    readonly property int thumbHeight: Math.round(thumbWidth * 9 / 16)

    width: parent ? parent.width : 0
    height: compact ? 70 : 88

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusSmall
        color: hover.hovered ? "#14FFFFFF" : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }
    }

    Rectangle {
        id: thumb
        x: 6
        anchors.verticalCenter: parent.verticalCenter
        width: root.thumbWidth
        height: root.thumbHeight
        radius: 6
        color: Theme.card
        clip: true

        MockCover {
            anchors.fill: parent
            radius: 6
            seed: Theme.seedFor(root.item)
            showGlyph: false
            visible: coverImage.status !== Image.Ready
        }

        Image {
            id: coverImage
            anchors.fill: parent
            source: root.item && root.item.cover ? root.item.cover : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
        }

        Rectangle {
            anchors {
                right: parent.right
                bottom: parent.bottom
                margins: 4
            }
            width: durationText.width + 8
            height: 16
            radius: 3
            color: "#B3000000"

            Text {
                id: durationText
                anchors.centerIn: parent
                text: root.item && root.item.duration ? Theme.fmtTime(root.item.duration) : ""
                color: "white"
                font.pixelSize: 9
            }
        }
    }

    Column {
        anchors {
            left: thumb.right
            leftMargin: 10
            right: parent.right
            rightMargin: 8
            verticalCenter: parent.verticalCenter
        }
        spacing: root.compact ? 4 : 6

        Text {
            width: parent.width
            text: root.item && root.item.title ? root.item.title : ""
            color: hover.hovered ? Theme.accent : Theme.text
            font.pixelSize: root.compact ? 12 : 13
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 1.3
        }

        Text {
            width: parent.width
            text: root.item && root.item.up ? (root.item.up + " · " + Theme.fmtViews(root.item.views || 0) + " 播放") : ""
            color: Theme.textFaint
            font.pixelSize: 11
            elide: Text.ElideRight
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TapHandler {
        onTapped: root.clicked()
    }
}
