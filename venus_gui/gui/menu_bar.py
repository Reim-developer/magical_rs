from typing import Optional
from PySide6.QtWidgets import (
    QMainWindow, QMenuBar, QMenu, QFileDialog
)
from PySide6.QtGui import QAction
from lib_shared.venus_core import get_user_home
from gui.signals import AppSignals

class MenuBar:
    def __init__(self, main_window: QMainWindow) -> None:
        self.is_debug = False
        self.__main_window = main_window
        self.__menu_bar: Optional[QMenuBar] = self.__main_window.menuBar()
        self.signals = AppSignals()


    def show_menu_bar(self, is_debug: bool = False) -> None:
        self.is_debug = is_debug

        if self.__menu_bar:
            file_menu = self.__menu_bar.addMenu("File")

            self.__setup_action(menu = file_menu)

    def __setup_action(self, menu: QMenu) -> None:
        open_file_ac: QAction = QAction(text = "Open File", parent = self.__main_window)
        open_file_ac.setShortcut("Ctrl+F")
        open_file_ac.triggered.connect(lambda: self.on_open_file())

        menu.addAction(open_file_ac)

    def on_open_file(self) -> None:
        user_home: str | None = get_user_home()
        
        file_path, _ = QFileDialog.getOpenFileName(
            parent = self.__main_window,
            caption = "Open File",
            dir = user_home if user_home else "",
            filter = "All Files (*.*)"
        )

        if file_path:
            if self.is_debug:
                from lib_shared.venus_core import debug
                debug("Found file path: {}", file_path)

            self.signals.file_open_signal.emit(file_path)

        else:
            if self.is_debug:
                from lib_shared.venus_core import debug
                debug("File path not found. Maybe it's None or canceled by user")