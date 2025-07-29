from typing import Optional
from PySide6.QtWidgets import (
    QMainWindow, QMenuBar, QMenu, QFileDialog
)
from PySide6.QtGui import QAction
from lib_shared.venus_core import get_user_home

class MenuBar:
    def __init__(self, main_window: QMainWindow) -> None:
        self.__main_window = main_window
        self.__menu_bar: Optional[QMenuBar] = self.__main_window.menuBar()

    def show_menu_bar(self) -> None:
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
            print(file_path)