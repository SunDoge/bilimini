#pragma once

#include <QQuickFramebufferObject>
#include <QString>

#include <mpv/client.h>
#include <mpv/render_gl.h>

class MpvItem : public QQuickFramebufferObject
{
    Q_OBJECT
    Q_PROPERTY(qreal position READ position NOTIFY positionChanged)
    Q_PROPERTY(qreal duration READ duration NOTIFY durationChanged)
    Q_PROPERTY(bool playing READ playing NOTIFY playingChanged)
    Q_PROPERTY(qreal volume READ volume WRITE setVolume NOTIFY volumeChanged)
    Q_PROPERTY(bool ready READ ready NOTIFY readyChanged)
    Q_PROPERTY(bool buffering READ buffering NOTIFY bufferingChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    explicit MpvItem(QQuickItem *parent = nullptr);
    ~MpvItem() override;

    QQuickFramebufferObject::Renderer *createRenderer() const override;

    qreal position() const { return m_position; }
    qreal duration() const { return m_duration; }
    bool playing() const { return m_playing; }
    qreal volume() const { return m_volume; }
    bool ready() const { return m_ready; }
    bool buffering() const { return m_buffering; }
    QString error() const { return m_error; }

    void setVolume(qreal volume);

    Q_INVOKABLE void play(const QString &videoUrl, const QString &audioUrl);
    Q_INVOKABLE void togglePause();
    Q_INVOKABLE void seek(qreal seconds);
    Q_INVOKABLE void stop();

signals:
    void positionChanged();
    void durationChanged();
    void playingChanged();
    void volumeChanged();
    void readyChanged();
    void errorChanged();
    void bufferingChanged();
    void ended();

private:
    friend class MpvRenderer;

    void processEvents();
    void setError(const QString &message);
    void updateBuffering();
    static void wakeupCallback(void *ctx);

    mpv_handle *m_mpv = nullptr;
    mpv_render_context *m_render = nullptr;
    qreal m_position = 0;
    qreal m_duration = 0;
    qreal m_volume = 1.0;
    bool m_playing = false;
    bool m_paused = false;
    bool m_seeking = false;
    bool m_cachePaused = false;
    bool m_buffering = false;
    bool m_ready = false;
    bool m_endedSent = false;
    QString m_error;
    QString m_pendingAudio;
};
