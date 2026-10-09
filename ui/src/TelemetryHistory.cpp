#include "TelemetryHistory.h"

#include <QDateTime>
#include <QtGlobal>
#include <cmath>

TelemetryHistory::TelemetryHistory(QObject *parent) : QObject(parent) {}

void TelemetryHistory::addSample(const QVariantMap &values)
{
    addSampleAt(values, QDateTime::currentMSecsSinceEpoch());
}

void TelemetryHistory::addSampleAt(const QVariantMap &values, qint64 atMs)
{
    if (!m_samples.isEmpty() && atMs <= m_samples.last().timestampMs) {
        return; // Discard out-of-order or duplicated sensor observations.
    }

    m_samples.append({atMs, values});
    const qint64 cutoff = atMs - HistoryDurationMs;
    int expired = 0;
    while (expired < m_samples.size() && m_samples[expired].timestampMs < cutoff) {
        ++expired;
    }

    if (expired > 0) {
        m_samples.remove(0, expired);
    }
    if (m_samples.size() > MaximumSamples) {
        m_samples.remove(0, m_samples.size() - MaximumSamples);
    }

    emit updated();
}

QVariantList TelemetryHistory::points(const QString &metric, int minutes) const
{
    return pointsAt(metric, minutes, QDateTime::currentMSecsSinceEpoch());
}

QVariantList TelemetryHistory::pointsAt(const QString &metric, int minutes, qint64 nowMs) const
{
    const qint64 timeWindowMs = qBound(1, minutes, 60) * 60LL * 1000;
    const qint64 cutoff = nowMs - timeWindowMs;

    QVariantList result;
    for (const Sample &sample : m_samples) {
        if (sample.timestampMs < cutoff || sample.timestampMs > nowMs) {
            continue;
        }

        const QVariant raw = sample.values.value(metric);
        QVariant cleanValue;
        bool ok = false;
        const double number = raw.toDouble(&ok);
        if (raw.isValid() && !raw.isNull() && ok && std::isfinite(number)) {
            cleanValue = number;
            if (metric == QLatin1String("memory_used_bytes")) {
                cleanValue = number / 1073741824.0; // GiB
            }
        }

        result.append(QVariantMap{
            {QStringLiteral("t"), sample.timestampMs},
            {QStringLiteral("v"), cleanValue},
        });
    }
    return result;
}

void TelemetryHistory::clear()
{
    if (m_samples.isEmpty()) {
        return;
    }

    m_samples.clear();
    emit updated();
}
