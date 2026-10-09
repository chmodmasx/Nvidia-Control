#pragma once

#include <QJsonArray>
#include <QString>
#include <QVariantList>
#include <QVariantMap>
#include <QVector>

// Validates complete D-Bus inventory snapshots and remembers a stable UUID
// across re-enumeration, index reorder and temporary disconnection.
class GpuCatalog
{
public:
    struct Entry {
        QString uuid;
        QString name;
        quint32 index = 0;
        QVariantMap device;
        QVariantMap limits;
    };

    bool update(const QJsonArray &array);
    bool select(const QString &uuid);
    void clear(); // Retains preference, to restore on service reconnection.

    const Entry *selected() const;
    QString selectedUuid() const { return m_selectedUuid; }
    QVariantList options() const;
    int size() const { return m_entries.size(); }

private:
    QVector<Entry> m_entries;
    QString m_selectedUuid;
    QString m_preferredUuid;
};
