#include "GpuCatalog.h"

#include <QJsonObject>
#include <QSet>
#include <cmath>
#include <limits>

bool GpuCatalog::update(const QJsonArray &array)
{
    if (array.isEmpty()) {
        return false;
    }

    QVector<Entry> entries;
    QSet<QString> uuids;
    entries.reserve(array.size());

    // Reject incomplete/duplicated inventories atomically; never show a
    // partially validated list or map telemetry to an ambiguous identifier.
    for (const auto &value : array) {
        if (!value.isObject()) {
            return false;
        }

        const QJsonObject report = value.toObject();
        const QJsonValue deviceValue = report.value(QStringLiteral("device"));
        const QJsonValue limitsValue = report.value(QStringLiteral("operating_limits"));
        if (!deviceValue.isObject() || !limitsValue.isObject()) {
            return false;
        }

        const QJsonObject device = deviceValue.toObject();
        const QJsonObject identity = device.value(QStringLiteral("id")).toObject();
        const QString uuid = identity.value(QStringLiteral("uuid")).toString();
        const QJsonValue indexValue = identity.value(QStringLiteral("index"));
        const double rawIndex = indexValue.toDouble(-1.0);

        if (uuid.isEmpty() || uuids.contains(uuid) || !indexValue.isDouble()
            || !std::isfinite(rawIndex) || rawIndex < 0
            || rawIndex > std::numeric_limits<quint32>::max()
            || std::floor(rawIndex) != rawIndex) {
            return false;
        }

        uuids.insert(uuid);
        Entry entry;
        entry.uuid = uuid;
        entry.index = static_cast<quint32>(rawIndex);
        entry.name = device.value(QStringLiteral("name")).toString();
        if (entry.name.isEmpty()) {
            entry.name = QStringLiteral("GPU %1").arg(entry.index);
        }
        entry.device = device.toVariantMap();
        entry.limits = limitsValue.toObject().toVariantMap();
        entries.append(entry);
    }

    m_entries = std::move(entries);
    const QString wanted = m_selectedUuid.isEmpty() ? m_preferredUuid : m_selectedUuid;
    m_selectedUuid.clear();

    for (const auto &entry : m_entries) {
        if (entry.uuid == wanted) {
            m_selectedUuid = wanted;
            break;
        }
    }
    if (m_selectedUuid.isEmpty()) {
        m_selectedUuid = m_entries.first().uuid;
    }
    m_preferredUuid = m_selectedUuid;
    return true;
}

bool GpuCatalog::select(const QString &uuid)
{
    for (const auto &entry : m_entries) {
        if (entry.uuid == uuid) {
            m_selectedUuid = uuid;
            m_preferredUuid = uuid;
            return true;
        }
    }
    return false;
}

const GpuCatalog::Entry *GpuCatalog::selected() const
{
    for (const auto &entry : m_entries) {
        if (entry.uuid == m_selectedUuid) {
            return &entry;
        }
    }
    return nullptr;
}

QVariantList GpuCatalog::options() const
{
    QVariantList options;
    for (const auto &entry : m_entries) {
        options.append(QVariantMap{
            {QStringLiteral("uuid"), entry.uuid},
            {QStringLiteral("index"), entry.index},
            {QStringLiteral("name"), entry.name},
            {QStringLiteral("label"), QStringLiteral("%1 · GPU %2").arg(entry.name).arg(entry.index)}
        });
    }
    return options;
}

void GpuCatalog::clear()
{
    m_entries.clear();
    m_selectedUuid.clear();
}
