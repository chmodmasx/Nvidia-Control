#include "TelemetryBridge.h"

#include <QDBusConnection>
#include <QDBusMessage>
#include <QDBusPendingCallWatcher>
#include <QDBusPendingReply>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>
#include <QVariant>

namespace {
constexpr int PollIntervalMs = 1000;
constexpr int InventoryRefreshMs = 60000;
constexpr int DBusTimeoutMs = 5000;

QDBusMessage makeCall(const QString &method)
{
    return QDBusMessage::createMethodCall(
        QStringLiteral("io.github.chmodmasx.NvidiaControl"),
        QStringLiteral("/io/github/chmodmasx/NvidiaControl"),
        QStringLiteral("io.github.chmodmasx.NvidiaControl1"),
        method
    );
}
}

TelemetryBridge::TelemetryBridge(QObject *parent) : QObject(parent)
{
    m_timer.setInterval(PollIntervalMs);
    connect(&m_timer, &QTimer::timeout, this, &TelemetryBridge::refresh);
    m_timer.start();
    QTimer::singleShot(0, this, &TelemetryBridge::refresh);
}

void TelemetryBridge::refresh()
{
    if (!QDBusConnection::sessionBus().isConnected()) {
        setDisconnected(QStringLiteral("Bus D-Bus de sesión no disponible"));
        return;
    }

    // Retry discovery automatically if there is no inventory. The daemon
    // caches the slow read path and will only reprobe after the TTL expires.
    if (m_gpuUuid.isEmpty()) {
        requestInventory();
        return;
    }

    if (!m_inventoryAge.isValid() || m_inventoryAge.elapsed() >= InventoryRefreshMs) {
        requestInventory();
    }

    requestTelemetry();
}

void TelemetryBridge::requestInventory()
{
    if (m_pendingInventory) {
        return;
    }

    m_pendingInventory = true;
    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(makeCall(QStringLiteral("GetInventory")), DBusTimeoutMs),
        this
    );
    watcher->setProperty("generation", QVariant::fromValue(m_generation));
    connect(watcher, &QDBusPendingCallWatcher::finished,
            this, &TelemetryBridge::handleInventoryReply);
}

void TelemetryBridge::requestTelemetry()
{
    if (m_pendingTelemetry || m_gpuUuid.isEmpty()) {
        return;
    }

    QDBusMessage message = makeCall(QStringLiteral("GetTelemetry"));
    message << m_gpuIndex << m_gpuUuid;
    m_pendingTelemetry = true;

    auto *watcher = new QDBusPendingCallWatcher(
        QDBusConnection::sessionBus().asyncCall(message, DBusTimeoutMs),
        this
    );
    watcher->setProperty("generation", QVariant::fromValue(m_generation));
    watcher->setProperty("uuid", m_gpuUuid);
    connect(watcher, &QDBusPendingCallWatcher::finished,
            this, &TelemetryBridge::handleTelemetryReply);
}

void TelemetryBridge::handleInventoryReply(QDBusPendingCallWatcher *watcher)
{
    QDBusPendingReply<QString> reply = *watcher;
    const auto generation = watcher->property("generation").toULongLong();
    watcher->deleteLater();
    if (generation != m_generation) {
        return;  // Response from an older connection attempt.
    }
    m_pendingInventory = false;

    if (reply.isError()) {
        setDisconnected(reply.error().message());
        return;
    }

    QJsonParseError error;
    const QJsonDocument doc = QJsonDocument::fromJson(reply.value().toUtf8(), &error);
    if (error.error != QJsonParseError::NoError || !doc.isArray()) {
        setDisconnected(QStringLiteral("Inventario NVIDIA inválido"));
        return;
    }

    const QJsonArray list = doc.array();
    if (list.isEmpty()) {
        setDisconnected(QStringLiteral("No se detectaron GPUs NVIDIA"));
        return;
    }

    // Preserve the current GPU across re-enumeration, using its UUID rather
    // than relying on the index or ordering of the hardware enumeration.
    QJsonObject selected;
    for (const QJsonValue &item : list) {
        if (!item.isObject()) {
            continue;
        }
        const QJsonObject candidate = item.toObject();
        const QJsonObject id = candidate.value(QStringLiteral("device"))
                                   .toObject().value(QStringLiteral("id")).toObject();
        if (!m_gpuUuid.isEmpty() && id.value(QStringLiteral("uuid")).toString() == m_gpuUuid) {
            selected = candidate;
            break;
        }
        if (selected.isEmpty()) {
            selected = candidate;
        }
    }

    const QJsonValue gpu = selected.value(QStringLiteral("device"));
    const QJsonValue limits = selected.value(QStringLiteral("operating_limits"));
    const QJsonObject id = gpu.toObject().value(QStringLiteral("id")).toObject();
    const QString uuid = id.value(QStringLiteral("uuid")).toString();
    const QJsonValue index = id.value(QStringLiteral("index"));
    if (!gpu.isObject() || !limits.isObject() || uuid.isEmpty() || !index.isDouble()
        || index.toDouble() < 0 || index.toDouble() > 4294967295.0) {
        setDisconnected(QStringLiteral("Formato de inventario incompatible"));
        return;
    }

    const bool gpuChanged = uuid != m_gpuUuid;
    m_gpuUuid = uuid;
    m_gpuIndex = static_cast<quint32>(index.toDouble());
    m_device = gpu.toObject().toVariantMap();
    m_limits = limits.toObject().toVariantMap();
    m_inventoryAge.restart();

    if (gpuChanged && !m_telemetry.isEmpty()) {
        m_telemetry.clear();
    }
    emit snapshotChanged();

    if (gpuChanged) {
        if (m_connected) {
            m_connected = false;
            m_error = QStringLiteral("Actualizando sensores");
            emit statusChanged();
        }
    }

    // Fetch the first sensor reading immediately after discovering hardware.
    requestTelemetry();
}

void TelemetryBridge::handleTelemetryReply(QDBusPendingCallWatcher *watcher)
{
    QDBusPendingReply<QString> reply = *watcher;
    const auto generation = watcher->property("generation").toULongLong();
    const QString responseUuid = watcher->property("uuid").toString();
    watcher->deleteLater();

    if (generation != m_generation) {
        return;
    }
    m_pendingTelemetry = false;
    if (responseUuid != m_gpuUuid) {
        return;  // GPU selection changed while the request was in flight.
    }

    if (reply.isError()) {
        setDisconnected(reply.error().message());
        return;
    }

    QJsonParseError error;
    const QJsonDocument doc = QJsonDocument::fromJson(reply.value().toUtf8(), &error);
    if (error.error != QJsonParseError::NoError || !doc.isObject()) {
        setDisconnected(QStringLiteral("Telemetría NVIDIA inválida"));
        return;
    }

    m_telemetry = doc.object().toVariantMap();
    emit snapshotChanged();
    markConnected();
}

void TelemetryBridge::markConnected()
{
    if (!m_connected || !m_error.isEmpty()) {
        m_connected = true;
        m_error.clear();
        emit statusChanged();
    }
}

void TelemetryBridge::setDisconnected(const QString &message)
{
    // Invalidate both outstanding request types; old responses must never
    // repaint the UI after the daemon is restarted or unplugged.
    ++m_generation;
    m_pendingInventory = false;
    m_pendingTelemetry = false;
    m_inventoryAge.invalidate();
    m_gpuUuid.clear();
    m_gpuIndex = 0;

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
