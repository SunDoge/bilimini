import QtQuick
import QtQuick.Layouts

// One comment: avatar/name row, message, meta line.
Column {
    id: root

    property var comment: null

    spacing: 5

    RowLayout {
        width: parent.width
        spacing: 7

        Image {
            Layout.preferredWidth: 22
            Layout.preferredHeight: 22
            Layout.alignment: Qt.AlignTop
            source: root.comment && root.comment.avatar ? root.comment.avatar : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
        }

        Text {
            Layout.fillWidth: true
            text: root.comment && root.comment.name ? root.comment.name : ""
            color: Theme.accent
            font.pixelSize: 11
            elide: Text.ElideRight
            maximumLineCount: 1
        }

        Text {
            text: root.comment && root.comment.like > 0 ? Theme.fmtViews(root.comment.like) : ""
            color: Theme.textFaint
            font.pixelSize: 10
        }
    }

    Text {
        width: parent.width
        text: root.comment ? (root.comment.message || "") : ""
        color: Theme.text
        font.pixelSize: 12
        wrapMode: Text.Wrap
        lineHeight: 1.35
    }

    Text {
        visible: text !== ""
        text: {
            if (!root.comment)
                return "";
            const parts = [];
            if (root.comment.floor)
                parts.push("#" + root.comment.floor);
            if (root.comment.ctime)
                parts.push(Theme.fmtDate(root.comment.ctime));
            if (root.comment.replies > 0)
                parts.push(root.comment.replies + " 条回复");
            return parts.join(" · ");
        }
        color: Theme.textFaint
        font.pixelSize: 9
    }
}
