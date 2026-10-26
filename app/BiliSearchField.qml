import QtQuick
import QtQuick.Controls

Rectangle {
    id: root

    property alias text: field.text
    property alias placeholderText: field.placeholderText

    signal searchEdited(string value)

    implicitWidth: 280
    implicitHeight: 30
    radius: 15
    color: "#12FFFFFF"
    border.width: field.activeFocus ? 1 : 0
    border.color: Theme.accent

    Image {
        anchors {
            left: parent.left
            leftMargin: 10
            verticalCenter: parent.verticalCenter
        }
        source: "icons/search.svg"
        sourceSize: Qt.size(13, 13)
        width: 13
        height: 13
        opacity: 0.5
    }

    TextField {
        id: field
        anchors {
            left: parent.left
            leftMargin: 30
            right: parent.right
            rightMargin: 12
            top: parent.top
            bottom: parent.bottom
        }
        background: null
        color: Theme.text
        placeholderTextColor: Theme.textFaint
        font.pixelSize: 12
        verticalAlignment: TextInput.AlignVCenter
        selectByMouse: true
        onTextChanged: root.searchEdited(text)
    }
}
