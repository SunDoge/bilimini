pragma Singleton

import QtQuick

// Thin wrapper around XMLHttpRequest for the local bili-core service.
QtObject {
    function request(path, ok, fail) {
        send("GET", path, ok, fail);
    }

    function post(path, ok, fail) {
        send("POST", path, ok, fail);
    }

    function send(method, path, ok, fail) {
        const xhr = new XMLHttpRequest();
        xhr.open(method, AppCore.coreUrl + path);
        xhr.setRequestHeader("Authorization", "Bearer " + AppCore.coreToken);
        xhr.timeout = 20000;

        xhr.onreadystatechange = function () {
            if (xhr.readyState !== XMLHttpRequest.DONE)
                return;
            if (xhr.status === 200) {
                try {
                    ok(JSON.parse(xhr.responseText));
                } catch (e) {
                    if (fail)
                        fail("解析响应失败: " + e);
                }
                return;
            }
            let message = "HTTP " + xhr.status;
            try {
                const parsed = JSON.parse(xhr.responseText);
                if (parsed && parsed.error)
                    message = parsed.error;
            } catch (e) {
                // keep the HTTP status message
            }
            if (fail)
                fail(message);
        };
        xhr.ontimeout = function () {
            if (fail)
                fail("请求超时");
        };
        xhr.send();
    }
}
