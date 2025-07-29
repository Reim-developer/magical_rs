from typing import Optional, Self
from PySide6.QtWidgets import QPushButton, QGridLayout, QMainWindow
from lib_shared.venus_core import debug
from gui.file_dialog import FileDialog

class OpenFile:
    def __init__(self) -> None:
        self.__button = QPushButton()
        self.__debug_mode = False
        self.__main_window: Optional[QMainWindow] = None

    def set_parent(self, main_window: QMainWindow) -> Self:
        self.__main_window = main_window
        self.__button.setParent(self.__main_window if self.__main_window else None)

        return self
    
    def set_open_file_button(self) -> Self:
        self.__button.setText("Open File")

        return self
    
    def set_layout(self, layout: QGridLayout) -> Self:
        layout.addWidget(self.__button, 3, 0)

        return self
    
    def set_event_listnener(self) -> Self:
        self.__button.clicked.connect(lambda: self.__show_dialog())

        return self
    
    def if_enable_debug(self, is_debug: bool):
        self.__debug_mode = is_debug

        if self.__debug_mode:
            debug("Found QPushButton: {}", self.__button)

    def __show_dialog(self) -> None:
        FileDialog() \
        .   with_parent(self.__main_window if self.__main_window else None) \
        .   show_file_dialog() \
        .   write_file_choose_path() \
        .   finnaly_if_enable_debug(self.__debug_mode)
