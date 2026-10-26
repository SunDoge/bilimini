#pragma once

#include <QObject>
#include <QPoint>
#include <QString>

class QProcess;

// Exposed to QML as the `AppCore` singleton.
// Owns the bili-core sidecar process and a couple of window helpers.
class AppCore : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString coreUrl READ coreUrl NOTIFY coreChanged)
    Q_PROPERTY(QString coreToken READ coreToken NOTIFY coreChanged)
    Q_PROPERTY(QString coreState READ coreState NOTIFY coreChanged)
    Q_PROPERTY(QString initialMode READ initialMode CONSTANT)
    Q_PROPERTY(QString initialView READ initialView CONSTANT)
    Q_PROPERTY(int initialItem READ initialItem CONSTANT)
    Q_PROPERTY(bool screenshotMode READ screenshotMode CONSTANT)
    Q_PROPERTY(QString initialSearch READ initialSearch CONSTANT)
    Q_PROPERTY(QString initialSearchType READ initialSearchType CONSTANT)
    Q_PROPERTY(bool initialAutoplay READ initialAutoplay CONSTANT)
    Q_PROPERTY(QString initialPlayBvid READ initialPlayBvid CONSTANT)
    Q_PROPERTY(bool stayExpanded READ stayExpanded CONSTANT)
    Q_PROPERTY(bool browseAfterPlay READ browseAfterPlay CONSTANT)

public:
    explicit AppCore(QObject *parent = nullptr);
    ~AppCore() override;

    QString coreUrl() const { return m_coreUrl; }
    QString coreToken() const { return m_coreToken; }
    QString coreState() const { return m_coreState; }
    QString initialMode() const { return m_initialMode; }
    QString initialView() const { return m_initialView; }
    int initialItem() const { return m_initialItem; }
    bool screenshotMode() const { return m_screenshotMode; }
    QString initialSearch() const { return m_initialSearch; }
    QString initialSearchType() const { return m_initialSearchType; }
    bool initialAutoplay() const { return m_initialAutoplay; }
    QString initialPlayBvid() const { return m_initialPlayBvid; }
    bool stayExpanded() const { return m_stayExpanded; }
    bool browseAfterPlay() const { return m_browseAfterPlay; }

    void setInitialMode(const QString &mode) { m_initialMode = mode; }
    void setInitialView(const QString &view) { m_initialView = view; }
    void setInitialItem(int item) { m_initialItem = item; }
    void setScreenshotMode(bool on) { m_screenshotMode = on; }
    void setInitialSearch(const QString &search) { m_initialSearch = search; }
    void setInitialSearchType(const QString &type) { m_initialSearchType = type; }
    void setInitialAutoplay(bool on) { m_initialAutoplay = on; }
    void setInitialPlayBvid(const QString &bvid) { m_initialPlayBvid = bvid; }
    void setStayExpanded(bool on) { m_stayExpanded = on; }
    void setBrowseAfterPlay(bool on) { m_browseAfterPlay = on; }
    void setNoCore(bool on) { m_noCore = on; }

    void startCore();

    // Frameless windows: let the compositor move the window while dragging.
    Q_INVOKABLE void startSystemMove(QObject *window);
    Q_INVOKABLE void startSystemResize(QObject *window, int edges);
    Q_INVOKABLE QPoint cursorPos() const;
    // Locks the cursor shape while a QML-driven resize is in progress.
    Q_INVOKABLE void pushCursor(int shape);
    Q_INVOKABLE void popCursor();
    // Applies position + size in one configure request (smoother resize).
    Q_INVOKABLE void setWindowGeometry(QObject *window, int x, int y, int w, int h);

    // Places `window` at the bottom-right of its screen's available geometry
    // (i.e. above panels). Returns false while the platform window has no
    // screen yet, so QML can retry.
    Q_INVOKABLE bool placeWindow(QObject *window, int pad);
    Q_INVOKABLE void quit();

signals:
    void coreChanged();
    void showRequested();

private:
    void readStdout();
    void setState(const QString &state);
    QString locateCoreBinary() const;

    QProcess *m_core = nullptr;
    QString m_coreUrl;
    QString m_coreToken;
    QString m_coreState = QStringLiteral("starting");
    QString m_initialMode = QStringLiteral("expanded");
    QString m_initialView = QStringLiteral("browse");
    int m_initialItem = -1;
    bool m_screenshotMode = false;
    QString m_initialSearch;
    QString m_initialSearchType;
    bool m_initialAutoplay = false;
    QString m_initialPlayBvid;
    bool m_stayExpanded = false;
    bool m_browseAfterPlay = false;
    bool m_noCore = false;
    bool m_stopping = false;
};
