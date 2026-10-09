cmake_minimum_required(VERSION 3.21)
project(NvidiaControl VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 DBus Test)
qt_standard_project_setup(REQUIRES 6.4)

qt_add_executable(nvidia-control
    src/main.cpp
    src/TelemetryBridge.cpp
    src/TelemetryBridge.h
    src/TelemetryHistory.cpp
    src/TelemetryHistory.h
)

qt_add_qml_module(nvidia-control
    URI NvidiaControl
    VERSION 1.0
    RESOURCE_PREFIX /qt/qml
    QML_FILES
        qml/Main.qml
        qml/MetricCard.qml
        qml/HistoryChart.qml
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

# History is an independent frontend component; no NVML or D-Bus needed to
# test time-window selection, bounded retention and missing samples.
enable_testing()

qt_add_executable(nvidia-control-history-tests
    tests/HistoryTests.cpp
    src/TelemetryHistory.cpp
    src/TelemetryHistory.h
)

target_include_directories(nvidia-control-history-tests PRIVATE src)
target_link_libraries(nvidia-control-history-tests PRIVATE Qt6::Core Qt6::Test)
add_test(NAME history-store COMMAND nvidia-control-history-tests)
