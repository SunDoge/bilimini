pragma Singleton

import QtQuick

QtObject {
    readonly property color bg: "#15161A"
    readonly property color bgRaised: "#1B1D22"
    readonly property color rail: "#111216"
    readonly property color card: "#1F2127"
    readonly property color cardHover: "#272A32"
    readonly property color text: "#ECEDF0"
    readonly property color textMuted: "#9BA1AC"
    readonly property color textFaint: "#6C7280"
    readonly property color accent: "#FB7299"
    readonly property color accentSoft: "#33FB7299"
    readonly property color border: "#1AFFFFFF"
    readonly property color ok: "#4ADE80"
    readonly property color warn: "#FBBF24"
    readonly property color bad: "#F87171"

    readonly property int radius: 12
    readonly property int radiusSmall: 8
    readonly property int pad: 16

    function fmtTime(seconds) {
        const total = Math.max(0, Math.floor(seconds));
        const h = Math.floor(total / 3600);
        const m = Math.floor((total % 3600) / 60);
        const s = total % 60;
        const mm = h > 0 ? String(m).padStart(2, "0") : String(m);
        return (h > 0 ? h + ":" : "") + mm + ":" + String(s).padStart(2, "0");
    }

    // Stable placeholder colour seed for items without a real cover yet.
    function seedFor(item) {
        if (!item)
            return 1;
        if (item.seed !== undefined)
            return item.seed;
        const text = (item.bvid || item.title || "");
        let hash = 0;
        for (let i = 0; i < text.length; ++i)
            hash = (hash * 31 + text.charCodeAt(i)) % 997;
        return hash;
    }

    function fmtDate(timestamp) {
        if (!timestamp)
            return "";
        const date = new Date(timestamp * 1000);
        return (date.getMonth() + 1) + "-" + date.getDate();
    }

    function fmtViews(views) {
        if (views >= 100000000)
            return (views / 100000000).toFixed(1) + " 亿";
        if (views >= 10000)
            return (views / 10000).toFixed(1) + " 万";
        return String(views);
    }
}
