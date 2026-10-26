import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Flickable {
    id: root

    property var host: null
    readonly property var item: host ? host.currentItem : null

    anchors.fill: parent
    contentWidth: width
    contentHeight: column.implicitHeight
    clip: true
    boundsBehavior: Flickable.StopAtBounds

    function reportVideoRect() {
        if (!root.host || root.host.mode === "mini" || root.host.view !== "player")
            return;
        const origin = videoBox.mapToItem(null, 0, 0);
        root.host.setVideoRect(origin.x - root.host.pad, origin.y - root.host.pad,
                               videoBox.width, videoBox.height);
    }

    Component.onCompleted: Qt.callLater(reportVideoRect)
    onWidthChanged: reportVideoRect()
    onHeightChanged: reportVideoRect()
    onContentYChanged: reportVideoRect()

    Connections {
        target: root.host
        function onVideoActiveChanged() { root.reportVideoRect(); }
        function onCurrentItemChanged() { Qt.callLater(root.reportVideoRect); }
    }

    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

    Column {
        id: column
        width: root.width
        spacing: 10

        Rectangle {
            id: videoBox
            onWidthChanged: root.reportVideoRect()
            onHeightChanged: root.reportVideoRect()
            width: parent.width
            height: Math.round(width * 9 / 16)
            radius: Theme.radiusSmall
            color: root.host && root.host.videoActive ? "transparent" : "#08090B"
            clip: true

            MockCover {
                anchors.fill: parent
                radius: Theme.radiusSmall
                seed: Theme.seedFor(root.item)
                showGlyph: false
                visible: !(root.host && root.host.videoActive) && coverImage.status !== Image.Ready
            }

            Image {
                id: coverImage
                anchors.fill: parent
                source: root.item && root.item.cover ? root.item.cover : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: true
                visible: !(root.host && root.host.videoActive)
            }

            DanmakuLayer {
                anchors.fill: parent
                host: root.host
            }

            Item {
                anchors.centerIn: parent
                width: 52
                height: 52
                visible: root.host && (root.host.loadingStream || root.host.bufferingSpinner)
                z: 5

                Rectangle {
                    anchors.fill: parent
                    radius: width / 2
                    color: "#A6000000"
                    border.width: 1
                    border.color: "#33FFFFFF"
                }

                Spinner {
                    anchors.centerIn: parent
                    width: 24
                    height: 24
                    color: "white"
                }
            }


            Rectangle {
                visible: root.host && !root.host.playing
                anchors.centerIn: parent
                width: 52
                height: 52
                radius: 26
                color: "#99000000"
                border.width: 1
                border.color: "#33FFFFFF"

                Image {
                    anchors.centerIn: parent
                    source: "icons/play.svg"
                    sourceSize: Qt.size(18, 18)
                    width: 18
                    height: 18
                }

                HoverHandler { cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: root.host.togglePlay() }
            }
        }

        ControlBar {
            width: parent.width
            host: root.host
        }

        Text {
            width: parent.width
            text: root.item ? root.item.title : ""
            color: Theme.text
            font.pixelSize: 16
            font.bold: true
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            lineHeight: 1.2
        }

        Row {
            spacing: 8

            Text {
                text: root.item ? root.item.up : ""
                color: Theme.accent
                font.pixelSize: 12
            }

            Text {
                text: root.item ? Theme.fmtViews(root.item.views) + " 播放" + (root.item.pubdate ? " · " + Theme.fmtDate(root.item.pubdate) : "") : ""
                color: Theme.textFaint
                font.pixelSize: 12
            }
        }

        Text {
            width: parent.width
            text: root.item ? (root.item.desc || "") : ""
            color: Theme.textMuted
            font.pixelSize: 12
            wrapMode: Text.Wrap
            maximumLineCount: 3
            elide: Text.ElideRight
            lineHeight: 1.6
        }

        Rectangle {
            width: parent.width
            height: 1
            color: Theme.border
        }

        RowLayout {
            width: parent.width
            spacing: 8

            Text {
                Layout.fillWidth: true
                text: root.host && root.host.commentCount > 0
                      ? "评论 <font color='" + Theme.textFaint + "'>" + root.host.commentCount + "</font>"
                      : "评论"
                textFormat: Text.RichText
                color: Theme.text
                font.pixelSize: 14
                font.bold: true
            }

            Row {
                spacing: 4

                Repeater {
                    model: [{ key: "hot", label: "热度" }, { key: "time", label: "时间" }]

                    delegate: Rectangle {
                        readonly property bool selected: root.host !== null && root.host.commentsSort === modelData.key
                        width: 44
                        height: 22
                        radius: 11
                        color: selected ? Theme.accentSoft : "transparent"
                        border.width: 1
                        border.color: selected ? Theme.accent : Theme.border

                        Text {
                            anchors.centerIn: parent
                            text: modelData.label
                            font.pixelSize: 10
                            color: selected ? Theme.accent : Theme.textMuted
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: if (root.host) root.host.setCommentSort(modelData.key)
                        }
                    }
                }
            }
        }

        Column {
            width: parent.width
            spacing: 14

            Repeater {
                model: root.host ? root.host.comments : []

                CommentRow {
                    width: parent.width
                    comment: modelData
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 32
            radius: 8
            visible: root.host && root.host.comments.length > 0 && !root.host.commentsEnd
            color: moreHover.hovered ? Theme.cardHover : Theme.card

            Text {
                anchors.centerIn: parent
                text: root.host && root.host.commentsMoreLoading ? "加载中…" : "查看更多评论"
                color: Theme.textMuted
                font.pixelSize: 11
            }

            HoverHandler { id: moreHover }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: if (root.host) root.host.loadMoreComments()
            }
        }

        Text {
            visible: root.host && root.host.commentsLoading
            text: "评论加载中…"
            color: Theme.textFaint
            font.pixelSize: 11
        }

        Text {
            width: parent.width
            wrapMode: Text.Wrap
            visible: root.host && !root.host.commentsLoading && root.host.comments.length === 0
            text: root.host && root.host.commentsError !== ""
                  ? "评论加载失败：" + root.host.commentsError
                  : "还没有评论"
            color: root.host && root.host.commentsError !== "" ? Theme.bad : Theme.textFaint
            font.pixelSize: 11
        }

        Rectangle {
            width: parent.width
            height: 1
            color: Theme.border
        }

        Text {
            text: "相关推荐"
            color: Theme.textMuted
            font.pixelSize: 12
        }

        Repeater {
            model: root.host ? root.host.relatedItems : []

            VideoRow {
                width: column.width
                compact: true
                item: modelData
                onClicked: root.host.playItem(modelData, root.host.relatedItems, index)
            }
        }

        Item {
            width: 1
            height: 8
        }
    }
}
