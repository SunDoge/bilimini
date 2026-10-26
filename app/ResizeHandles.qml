import QtQuick

// Frameless resize grips used by the expanded window: four corners only, so
// the edges stay free for dragging the window. Resizing is done in QML rather
// than with QWindow::startSystemResize(), because on this setup the window
// manager sometimes treated a corner press as a move request.
Item {
    id: root

    property var host: null
    property int cornerSize: 14

    MouseArea {
        width: root.cornerSize
        height: root.cornerSize
        anchors {
            left: parent.left
            top: parent.top
        }
        cursorShape: Qt.SizeFDiagCursor
        preventStealing: true

        property int xSign: -1
        property int ySign: -1

        onPressed: root.host.beginResize(xSign, ySign)
        onReleased: root.host.endResize()
        onCanceled: root.host.endResize()
    }

    MouseArea {
        width: root.cornerSize
        height: root.cornerSize
        anchors {
            right: parent.right
            top: parent.top
        }
        cursorShape: Qt.SizeBDiagCursor
        preventStealing: true

        property int xSign: 1
        property int ySign: -1

        onPressed: root.host.beginResize(xSign, ySign)
        onReleased: root.host.endResize()
        onCanceled: root.host.endResize()
    }

    MouseArea {
        width: root.cornerSize
        height: root.cornerSize
        anchors {
            left: parent.left
            bottom: parent.bottom
        }
        cursorShape: Qt.SizeBDiagCursor
        preventStealing: true

        property int xSign: -1
        property int ySign: 1

        onPressed: root.host.beginResize(xSign, ySign)
        onReleased: root.host.endResize()
        onCanceled: root.host.endResize()
    }

    MouseArea {
        width: root.cornerSize
        height: root.cornerSize
        anchors {
            right: parent.right
            bottom: parent.bottom
        }
        cursorShape: Qt.SizeFDiagCursor
        preventStealing: true

        property int xSign: 1
        property int ySign: 1

        onPressed: root.host.beginResize(xSign, ySign)
        onReleased: root.host.endResize()
        onCanceled: root.host.endResize()
    }
}
