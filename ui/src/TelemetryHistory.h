#pragma once

#include <QObject>
#include <QVariantList>
#include <QVariantMap>
#include <QVector>

// Frontend-only bounded telemetry history. No NVML, D-Bus or persistence.
class TelemetryHistory : public QObject
{
    Q_OBJECT
    Q_PROPERTY(int sampleCount READ sampleCount NOTIFY updated)

public:
    explicit TelemetryHistory(QObject *parent = nullptr);

    int sampleCount() const { return m_samples.size(); }

    // Returns chronological {t: epochMilliseconds, v: number|null} points.
    // Null values are kept so the graph never bridges missing sensor readings.
    Q_INVOKABLE QVariantList points(const QString &metric, int minutes) const;
    QVariantList pointsAt(const QString &metric, int minutes, qint64 nowMs) const;

    void addSampleAt(const QVariantMap &values, qint64 atMs);

public slots:
    void addSample(const QVariantMap &values);
    void clear();

signals:
    void updated();

private:
    struct Sample {
        qint64 timestampMs;
        QVariantMap values;
    };

    static constexpr qint64 HistoryDurationMs = 60LL * 60 * 1000;
    static constexpr int MaximumSamples = 3601;

    QVector<Sample> m_samples;
};
