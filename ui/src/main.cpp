#include "TelemetryBridge.h"

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
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("telemetryBridge"), &bridge);
    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/NvidiaControl/qml/Main.qml")));

    if (engine.rootObjects().isEmpty()) {
        return 1;
    }

    return app.exec();
}
