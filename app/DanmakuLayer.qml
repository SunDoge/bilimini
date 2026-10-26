import QtQuick

// Scrolling danmaku overlay: a fixed pool of Text items driven by the mpv
// playback clock. Comments are spawned when their timestamp is reached and
// move right-to-left until they leave the layer.
Item {
    id: root

    property var host: null
    property var items: host ? host.danmakuItems : []
    property real position: host ? host.position : 0
    property bool active: host !== null && host.videoActive && host.danmakuEnabled

    visible: active
    clip: true

    readonly property int laneCount: Math.max(1, Math.min(9, Math.floor(height / 20)))
    readonly property real laneHeight: height / laneCount
    readonly property real travel: 8.0

    property real lastPos: 0
    property int nextIndex: 0
    property var lanesFree: []
    property var activeList: []

    onPositionChanged: tick()

    Repeater {
        id: pool
        model: 64

        delegate: Text {
            visible: false
            font.pixelSize: 12
            font.bold: true
            color: "white"
            style: Text.Outline
            styleColor: "#A6000000"
            textFormat: Text.PlainText
        }
    }

    function reset() {
        nextIndex = 0
        lastPos = 0
        activeList = []
        lanesFree = []
        for (let l = 0; l < laneCount; ++l)
            lanesFree.push(-1000000000)
        if (pool.count > 0) {
            for (let i = 0; i < pool.count; ++i)
                pool.itemAt(i).visible = false
        }
    }

    function colorOf(value) {
        if (!value)
            return "white";
        return Qt.rgba(((value >> 16) & 255) / 255,
                       ((value >> 8) & 255) / 255,
                       (value & 255) / 255, 1);
    }

    function spawn(item, now) {
        if (width < 40 || height < 20 || pool.count === 0)
            return;

        let slot = -1;
        for (let i = 0; i < pool.count; ++i) {
            if (!pool.itemAt(i).visible) {
                slot = i;
                break;
            }
        }
        if (slot < 0)
            return;

        let lane = -1;
        for (let l = 0; l < laneCount; ++l) {
            if (lanesFree[l] <= now) {
                lane = l;
                break;
            }
        }
        if (lane < 0)
            return;

        const text = pool.itemAt(slot);
        text.text = item.text;
        text.color = colorOf(item.color);
        text.visible = true;

        const textWidth = text.implicitWidth;
        const speed = (width + textWidth) / travel;
        text.x = width;
        text.y = lane * laneHeight + 2;

        const gap = 26;
        lanesFree[lane] = now + travel * (textWidth + gap) / (width + textWidth);
        activeList.push({ slot: slot, start: now, y: text.y, speed: speed });
    }

    function tick() {
        if (!active)
            return;

        const now = position;

        // Seek (or a big clock jump): rebuild the spawn cursor.
        if (now < lastPos - 0.6 || now > lastPos + 4) {
            reset();
            while (nextIndex < items.length && items[nextIndex].time < now - 0.2)
                nextIndex++;
        }
        lastPos = now;

        while (nextIndex < items.length && items[nextIndex].time <= now + 0.05) {
            const item = items[nextIndex++];
            if (item.time >= now - 0.4 && (item.mode === 1 || item.mode === 4 || item.mode === 5))
                spawn(item, now);
        }

        for (let i = activeList.length - 1; i >= 0; --i) {
            const entry = activeList[i];
            const text = pool.itemAt(entry.slot);
            const x = width - (now - entry.start) * entry.speed;
            if (x + text.implicitWidth < 0) {
                text.visible = false;
                activeList.splice(i, 1);
                continue;
            }
            text.x = x;
            text.y = entry.y;
        }
    }

    onItemsChanged: reset()
    onActiveChanged: if (active) reset()
    Component.onCompleted: reset()
}
