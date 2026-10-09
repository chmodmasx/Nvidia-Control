#pragma once

#include "GpuCatalog.h"

#include <QElapsedTimer>
#include <QObject>
#include <QTimer>
#include <QVariantList>
#include <QVariantMap>

class QDBusPendingCallWatcher;

// Presentation-side D-Bus adapter with a UUID-based multi-GPU catalog.
// Inventory is slow (60s), selected GPU telemetry is fast (1s).
class TelemetryBridge : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool connected READ connected NOTIFY statusChanged)
    Q_PROPERTY(QString error READ error NOTIFY statusChanged)
    Q_PROPERTY(QVariantMap device READ device NOTIFY snapshotChanged)
    Q_PROPERTY(QVariantMap telemetry READ telemetry NOTIFY snapshotChanged)
    Q_PROPERTY(QVariantMap limits READ limits NOTIFY snapshotChanged)
    Q_PROPERTY(QVariantList gpuOptions READ gpuOptions NOTIFY inventoryChanged)
    Q_PROPERTY(QString selectedGpuUuid READ selectedGpuUuid NOTIFY inventoryChanged)

public:
    explicit TelemetryBridge(QObject *parent = nullptr);

    bool connected() const { return m_connected; }
    QString error() const { return m_error; }
    QVariantMap device() const { return m_device; }
    QVariantMap telemetry() const { return m_telemetry; }
    QVariantMap limits() const { return m_limits; }
    QVariantList gpuOptions() const { return m_catalog.options(); }
    QString selectedGpuUuid() const { return m_catalog.selectedUuid(); }

    Q_INVOKABLE void refresh();
    Q_INVOKABLE bool selectGpu(const QString &uuid);

signals:
    void snapshotChanged();
    void statusChanged();
    void inventoryChanged();
    void telemetryReceived(const QVariantMap &values);
    void deviceChanged();

private:
    void requestInventory();
    void requestTelemetry();
    void handleInventoryReply(QDBusPendingCallWatcher *watcher);
    void handleTelemetryReply(QDBusPendingCallWatcher *watcher);
    void applySelectedGpu();
    void setDisconnected(const QString &message);
    void markConnected();

    QTimer m_timer;
    QElapsedTimer m_inventoryAge;
    GpuCatalog m_catalog;

    // Every pending async response carries a generation ID. A user GPU switch
    // or lost daemon invalidates older calls before they can update the UI.
    quint64 m_generation = 0;
    bool m_pendingInventory = false;
    bool m_pendingTelemetry = false;
    bool m_connected = false;
    QString m_error = QStringLiteral("Servicio no conectado");
    QString m_gpuUuid;
    QString m_lastObservedUuid;
    quint32 m_gpuIndex = 0;

    QVariantMap m_device;
    QVariantMap m_telemetry;
    QVariantMap m_limits;
};
