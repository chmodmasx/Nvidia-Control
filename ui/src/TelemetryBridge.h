#pragma once

#include <QObject>
#include <QTimer>
#include <QVariantMap>

class QDBusPendingCallWatcher;

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
    void handleReply(QDBusPendingCallWatcher *watcher);
    void setDisconnected(const QString &message);

    QTimer m_timer;
    bool m_pending = false;
    bool m_connected = false;
    QString m_error = QStringLiteral("Servicio no conectado");
    QVariantMap m_device;
    QVariantMap m_telemetry;
    QVariantMap m_limits;
};
