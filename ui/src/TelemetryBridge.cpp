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

bool TelemetryBridge::selectGpu(const QString &uuid)
{
    if (uuid == m_gpuUuid || !m_catalog.select(uuid)) {
        return uuid == m_gpuUuid && !uuid.isEmpty();
    }

    applySelectedGpu();
    requestTelemetry(); // Do not wait for the next timer tick after switching.
    return true;
}

void TelemetryBridge::applySelectedGpu()
{
    const GpuCatalog::Entry *entry = m_catalog.selected();
    if (!entry) {
        return;
    }

    const bool changed = entry->uuid != m_gpuUuid;
    const bool reindexed = entry->index != m_gpuIndex;

    if (changed || reindexed) {
        // Discard both in-flight request types. A late response from another
        // GPU must never reset m_pendingTelemetry for the newly selected GPU.
        ++m_generation;
        m_pendingInventory = false;
        m_pendingTelemetry = false;
    }

    if (!m_lastObservedUuid.isEmpty() && entry->uuid != m_lastObservedUuid) {
        emit deviceChanged(); // Clears history, never mixes GPU time series.
    }

    m_lastObservedUuid = entry->uuid;
    m_gpuUuid = entry->uuid;
    m_gpuIndex = entry->index;
    m_device = entry->device;
    m_limits = entry->limits;

    if (changed || reindexed) {
        m_telemetry.clear();
        if (m_connected || m_error != QStringLiteral("Actualizando sensores")) {
            m_connected = false;
            m_error = QStringLiteral("Actualizando sensores");
            emit statusChanged();
        }
    }

    emit inventoryChanged();
    emit snapshotChanged();
}

void TelemetryBridge::handleInventoryReply(QDBusPendingCallWatcher *watcher)
{
    QDBusPendingReply<QString> reply = *watcher;
    const auto generation = watcher->property("generation").toULongLong();
    watcher->deleteLater();
    if (generation != m_generation) {
        return;
    }
    m_pendingInventory = false;

    if (reply.isError()) {
        setDisconnected(reply.error().message());
        return;
    }

    QJsonParseError error;
    const QJsonDocument doc = QJsonDocument::fromJson(reply.value().toUtf8(), &error);
    if (error.error != QJsonParseError::NoError || !doc.isArray()
        || !m_catalog.update(doc.array())) {
        setDisconnected(QStringLiteral("Inventario NVIDIA inválido"));
        return;
    }

    // The catalogue preserves selected UUID through reorder; if that UUID
    // vanishes it falls back to the first remaining valid device.
    applySelectedGpu();
    m_inventoryAge.restart();

    if (m_telemetry.isEmpty()) {
        requestTelemetry();
    }
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
        return;
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
    emit telemetryReceived(m_telemetry);
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
    ++m_generation;
    m_pendingInventory = false;
    m_pendingTelemetry = false;
    m_inventoryAge.invalidate();
    m_catalog.clear(); // Keep preferred UUID for reconnecting to same GPU.
    m_gpuUuid.clear();
    m_gpuIndex = 0;
    emit inventoryChanged();

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
