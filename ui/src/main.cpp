#include "TelemetryBridge.h"
#include "TelemetryHistory.h"

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QUrl>

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Nvidia-Control"));
    QGuiApplication::setOrganizationName(QStringLiteral("Nvidia-Control"));

    TelemetryBridge bridge;
    TelemetryHistory history;
    QObject::connect(&bridge, &TelemetryBridge::telemetryReceived,
                     &history, &TelemetryHistory::addSample);
    QObject::connect(&bridge, &TelemetryBridge::deviceChanged,
                     &history, &TelemetryHistory::clear);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("telemetryBridge"), &bridge);
    engine.rootContext()->setContextProperty(QStringLiteral("telemetryHistory"), &history);
    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/NvidiaControl/qml/Main.qml")));

    if (engine.rootObjects().isEmpty()) {
        return 1;
    }

    return app.exec();
}
