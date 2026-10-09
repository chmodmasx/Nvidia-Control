#include "GpuCatalog.h"

#include <QJsonArray>
#include <QJsonObject>
#include <QTest>

namespace {
QJsonObject gpu(int index, const QString &uuid, const QString &name, int power)
{
    return QJsonObject{
        {"device", QJsonObject{
            {"id", QJsonObject{{"index", index}, {"uuid", uuid}}},
            {"name", name}
        }},
        {"operating_limits", QJsonObject{
            {"power", QJsonObject{{"current_watts", power}}}
        }}
    };
}
}

class CatalogTests : public QObject
{
    Q_OBJECT

private slots:
    void singleGpuHasOneStableOption();
    void switchUpdatesDeviceAndLimits();
    void preservesSelectionWhenIndicesReorder();
    void restoresSelectionAfterDisconnection();
    void fallsBackIfSelectedGpuDisappears();
    void rejectsUnknownSelectionAndMalformedInventory();
};

void CatalogTests::singleGpuHasOneStableOption()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{gpu(0, "GPU-MOCK-0000", "Card A", 250)}));
    QCOMPARE(catalog.size(), 1);
    QCOMPARE(catalog.selectedUuid(), QString("GPU-MOCK-0000"));
    QCOMPARE(catalog.options().size(), 1);
    QVERIFY(!catalog.select("unknown"));
}

void CatalogTests::switchUpdatesDeviceAndLimits()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{
        gpu(0, "GPU-MOCK-0000", "Card A", 250),
        gpu(1, "GPU-MOCK-0001", "Card B", 130)
    }));
    QVERIFY(catalog.select("GPU-MOCK-0001"));
    QVERIFY(catalog.selected() != nullptr);
    QCOMPARE(catalog.selected()->name, QString("Card B"));
    QCOMPARE(catalog.selected()->index, quint32(1));
    QCOMPARE(catalog.selected()->limits.value("power").toMap().value("current_watts").toInt(), 130);
}

void CatalogTests::preservesSelectionWhenIndicesReorder()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{
        gpu(0, "A", "Card A", 250), gpu(1, "B", "Card B", 130)
    }));
    QVERIFY(catalog.select("B"));
    QVERIFY(catalog.update(QJsonArray{
        gpu(0, "B", "Card B", 130), gpu(1, "A", "Card A", 250)
    }));
    QCOMPARE(catalog.selectedUuid(), QString("B"));
    QCOMPARE(catalog.selected()->index, quint32(0));
}

void CatalogTests::restoresSelectionAfterDisconnection()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{gpu(0, "A", "A", 250), gpu(1, "B", "B", 130)}));
    QVERIFY(catalog.select("B"));
    catalog.clear();
    QCOMPARE(catalog.size(), 0);
    QVERIFY(catalog.update(QJsonArray{gpu(0, "A", "A", 250), gpu(1, "B", "B", 130)}));
    QCOMPARE(catalog.selectedUuid(), QString("B"));
}

void CatalogTests::fallsBackIfSelectedGpuDisappears()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{gpu(0, "A", "A", 250), gpu(1, "B", "B", 130)}));
    QVERIFY(catalog.select("B"));
    QVERIFY(catalog.update(QJsonArray{gpu(0, "A", "A", 250)}));
    QCOMPARE(catalog.selectedUuid(), QString("A"));
    QCOMPARE(catalog.options().size(), 1);
}

void CatalogTests::rejectsUnknownSelectionAndMalformedInventory()
{
    GpuCatalog catalog;
    QVERIFY(catalog.update(QJsonArray{gpu(0, "A", "A", 250)}));
    QVERIFY(!catalog.select("B"));
    QVERIFY(!catalog.update(QJsonArray{
        gpu(0, "A", "A", 250), gpu(1, "A", "Duplicate", 100)
    }));
    QVERIFY(!catalog.update(QJsonArray{
        gpu(0, "A", "A", 250), QJsonObject{{"device", QJsonObject{}}}
    }));
    QCOMPARE(catalog.size(), 1);
    QCOMPARE(catalog.selectedUuid(), QString("A"));
}

QTEST_APPLESS_MAIN(CatalogTests)
#include "GpuCatalogTests.moc"
