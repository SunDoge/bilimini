import QtQuick
import QtQuick.Layouts

Item {
    id: root

    property var host: null

    implicitHeight: 46

    SeekBar {
        id: seek
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            topMargin: 1
        }
        value: root.host ? root.host.position : 0
        maximum: root.host ? Math.max(1, root.host.duration) : 1
        onSeekTo: function (ratio) {
            if (root.host)
                root.host.seekToRatio(ratio);
        }
    }

    RowLayout {
        anchors {
            left: parent.left
            right: parent.right
            top: seek.bottom
            bottom: parent.bottom
        }
        spacing: 8

        IconButton {
            icon: "icons/prev.svg"
            tip: "上一个"
            size: 30
            onClicked: root.host.prevVideo()
        }

        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            width: 34
            height: 34
            radius: 17
            color: Theme.accent

            Image {
                anchors.centerIn: parent
                source: root.host && root.host.playing ? "icons/pause.svg" : "icons/play.svg"
                sourceSize: Qt.size(15, 15)
                width: 15
                height: 15
            }

            HoverHandler { cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: root.host.togglePlay() }
        }

        IconButton {
            icon: "icons/next.svg"
            tip: "下一个"
            size: 30
            onClicked: root.host.nextVideo()
        }

        Text {
            Layout.leftMargin: 6
            text: root.host
                  ? Theme.fmtTime(root.host.position) + " / " + Theme.fmtTime(root.host.duration)
                  : ""
            color: Theme.textMuted
            font.pixelSize: 11
        }

        Item { Layout.fillWidth: true }

        IconButton {
            icon: root.host && root.host.muted ? "icons/mute.svg" : "icons/volume.svg"
            tip: "静音 (M)"
            size: 28
            onClicked: root.host.toggleMute()
        }

        SeekBar {
            Layout.preferredWidth: 76
            Layout.alignment: Qt.AlignVCenter
            value: root.host ? (root.host.muted ? 0 : root.host.volume) : 0
            maximum: 1
            onSeekTo: function (ratio) {
                if (!root.host)
                    return;
                root.host.volume = ratio;
                if (root.host.muted && ratio > 0)
                    root.host.muted = false;
            }
        }

        Text {
            Layout.leftMargin: 2
            text: "1.0x"
            color: Theme.textFaint
            font.pixelSize: 11
        }

        IconButton {
            icon: "icons/fullscreen.svg"
            tip: "全屏 (F)"
            size: 30
            onClicked: root.host.toggleFullscreen()
        }
    }
}
