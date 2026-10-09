#include "TelemetryHistory.h"

#include <QSignalSpy>
#include <QTest>

class HistoryTests : public QObject
{
    Q_OBJECT

private slots:
    void retainsOnlyLastHour();
    void boundsMemoryForBurstTraffic();
    void selectsTimeWindows();
    void representsMissingAndInvalidReadingsAsGaps();
    void convertsVRAMToGiB();
    void discardsOutOfOrderSamples();
    void clearEmitsUpdate();
};

void HistoryTests::retainsOnlyLastHour()
{
    TelemetryHistory history;
    const qint64 start = 10000000;
    history.addSampleAt({{"temperature_c", 48.0}}, start);
    history.addSampleAt({{"temperature_c", 51.0}}, start + 3600001);
    QCOMPARE(history.sampleCount(), 1);
    QCOMPARE(history.pointsAt("temperature_c", 60, start + 3600001).length(), 1);
}

void HistoryTests::boundsMemoryForBurstTraffic()
{
    TelemetryHistory history;
    const qint64 start = 10000000;
    for (int i = 0; i < 5000; ++i) {
        history.addSampleAt({{"gpu_util_percent", i}}, start + i);
    }
    QCOMPARE(history.sampleCount(), 3601);
}

void HistoryTests::selectsTimeWindows()
{
    TelemetryHistory history;
    const qint64 now = 10000000;
    history.addSampleAt({{"power_watts", 200.0}}, now - 16 * 60000);
    history.addSampleAt({{"power_watts", 250.0}}, now - 10 * 60000);
    history.addSampleAt({{"power_watts", 300.0}}, now - 1 * 60000);
    QCOMPARE(history.pointsAt("power_watts", 5, now).size(), 1);
    QCOMPARE(history.pointsAt("power_watts", 15, now).size(), 2);
    QCOMPARE(history.pointsAt("power_watts", 60, now).size(), 3);
}

void HistoryTests::representsMissingAndInvalidReadingsAsGaps()
{
    TelemetryHistory history;
    const qint64 now = 10000000;
    history.addSampleAt({{"memory_temperature_c", 81.0}}, now - 2000);
    history.addSampleAt({{"memory_temperature_c", QVariant()}}, now - 1000);
    history.addSampleAt({{"memory_temperature_c", "not-a-number"}}, now);
    const auto points = history.pointsAt("memory_temperature_c", 5, now);
    QCOMPARE(points.size(), 3);
    QVERIFY(points.at(0).toMap().value("v").isValid());
    QVERIFY(!points.at(1).toMap().value("v").isValid());
    QVERIFY(!points.at(2).toMap().value("v").isValid());
}

void HistoryTests::convertsVRAMToGiB()
{
    TelemetryHistory history;
    const qint64 now = 10000000;
    history.addSampleAt({{"memory_used_bytes", 2LL * 1073741824}}, now);
    const auto points = history.pointsAt("memory_used_bytes", 5, now);
    QCOMPARE(points.at(0).toMap().value("v").toDouble(), 2.0);
}

void HistoryTests::discardsOutOfOrderSamples()
{
    TelemetryHistory history;
    history.addSampleAt({{"power_watts", 100.0}}, 9000);
    history.addSampleAt({{"power_watts", 200.0}}, 9000);
    history.addSampleAt({{"power_watts", 300.0}}, 8999);
    QCOMPARE(history.sampleCount(), 1);
}

void HistoryTests::clearEmitsUpdate()
{
    TelemetryHistory history;
    QSignalSpy spy(&history, &TelemetryHistory::updated);
    history.addSampleAt({{"temperature_c", 42.0}}, 1000);
    history.clear();
    QCOMPARE(history.sampleCount(), 0);
    QCOMPARE(spy.size(), 2);
}

QTEST_APPLESS_MAIN(HistoryTests)
#include "HistoryTests.moc"
