import QtQuick

// Small arc spinner used while a stream is being resolved / first frame decoded.
Item {
    id: root

    property color color: "white"

    Canvas {
        id: canvas
        anchors.fill: parent

        onPaint: {
            const ctx = getContext("2d");
            ctx.reset();
            const radius = Math.min(width, height) / 2 - 2.5;
            if (radius <= 0)
                return;
            ctx.strokeStyle = root.color;
            ctx.lineWidth = 2.4;
            ctx.lineCap = "round";
            ctx.beginPath();
            ctx.arc(width / 2, height / 2, radius, 0, Math.PI * 1.45);
            ctx.stroke();
        }

        Connections {
            target: root
            function onColorChanged() { canvas.requestPaint(); }
            function onWidthChanged() { canvas.requestPaint(); }
            function onHeightChanged() { canvas.requestPaint(); }
        }

        Component.onCompleted: requestPaint()
    }

    RotationAnimator on rotation {
        from: 0
        to: 360
        duration: 850
        loops: Animation.Infinite
        running: root.visible
    }
}
