#pragma once

#include <QElapsedTimer>
#include <QObject>
#include <QTimer>
#include <QVariantMap>

class QDBusPendingCallWatcher;

// Thin, asynchronous presentation adapter. Dynamic telemetry is fetched every
// second; device capabilities and hardware limits are refreshed once a minute.
class TelemetryBridge : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool connected READ connected NOTIFY statusChanged)
    Q_PROPERTY(QString error READ error NOTIFY statusChanged)
    Q_PROPERTY(QVariantMap device READ device NOTIFY snapshotChanged)
    Q_PROPERTY(QVariantMap telemetry READ telemetry NOTIFY snapshotChanged)
    Q_PROPERTY(QVariantMap limits READ limits NOTIFY snapshotChanged)

public:
    explicit TelemetryBridge(QObject *parent = nullptr);

    bool connected() const { return m_connected; }
    QString error() const { return m_error; }
    QVariantMap device() const { return m_device; }
    QVariantMap telemetry() const { return m_telemetry; }
    QVariantMap limits() const { return m_limits; }

    Q_INVOKABLE void refresh();

signals:
    void snapshotChanged();
    void statusChanged();

private:
    void requestInventory();
    void requestTelemetry();
    void handleInventoryReply(QDBusPendingCallWatcher *watcher);
    void handleTelemetryReply(QDBusPendingCallWatcher *watcher);
    void setDisconnected(const QString &message);
    void markConnected();

    QTimer m_timer;
    QElapsedTimer m_inventoryAge;

    // Requests may overlap with one another, but never with another request of
    // their own kind. Generation/UUID guards discard replies after reconnects.
    quint64 m_generation = 0;
    bool m_pendingInventory = false;
    bool m_pendingTelemetry = false;
    bool m_connected = false;
    QString m_error = QStringLiteral("Servicio no conectado");
    QString m_gpuUuid;
    quint32 m_gpuIndex = 0;

    QVariantMap m_device;
    QVariantMap m_telemetry;
    QVariantMap m_limits;
};
