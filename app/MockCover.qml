import QtQuick

// Fake video thumbnail. Deterministic hue from `seed` so the mock feed looks
// varied without shipping any artwork.
Rectangle {
    id: root

    property int seed: 1
    property bool showGlyph: true
    property real glyphSize: 36

    // Hand-picked pairs so the mock feed reads like colourful video stills
    // instead of muddy HSL gradients.
    readonly property var colorPairs: [
        ["#4B3FB0", "#241A5E", "#0D0A22"],
        ["#B03F6B", "#5E1A35", "#220A14"],
        ["#2F7FA8", "#123C52", "#081720"],
        ["#B07A2F", "#523210", "#1F1408"],
        ["#3FA87A", "#13523A", "#081B13"],
        ["#8A4CB0", "#3C1A52", "#160A20"],
        ["#4C6BD8", "#1A2A6B", "#0A0F24"],
        ["#B04C4C", "#521313", "#1F0808"]
    ]
    readonly property var colors: colorPairs[Math.abs(seed) % colorPairs.length]

    radius: 0
    clip: true

    gradient: Gradient {
        GradientStop { position: 0.0; color: root.colors[0] }
        GradientStop { position: 0.55; color: root.colors[1] }
        GradientStop { position: 1.0; color: root.colors[2] }
    }

    Rectangle {
        width: parent.width * 1.5
        height: parent.height * 0.55
        x: -parent.width * 0.25
        y: -parent.height * 0.2
        rotation: -14
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#22FFFFFF" }
            GradientStop { position: 1.0; color: "#00FFFFFF" }
        }
    }

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.55; color: "#00000000" }
            GradientStop { position: 1.0; color: "#66000000" }
        }
    }

    Image {
        visible: root.showGlyph
        anchors.centerIn: parent
        source: "icons/play.svg"
        sourceSize: Qt.size(root.glyphSize, root.glyphSize)
        width: root.glyphSize
        height: root.glyphSize
        opacity: 0.28
    }
}
