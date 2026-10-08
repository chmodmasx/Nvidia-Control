#include "TelemetryBridge.h"

#include <QDBusConnection>
#include <QDBusInterface>
#include <QDBusPendingCall>
#include <QDBusPendingCallWatcher>
#include <QDBusPendingReply>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>

TelemetryBridge::TelemetryBridge(QObject *parent) : QObject(parent)
{
    m_timer.setInterval(1000);
    connect(&m_timer, &QTimer::timeout, this, &TelemetryBridge::refresh);
    m_timer.start();
    QTimer::singleShot(0, this, &TelemetryBridge::refresh);
}

void TelemetryBridge::refresh()
{
    if (m_pending) {
        return;  // No overlapping requests if NVML or D-Bus is slow.
    }

    if (!QDBusConnection::sessionBus().isConnected()) {
        setDisconnected(QStringLiteral("Bus D-Bus de sesión no disponible"));
        return;
    }

    QDBusInterface iface(
        QStringLiteral("io.github.chmodmasx.NvidiaControl"),
        QStringLiteral("/io/github/chmodmasx/NvidiaControl"),
        QStringLiteral("io.github.chmodmasx.NvidiaControl1"),
        QDBusConnection::sessionBus()
    );

    if (!iface.isValid()) {
        setDisconnected(QStringLiteral("Servicio Nvidia-Control no disponible"));
        return;
    }

    m_pending = true;
    auto *watcher = new QDBusPendingCallWatcher(iface.asyncCall(QStringLiteral("GetSnapshot")), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this, &TelemetryBridge::handleReply);
}

void TelemetryBridge::handleReply(QDBusPendingCallWatcher *watcher)
{
    m_pending = false;
    QDBusPendingReply<QString> reply = *watcher;
    watcher->deleteLater();

    if (reply.isError()) {
        setDisconnected(reply.error().message());
        return;
    }

    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(reply.value().toUtf8(), &error);
    if (error.error != QJsonParseError::NoError || !document.isArray()) {
        setDisconnected(QStringLiteral("Respuesta de telemetría inválida"));
        return;
    }

    const QJsonArray devices = document.array();
    if (devices.isEmpty() || !devices.first().isObject()) {
        setDisconnected(QStringLiteral("No se detectaron GPUs NVIDIA"));
        return;
    }

    const QJsonObject item = devices.first().toObject();
    if (!item.value(QStringLiteral("device")).isObject()
        || !item.value(QStringLiteral("telemetry")).isObject()
        || !item.value(QStringLiteral("operating_limits")).isObject()) {
        setDisconnected(QStringLiteral("Formato de API incompatible"));
        return;
    }

    m_device = item.value(QStringLiteral("device")).toObject().toVariantMap();
    m_telemetry = item.value(QStringLiteral("telemetry")).toObject().toVariantMap();
    m_limits = item.value(QStringLiteral("operating_limits")).toObject().toVariantMap();
    emit snapshotChanged();

    if (!m_connected || !m_error.isEmpty()) {
        m_connected = true;
        m_error.clear();
        emit statusChanged();
    }
}

void TelemetryBridge::setDisconnected(const QString &message)
{
    if (!m_device.isEmpty() || !m_telemetry.isEmpty() || !m_limits.isEmpty()) {
        m_device.clear();
        m_telemetry.clear();
        m_limits.clear();
        emit snapshotChanged();
    }

    if (m_connected || m_error != message) {
        m_connected = false;
        m_error = message;
        emit statusChanged();
    }
}
