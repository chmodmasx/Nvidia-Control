import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    width: 1120
    height: 790
    minimumWidth: 690
    minimumHeight: 540
    visible: true
    title: "Nvidia-Control"
    color: "#20242c"

    property int currentPage: 0
    property int historyMinutes: 5
    readonly property var device: telemetryBridge.device
    readonly property var sensors: telemetryBridge.telemetry
    readonly property var limits: telemetryBridge.limits
    readonly property var power: limits.power || ({})
    readonly property var clocks: limits.clocks || ({})
    readonly property var fans: limits.fans || ({})

    function fmt(value, unit, digits) {
        return value === undefined || value === null
            ? "—" : Number(value).toFixed(digits || 0) + (unit || "")
    }
    function gib(value) {
        return value === undefined || value === null
            ? "—" : (Number(value) / 1073741824).toFixed(2) + " GiB"
    }
    function ratio(used, total) {
        return total > 0 ? used / total : -1
    }
    function capabilityName(value) {
        if (value === "read_only") return "Solo lectura"
        if (value === "read_write") return "Lectura / escritura"
        if (value === "experimental_read_write") return "Experimental"
        if (value === "unsupported") return "No compatible"
        return "Sin verificar"
    }
    function frequencyList(values) {
        return values && values.length > 0 ? values.join(", ") + " MHz" : "No informado"
    }

    Rectangle {
        id: sidebar
        width: 216
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        color: "#252b34"
        border.color: "#343d4c"
        border.width: 1

        Column {
            anchors.fill: parent
            anchors.margins: 18
            spacing: 14

            Label {
                text: "NVIDIA CONTROL"
                color: "#eaf5ff"
                font.pixelSize: 16
                font.bold: true
                topPadding: 8
                bottomPadding: 18
            }

            Repeater {
                model: ["Resumen", "Detalles", "Historial"]
                delegate: Button {
                    required property int index
                    required property string modelData
                    width: parent.width
                    height: 46
                    text: modelData
                    onClicked: window.currentPage = index

                    background: Rectangle {
                        radius: 10
                        color: window.currentPage === index ? "#335e7b" : "#303741"
                        border.color: window.currentPage === index ? "#51a2da" : "#424b56"
                    }
                    contentItem: Label {
                        text: parent.text
                        verticalAlignment: Text.AlignVCenter
                        horizontalAlignment: Text.AlignLeft
                        leftPadding: 16
                        color: "#eff6fc"
                        font.pixelSize: 14
                    }
                }
            }

            Item { width: 1; height: 10 }

            Label {
                text: "Ajustes de GPU"
                color: "#738395"
                font.pixelSize: 13
            }
            Label {
                width: parent.width
                text: "Próxima etapa: perfiles, clocks y ventiladores con autorización."
                color: "#8495ab"
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }
        }
    }

    ColumnLayout {
        anchors.left: sidebar.right
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 28
        spacing: 17

        RowLayout {
            Layout.fillWidth: true

            ColumnLayout {
                spacing: 4
                Label {
                    text: window.currentPage === 0 ? "Resumen de GPU" :
                          window.currentPage === 1 ? "Límites y capacidades" : "Historial de telemetría"
                    color: "#f6f6f6"
                    font.pixelSize: 27
                    font.weight: Font.DemiBold
                }
                Label {
                    text: device.name || "Esperando datos de NVIDIA"
                    color: "#aab7c6"
                    font.pixelSize: 14
                }
            }

            Item { Layout.fillWidth: true }

            Rectangle {
                width: 12
                height: 12
                radius: 6
                color: telemetryBridge.connected ? "#5ccc9a" : "#dc855b"
            }

            Label {
                text: telemetryBridge.connected ? "En vivo · 1 s" : "Desconectado"
                color: telemetryBridge.connected ? "#bce7d1" : "#e3b19a"
                font.pixelSize: 12
            }
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 50
            radius: 9
            color: "#3b3030"
            border.color: "#775451"
            visible: !telemetryBridge.connected

            Label {
                anchors.fill: parent
                anchors.margins: 12
                text: telemetryBridge.error + " · Iniciá el daemon con --session."
                color: "#ffd5c7"
                wrapMode: Text.WordWrap
                verticalAlignment: Text.AlignVCenter
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

            ColumnLayout {
                width: parent.width
                spacing: 17

                ColumnLayout {
                    visible: window.currentPage === 0
                    Layout.fillWidth: true
                    spacing: 16

                    GridLayout {
                        Layout.fillWidth: true
                        columns: width >= 730 ? 3 : 2
                        columnSpacing: 12
                        rowSpacing: 12

                        MetricCard {
                            Layout.fillWidth: true
                            label: "Uso de GPU"
                            value: window.fmt(sensors.gpu_util_percent, " %")
                            progress: sensors.gpu_util_percent === undefined ? -1 : Number(sensors.gpu_util_percent) / 100
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Temperatura GPU"
                            value: window.fmt(sensors.temperature_c, " °C")
                            subtext: "Sensor del núcleo"
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Consumo"
                            value: window.fmt(sensors.power_watts, " W", 1)
                            subtext: "Lectura instantánea"
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Memoria de video"
                            value: window.gib(sensors.memory_used_bytes)
                            subtext: "de " + window.gib(sensors.memory_total_bytes)
                            progress: window.ratio(sensors.memory_used_bytes, sensors.memory_total_bytes)
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Reloj del núcleo"
                            value: window.fmt(sensors.core_clock_mhz, " MHz")
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Reloj de memoria"
                            value: window.fmt(sensors.memory_clock_mhz, " MHz")
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Ventiladores"
                            value: window.fmt(sensors.fan_percent, " %")
                            progress: sensors.fan_percent === undefined ? -1 : Number(sensors.fan_percent) / 100
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Controlador de memoria"
                            value: window.fmt(sensors.memory_util_percent, " %")
                            subtext: "Actividad, no VRAM ocupada"
                            progress: sensors.memory_util_percent === undefined ? -1 : Number(sensors.memory_util_percent) / 100
                        }
                        MetricCard {
                            Layout.fillWidth: true
                            label: "Memoria / hotspot"
                            value: window.fmt(sensors.memory_temperature_c, " °C")
                            subtext: "Hotspot: " + window.fmt(sensors.hotspot_temperature_c, " °C")
                        }
                    }

                    Label {
                        text: "Los valores no disponibles se muestran como «—». Sin cambios de hardware."
                        color: "#8b9bae"
                        font.pixelSize: 12
                    }
                }

                ColumnLayout {
                    visible: window.currentPage === 2
                    Layout.fillWidth: true
                    spacing: 16

                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            text: "Historial local · hasta 60 minutos"
                            color: "#b1bfd0"
                            font.pixelSize: 13
                            Layout.fillWidth: true
                        }

                        Repeater {
                            model: [5, 15, 60]
                            delegate: Button {
                                required property int modelData
                                text: modelData + " min"
                                onClicked: window.historyMinutes = modelData
                                background: Rectangle {
                                    radius: 8
                                    color: window.historyMinutes === modelData ? "#335e7b" : "#303741"
                                    border.color: window.historyMinutes === modelData ? "#51a2da" : "#424b56"
                                }
                                contentItem: Label {
                                    text: parent.text
                                    color: "#eff6fc"
                                    font.pixelSize: 12
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                    leftPadding: 12
                                    rightPadding: 12
                                }
                            }
                        }
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: width >= 880 ? 2 : 1
                        columnSpacing: 12
                        rowSpacing: 12

                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Uso de GPU"
                            metricKey: "gpu_util_percent"
                            unit: "%"
                            percentage: true
                            minutes: window.historyMinutes
                        }
                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Temperatura GPU"
                            metricKey: "temperature_c"
                            unit: "°C"
                            minutes: window.historyMinutes
                        }
                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Potencia"
                            metricKey: "power_watts"
                            unit: "W"
                            decimals: 1
                            minutes: window.historyMinutes
                        }
                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Memoria de video utilizada"
                            metricKey: "memory_used_bytes"
                            unit: "GiB"
                            decimals: 2
                            minutes: window.historyMinutes
                        }
                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Frecuencia de GPU"
                            metricKey: "core_clock_mhz"
                            unit: "MHz"
                            minutes: window.historyMinutes
                        }
                        HistoryChart {
                            Layout.fillWidth: true
                            heading: "Frecuencia de memoria"
                            metricKey: "memory_clock_mhz"
                            unit: "MHz"
                            minutes: window.historyMinutes
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: "Los gráficos se completan mientras Nvidia-Control esté abierto. " +
                              "Las lecturas no disponibles y las desconexiones no se unen con líneas."
                        color: "#8b9bae"
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                    }
                }

                ColumnLayout {
                    visible: window.currentPage === 1
                    Layout.fillWidth: true
                    spacing: 12

                    Label {
                        text: "Potencia"
                        color: "#edf5fb"
                        font.pixelSize: 20
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 12
                        MetricCard { Layout.fillWidth: true; label: "Límite actual"; value: window.fmt(power.current_watts, " W") }
                        MetricCard { Layout.fillWidth: true; label: "Rango disponible"; value: window.fmt(power.min_watts, " W") + " – " + window.fmt(power.max_watts, " W") }
                        MetricCard { Layout.fillWidth: true; label: "Límite por defecto"; value: window.fmt(power.default_watts, " W") }
                        MetricCard { Layout.fillWidth: true; label: "Límite aplicado"; value: window.fmt(power.enforced_watts, " W") }
                    }

                    Label {
                        text: "Frecuencias y ventiladores"
                        color: "#edf5fb"
                        font.pixelSize: 20
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 12
                        MetricCard { Layout.fillWidth: true; label: "Máximo GPU"; value: window.fmt(clocks.max_graphics_mhz, " MHz") }
                        MetricCard { Layout.fillWidth: true; label: "Máximo memoria"; value: window.fmt(clocks.max_memory_mhz, " MHz") }
                        MetricCard { Layout.fillWidth: true; label: "Rango de ventiladores"; value: window.fmt(fans.min_percent, " %") + " – " + window.fmt(fans.max_percent, " %") }
                        MetricCard { Layout.fillWidth: true; label: "Driver NVIDIA"; value: device.driver_version || "—" }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: "Frecuencias de memoria publicadas por el driver"
                        color: "#c3d4e3"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                    }
                    Label {
                        Layout.fillWidth: true
                        text: window.frequencyList(clocks.supported_application_memory_mhz)
                        color: "#8facbf"
                        wrapMode: Text.WordWrap
                    }
                    Label {
                        Layout.fillWidth: true
                        text: "Tabla de application clocks: solo diagnóstico; no representa la curva V/F ni habilita escritura."
                        color: "#8b9bae"
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                    }

                    Label {
                        text: "Capacidades detectadas"
                        color: "#edf5fb"
                        font.pixelSize: 20
                    }
                    Repeater {
                        model: [
                            { name: "Telemetría", key: "telemetry" },
                            { name: "Power Limit", key: "power_limit" },
                            { name: "Clocks", key: "clocks" },
                            { name: "Ventiladores", key: "fan_control" },
                            { name: "Curva voltaje/frecuencia", key: "voltage_frequency_curve" },
                            { name: "Digital Vibrance", key: "digital_vibrance" },
                            { name: "DLSS", key: "dlss_overrides" },
                            { name: "Reflex", key: "reflex" },
                            { name: "Smooth Motion", key: "smooth_motion" }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            Layout.fillWidth: true
                            implicitHeight: 38
                            color: "#2c3039"
                            radius: 8
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 12
                                anchors.rightMargin: 12
                                Label { text: modelData.name; color: "#e5eaf1"; Layout.fillWidth: true }
                                Label {
                                    text: window.capabilityName((device.capabilities || ({}))[modelData.key])
                                    color: "#9dc9e6"
                                }
                            }
                        }
                    }
                }
            }
        }

        Label {
            Layout.fillWidth: true
            text: "Nvidia-Control · Solo lectura · Sin privilegios"
            horizontalAlignment: Text.AlignRight
            color: "#738395"
            font.pixelSize: 11
        }
    }
}
