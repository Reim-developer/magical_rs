from typing import Optional, Self
from PySide6.QtWidgets import (
    QPushButton, QMainWindow, QGridLayout
)
from lib_shared.venus_core import debug
from gui.dialog.info_dialog import InfoDialog

class VenusAbout:
    def __init__(self) -> None:
        self.__button = QPushButton()
        self.__is_debug = False
        self.__verbose = False
        self.__main_window: Optional[QMainWindow] = None

    def enable_if_debug(self, is_debug: bool) -> Self:
        self.__is_debug = is_debug

        return self
    
    def enable_if_verbose(self, is_verbose: bool) -> Self:
        self.__verbose = is_verbose

        return self

    def set_parent(self, main_window: QMainWindow) -> Self:
        self.__main_window = main_window

        if self.__main_window:
            self.__button.setParent(main_window)

        else:
            if self.__is_debug:
                debug("Could not set About Button parent. Because 'MainWindow' is None.")

        return self

    def set_about_button(self) -> Self:
        self.__button.setText("About")

        return self
    
    def set_layout(self, grid_layout: QGridLayout) -> Self:
        grid_layout.addWidget(self.__button, 3, 1)

        return self 
    
    def set_clicked_event(self) -> None:
        self.__button.clicked.connect(lambda: self.__show_info_dialog())
    
    def __show_info_dialog(self) -> None:
        self.info_dialog = InfoDialog(self.__main_window)

        if self.__main_window:
            self.__main_window.hide()

        self.info_dialog \
        .   enable_if_debug(self.__is_debug) \
        .   enable_if_verbose(self.__verbose) \
        .   set_dialog_gui() \
        .   show_dialog()

    