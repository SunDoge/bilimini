#include "mpvitem.h"

#include <QDebug>
#include <QMetaObject>
#include <QOpenGLContext>
#include <QOpenGLFramebufferObject>
#include <QQuickWindow>


namespace {

constexpr const char *kReferer = "https://www.bilibili.com/";
constexpr const char *kUserAgent =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

void *glProcAddress(void *, const char *name)
{
    auto *context = QOpenGLContext::currentContext();
    return context ? reinterpret_cast<void *>(context->getProcAddress(name)) : nullptr;
}

} // namespace

MpvItem::MpvItem(QQuickItem *parent)
    : QQuickFramebufferObject(parent)
{
    m_mpv = mpv_create();
    if (!m_mpv) {
        setError(QStringLiteral("无法创建 mpv 实例"));
        return;
    }

    mpv_set_option_string(m_mpv, "vo", "libmpv");
    mpv_set_option_string(m_mpv, "hwdec", "no");
    mpv_set_option_string(m_mpv, "idle", "yes");
    mpv_set_option_string(m_mpv, "keep-open", "no");
    mpv_set_option_string(m_mpv, "osc", "no");
    mpv_set_option_string(m_mpv, "input-default-bindings", "no");
    mpv_set_option_string(m_mpv, "cache", "yes");
    mpv_set_option_string(m_mpv, "audio-client-name", "bili-mini");
    const QByteArray aoOverride = qgetenv("BILI_MPV_AO");
    if (!aoOverride.isEmpty())
        mpv_set_option_string(m_mpv, "ao", aoOverride.constData());

    mpv_set_option_string(m_mpv, "user-agent", kUserAgent);
    mpv_set_option_string(m_mpv, "referrer", kReferer);
    mpv_request_log_messages(m_mpv, "warn");

    if (mpv_initialize(m_mpv) < 0) {
        setError(QStringLiteral("mpv 初始化失败"));
        qWarning() << "mpv_initialize failed";
        return;
    }

    mpv_observe_property(m_mpv, 0, "time-pos", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "duration", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "pause", MPV_FORMAT_FLAG);
    mpv_observe_property(m_mpv, 0, "volume", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "seeking", MPV_FORMAT_FLAG);
    mpv_observe_property(m_mpv, 0, "paused-for-cache", MPV_FORMAT_FLAG);

    mpv_set_wakeup_callback(m_mpv, &MpvItem::wakeupCallback, this);

    m_ready = true;
    emit readyChanged();
}

MpvItem::~MpvItem()
{
    if (m_mpv) {
        mpv_set_wakeup_callback(m_mpv, nullptr, nullptr);
        // The render context is freed by the renderer on the render thread.
        mpv_terminate_destroy(m_mpv);
        m_mpv = nullptr;
    }
}

void MpvItem::wakeupCallback(void *ctx)
{
    auto *item = static_cast<MpvItem *>(ctx);
    QMetaObject::invokeMethod(item, [item] { item->processEvents(); }, Qt::QueuedConnection);
}

void MpvItem::processEvents()
{
    if (!m_mpv)
        return;

    while (true) {
        mpv_event *event = mpv_wait_event(m_mpv, 0);
        if (!event || event->event_id == MPV_EVENT_NONE)
            break;

        switch (event->event_id) {
        case MPV_EVENT_PROPERTY_CHANGE: {
            auto *property = static_cast<mpv_event_property *>(event->data);
            if (!property || !property->data)
                break;
            const QString name = QString::fromUtf8(property->name);
            if (name == QLatin1String("time-pos") && property->format == MPV_FORMAT_DOUBLE) {
                m_position = *static_cast<double *>(property->data);
                emit positionChanged();
            } else if (name == QLatin1String("duration") && property->format == MPV_FORMAT_DOUBLE) {
                m_duration = *static_cast<double *>(property->data);
                emit durationChanged();
            } else if (name == QLatin1String("pause") && property->format == MPV_FORMAT_FLAG) {
                m_paused = *static_cast<int *>(property->data) != 0;
                m_playing = !m_paused;
                emit playingChanged();
            } else if (name == QLatin1String("seeking") && property->format == MPV_FORMAT_FLAG) {
                m_seeking = *static_cast<int *>(property->data) != 0;
                updateBuffering();
            } else if (name == QLatin1String("paused-for-cache") && property->format == MPV_FORMAT_FLAG) {
                m_cachePaused = *static_cast<int *>(property->data) != 0;
                updateBuffering();
            } else if (name == QLatin1String("volume") && property->format == MPV_FORMAT_DOUBLE) {
                m_volume = *static_cast<double *>(property->data) / 100.0;
                emit volumeChanged();
            }
            break;
        }
        case MPV_EVENT_FILE_LOADED:
            m_endedSent = false;
            // A replacement load emits END_FILE(STOP) for the old file without
            // a pause-property change, so refresh the playing state here.
            m_playing = !m_paused;
            emit playingChanged();
            if (!m_pendingAudio.isEmpty()) {
                const QByteArray audio = m_pendingAudio.toUtf8();
                m_pendingAudio.clear();
                const char *addArgs[] = {"audio-add", audio.constData(), "select", nullptr};
                if (mpv_command(m_mpv, addArgs) < 0)
                    qWarning() << "mpv audio-add failed";
            }
            break;
        case MPV_EVENT_END_FILE: {
            auto *end = static_cast<mpv_event_end_file *>(event->data);
            if (!end)
                break;
            // STOP means "replaced by another loadfile"; the new file will
            // report FILE_LOADED and must not leave the UI paused.
            if (end->reason == MPV_END_FILE_REASON_EOF
                || end->reason == MPV_END_FILE_REASON_ERROR) {
                m_playing = false;
                emit playingChanged();
            }
            if (end->reason == MPV_END_FILE_REASON_EOF && !m_endedSent) {
                m_endedSent = true;
                emit ended();
            }
            break;
        }
        case MPV_EVENT_LOG_MESSAGE: {
            auto *message = static_cast<mpv_event_log_message *>(event->data);
            if (message)
                qWarning().noquote() << "mpv:" << message->prefix << message->text;
            break;
        }
        default:
            break;
        }
    }
}

void MpvItem::updateBuffering()
{
    const bool buffering = m_seeking || m_cachePaused;
    if (buffering == m_buffering)
        return;
    m_buffering = buffering;
    emit bufferingChanged();
}

void MpvItem::setError(const QString &message)
{
    if (m_error == message)
        return;
    m_error = message;
    emit errorChanged();
}

void MpvItem::setVolume(qreal volume)
{
    volume = qBound(0.0, volume, 1.0);
    if (qFuzzyCompare(volume, m_volume))
        return;
    m_volume = volume;
    if (m_mpv) {
        double value = volume * 100.0;
        mpv_set_property(m_mpv, "volume", MPV_FORMAT_DOUBLE, &value);
    }
    emit volumeChanged();
}

void MpvItem::play(const QString &videoUrl, const QString &audioUrl)
{
    if (!m_mpv || videoUrl.isEmpty())
        return;

    setError(QString());
    m_position = 0;
    m_duration = 0;
    m_endedSent = false;
    emit positionChanged();
    emit durationChanged();

    const QByteArray video = videoUrl.toUtf8();
    const char *loadArgs[] = {"loadfile", video.constData(), "replace", nullptr};
    const int result = mpv_command(m_mpv, loadArgs);
    if (result < 0) {
        setError(QStringLiteral("无法加载视频流"));
        qWarning() << "mpv loadfile failed:" << result;
        return;
    }

    // The audio is added on MPV_EVENT_FILE_LOADED; adding it before the file
    // is active fails, and a loadfile option breaks on commas in the URL.
    m_pendingAudio = audioUrl;

    mpv_set_property_string(m_mpv, "pause", "no");
}

void MpvItem::togglePause()
{
    if (!m_mpv)
        return;
    const bool paused = !m_playing;
    mpv_set_property_string(m_mpv, "pause", paused ? "no" : "yes");
}

void MpvItem::seek(qreal seconds)
{
    if (!m_mpv)
        return;
    double value = qMax(0.0, seconds);
    mpv_set_property(m_mpv, "time-pos", MPV_FORMAT_DOUBLE, &value);
}

void MpvItem::stop()
{
    if (!m_mpv)
        return;
    const char *args[] = {"stop", nullptr};
    mpv_command(m_mpv, args);
}

class MpvRenderer : public QQuickFramebufferObject::Renderer
{
public:
    explicit MpvRenderer(MpvItem *item)
        : m_item(item)
    {
    }

    ~MpvRenderer() override
    {
        if (m_item->m_render) {
            mpv_render_context_set_update_callback(m_item->m_render, nullptr, nullptr);
            mpv_render_context_free(m_item->m_render);
            m_item->m_render = nullptr;
        }
    }

    QOpenGLFramebufferObject *createFramebufferObject(const QSize &size) override
    {
        QOpenGLFramebufferObjectFormat format;
        format.setAttachment(QOpenGLFramebufferObject::CombinedDepthStencil);
        return new QOpenGLFramebufferObject(size, format);
    }

    void render() override
    {
        if (!m_item->m_mpv)
            return;

        if (!m_item->m_render) {
            mpv_opengl_init_params initParams{&glProcAddress, nullptr};
            // Advanced control requires report_swap() + update flags; leave it
            // off so every render() presents a frame.
            mpv_render_param params[] = {
                {MPV_RENDER_PARAM_API_TYPE, const_cast<char *>(MPV_RENDER_API_TYPE_OPENGL)},
                {MPV_RENDER_PARAM_OPENGL_INIT_PARAMS, &initParams},
                {MPV_RENDER_PARAM_INVALID, nullptr},
            };
            int ctx_rc = mpv_render_context_create(&m_item->m_render, m_item->m_mpv, params);
            if (ctx_rc < 0) {
                m_item->setError(QStringLiteral("mpv 渲染上下文创建失败"));
                return;
            }
            mpv_render_context_set_update_callback(
                m_item->m_render,
                [](void *ctx) {
                    auto *item = static_cast<MpvItem *>(ctx);
                    QMetaObject::invokeMethod(item, [item] { item->update(); }, Qt::QueuedConnection);
                },
                m_item);
        }

        QOpenGLFramebufferObject *fbo = framebufferObject();
        if (!fbo)
            return;
        mpv_opengl_fbo mpvFbo{fbo->handle(), fbo->width(), fbo->height()};
        int flipY = 0;
        mpv_render_param renderParams[] = {
            {MPV_RENDER_PARAM_OPENGL_FBO, &mpvFbo},
            {MPV_RENDER_PARAM_FLIP_Y, &flipY},
            {MPV_RENDER_PARAM_INVALID, nullptr},
        };
        mpv_render_context_render(m_item->m_render, renderParams);
    }

private:
    MpvItem *m_item;
};

QQuickFramebufferObject::Renderer *MpvItem::createRenderer() const
{
    return new MpvRenderer(const_cast<MpvItem *>(this));
}
