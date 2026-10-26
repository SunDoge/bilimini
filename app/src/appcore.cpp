#include "appcore.h"

#include <QCoreApplication>
#include <QDebug>
#include <QDir>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <QQuickWindow>
#include <QScreen>
#include <QCursor>
#include <QGuiApplication>
#include <QStandardPaths>

AppCore::AppCore(QObject *parent)
    : QObject(parent)
{
}

AppCore::~AppCore()
{
    if (m_core && m_core->state() != QProcess::NotRunning) {
        m_stopping = true;
        m_core->terminate();
        if (!m_core->waitForFinished(1500))
            m_core->kill();
    }
}

QString AppCore::locateCoreBinary() const
{
    const QString fromEnv = qEnvironmentVariable("BILI_CORE_BIN");
    if (!fromEnv.isEmpty())
        return fromEnv;

    const QString sibling =
        QCoreApplication::applicationDirPath() + QStringLiteral("/bili-core");
    if (QFileInfo::exists(sibling))
        return sibling;

    return QStandardPaths::findExecutable(QStringLiteral("bili-core"));
}

void AppCore::startCore()
{
    if (m_noCore || m_core)
        return;

    const QString binary = locateCoreBinary();
    if (binary.isEmpty()) {
        qWarning() << "bili-core binary not found; running without data service";
        setState(QStringLiteral("missing"));
        return;
    }

    m_core = new QProcess(this);
    m_core->setProgram(binary);
    m_core->setProcessChannelMode(QProcess::SeparateChannels);

    connect(m_core, &QProcess::readyReadStandardOutput, this, &AppCore::readStdout);
    connect(m_core, &QProcess::readyReadStandardError, this, [this] {
        const QByteArray err = m_core->readAllStandardError().trimmed();
        if (!err.isEmpty())
            qWarning().noquote() << "bili-core:" << err;
    });
    connect(m_core, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (m_stopping)
            return;
        qWarning() << "bili-core process error:" << error;
        setState(QStringLiteral("error"));
    });
    connect(m_core, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished), this,
            [this](int code, QProcess::ExitStatus status) {
                if (m_stopping)
                    return;
                qWarning() << "bili-core exited:" << code << status;
                setState(QStringLiteral("exited"));
            });

    qInfo() << "starting bili-core:" << binary;
    m_core->start();
}

void AppCore::readStdout()
{
    while (m_core->canReadLine()) {
        const QByteArray line = m_core->readLine().trimmed();
        if (line.isEmpty())
            continue;

        const QJsonDocument doc = QJsonDocument::fromJson(line);
        if (!doc.isObject())
            continue;

        const QJsonObject obj = doc.object();
        const int port = obj.value(QStringLiteral("port")).toInt();
        const QString token = obj.value(QStringLiteral("token")).toString();
        if (port <= 0 || token.isEmpty())
            continue;

        m_coreUrl = QStringLiteral("http://127.0.0.1:%1").arg(port);
        m_coreToken = token;
        setState(QStringLiteral("ready"));
        qInfo() << "bili-core ready at" << m_coreUrl;
    }
}

void AppCore::setState(const QString &state)
{
    if (m_coreState == state)
        return;
    m_coreState = state;
    emit coreChanged();
}

void AppCore::startSystemMove(QObject *window)
{
    if (auto *quickWindow = qobject_cast<QQuickWindow *>(window))
        quickWindow->startSystemMove();
}

QPoint AppCore::cursorPos() const
{
    return QCursor::pos();
}

void AppCore::pushCursor(int shape)
{
    QGuiApplication::setOverrideCursor(QCursor(Qt::CursorShape(shape)));
}

void AppCore::popCursor()
{
    QGuiApplication::restoreOverrideCursor();
}

void AppCore::setWindowGeometry(QObject *window, int x, int y, int w, int h)
{
    if (auto *quickWindow = qobject_cast<QQuickWindow *>(window))
        quickWindow->setGeometry(x, y, w, h);
}

void AppCore::startSystemResize(QObject *window, int edges)
{
    if (auto *quickWindow = qobject_cast<QQuickWindow *>(window))
        quickWindow->startSystemResize(Qt::Edges(edges));
}

bool AppCore::placeWindow(QObject *window, int pad)
{
    auto *quickWindow = qobject_cast<QQuickWindow *>(window);
    if (!quickWindow)
        return false;
    if (quickWindow->visibility() == QWindow::FullScreen)
        return false;

    QScreen *screen = quickWindow->screen();
    if (!screen)
        return false;

    const QRect area = screen->availableGeometry();
    if (area.isEmpty())
        return false;

    // `pad` is the transparent margin reserved for the drop shadow; the
    // visible frame should sit 16px away from the screen edges.
    const int margin = 16 - pad;
    quickWindow->setPosition(area.x() + area.width() - quickWindow->width() - margin,
                             area.y() + area.height() - quickWindow->height() - margin);
    return true;
}

void AppCore::quit()
{
    QCoreApplication::quit();
}
