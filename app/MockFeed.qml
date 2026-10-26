pragma Singleton

import QtQuick

// Placeholder data for M1 (real feed/search arrive with bili-core in M2).
// Shape is kept close to what /feed will return so the UI does not change.
QtObject {
    property var items: [
        {
            bvid: "BV1Ab411c7mD", cid: 1101, seed: 0, date: "10-24",
            title: "凌晨四点的便利店，和一锅关东煮",
            up: "深夜食堂", duration: 512, views: 128400,
            desc: "城市睡着以后，便利店是唯一还亮着的地方。这一期我们从一碗关东煮聊到深夜食堂的选址逻辑。"
        },
        {
            bvid: "BV1Bc411x7nE", cid: 1102, seed: 1, date: "10-23",
            title: "从零写一个玩具编译器：词法分析",
            up: "编译器实验室", duration: 1847, views: 96400,
            desc: "本期实现词法分析器：手写 scanner、处理数字与标识符、为后续语法分析准备 token 流。"
        },
        {
            bvid: "BV1Cd411y7oF", cid: 1103, seed: 2, date: "10-22",
            title: "海边日落 4K 延时，建议全屏",
            up: "光影研究所", duration: 236, views: 512300,
            desc: "拍摄于海边礁石区，三机位延时合成。建议开启 4K 并全屏观看。"
        },
        {
            bvid: "BV1De411z7pG", cid: 1104, seed: 3, date: "10-21",
            title: "三分钟看懂傅里叶变换到底在干嘛",
            up: "数学不挂科", duration: 342, views: 2210000,
            desc: "不推导公式，只讲直觉：为什么一个信号可以拆成一堆正弦波。"
        },
        {
            bvid: "BV1Ef411a7qH", cid: 1105, seed: 4, date: "10-20",
            title: "猫咪的一百种睡姿（第六种最离谱）",
            up: "橘猫日记", duration: 428, views: 780400,
            desc: "记录家里两只猫的日常。第六种睡姿建议反复观看。"
        },
        {
            bvid: "BV1Fg411b7rI", cid: 1106, seed: 5, date: "10-19",
            title: "手冲咖啡入门：先别买设备",
            up: "一个人喝咖啡", duration: 963, views: 332000,
            desc: "从水温、粉水比到注水手法，用最便宜的设备做出一杯能喝的咖啡。"
        },
        {
            bvid: "BV1Hh411c7sJ", cid: 1107, seed: 6, date: "10-18",
            title: "城市骑行 Vlog：沿着江边骑了 40 公里",
            up: "骑不动了", duration: 1214, views: 88000,
            desc: "周末沿着江边骑了四十公里，中途在桥洞下面吃了一碗面。"
        },
        {
            bvid: "BV1Ii411d7tK", cid: 1108, seed: 7, date: "10-17",
            title: "修复一台 1998 年的随身听",
            up: "老物件诊所", duration: 1526, views: 415000,
            desc: "皮带老化、电容漏液、齿轮开裂，看看二十多年前的工业设计有多讲究。"
        },
        {
            bvid: "BV1Jj411e7uL", cid: 1109, seed: 8, date: "10-16",
            title: "用 Rust 写个小游戏，顺便聊聊所有权",
            up: "代码美学", duration: 2104, views: 156000,
            desc: "从零实现一个终端小游戏，在真实代码里理解借用检查和生命周期。"
        },
        {
            bvid: "BV1Kk411f7vM", cid: 1110, seed: 9, date: "10-15",
            title: "月球背面到底有什么？",
            up: "看星星的人", duration: 734, views: 3120000,
            desc: "嫦娥四号在月球背面看到了什么？为什么我们永远看不到那一面？"
        },
        {
            bvid: "BV1Ll411g7wN", cid: 1111, seed: 10, date: "10-14",
            title: "雨夜钢琴即兴，一段没人听过的旋律",
            up: "琴房 403", duration: 389, views: 267000,
            desc: "下雨的晚上随手录的一段即兴，没有谱子，弹完就忘了。"
        },
        {
            bvid: "BV1Mm411h7xO", cid: 1112, seed: 11, date: "10-13",
            title: "拆解一台 2007 年的旗舰手机",
            up: "拆解爱好者", duration: 1655, views: 908000,
            desc: "滑盖、实体键盘、可换电池，一台十七年前的旗舰机内部长什么样。"
        }
    ]

    property var history: items.slice(0, 3)
}
