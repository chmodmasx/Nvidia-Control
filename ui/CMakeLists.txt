cmake_minimum_required(VERSION 3.21)
project(NvidiaControl VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 DBus)
qt_standard_project_setup(REQUIRES 6.4)

qt_add_executable(nvidia-control
    src/main.cpp
    src/TelemetryBridge.cpp
    src/TelemetryBridge.h
)

qt_add_qml_module(nvidia-control
    URI NvidiaControl
    VERSION 1.0
    RESOURCE_PREFIX /qt/qml
    QML_FILES
        qml/Main.qml
        qml/MetricCard.qml
)

target_link_libraries(nvidia-control PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::DBus
)

install(TARGETS nvidia-control RUNTIME DESTINATION bin)
