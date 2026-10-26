import QtQuick
import QtQuick.Controls

// In-app UP主 page: header with the profile, then the uploaded videos.
Item {
    id: root

    property var host: null
    readonly property var user: host ? host.currentUser : null

    Item {
        id: header
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }
        // The profile column (name + fans + sign) is taller than the avatar;
        // size the header from the row so the list never overlaps the sign.
        height: backRow.height + 8 + infoRow.implicitHeight

        Row {
            id: backRow
            height: 18
            spacing: 4

            Item {
                width: 18
                height: 18

                Image {
                    anchors.fill: parent
                    source: "icons/back.svg"
                    sourceSize: Qt.size(14, 14)
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (root.host) root.host.closeUser()
                }
            }

            Text {
                text: "返回"
                color: Theme.textMuted
                font.pixelSize: 11
                anchors.verticalCenter: parent.verticalCenter
            }
        }

        Row {
            id: infoRow
            anchors {
                left: parent.left
                right: parent.right
                top: backRow.bottom
                topMargin: 8
            }
            spacing: 10

            Rectangle {
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
                width: parent.width - 54
                spacing: 3

                Text {
                    width: parent.width
                    text: root.user ? root.user.name : ""
                    color: Theme.text
                    font.pixelSize: 15
                    font.bold: true
                    elide: Text.ElideRight
                    maximumLineCount: 1
                }

                Text {
                    width: parent.width
                    text: root.user ? ("粉丝 " + Theme.fmtViews(root.user.fans || 0)
                                      + " · 视频 " + (root.user.videos || 0)) : ""
                    color: Theme.textFaint
                    font.pixelSize: 11
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
        }
    }

    Rectangle {
        anchors {
            left: parent.left
            right: parent.right
            top: header.bottom
            topMargin: 6
        }
        height: 1
        color: Theme.border
    }

    ListView {
        id: list
        anchors {
            left: parent.left
            right: parent.right
            top: header.bottom
            bottom: parent.bottom
            topMargin: 14
        }
        clip: true
        spacing: 4
        model: root.host ? root.host.userVideos : []
        boundsBehavior: Flickable.StopAtBounds

        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        delegate: VideoRow {
            width: ListView.view.width
            item: modelData
            onClicked: root.host.playItem(modelData, root.host.userVideos, index)
        }
    }

    Text {
        anchors.centerIn: list
        width: Math.min(implicitWidth, list.width - 24)
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.Wrap
        visible: list.count === 0
        text: root.host && root.host.userLoading ? "加载中…"
              : root.host && root.host.userError !== "" ? "加载失败：" + root.host.userError
              : "这个 UP 还没有投稿"
        color: root.host && root.host.userError !== "" ? Theme.bad : Theme.textFaint
        font.pixelSize: 12
    }
}
