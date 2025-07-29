from PySide6.QtCore import QObject, Signal

class AppSignals(QObject):
    file_open_signal: Signal = Signal(str)