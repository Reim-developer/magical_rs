from typing import Optional, Self
from PySide6.QtWidgets import (
    QMainWindow, QTabWidget, QGridLayout, QWidget,
    QLabel
)

from gui.table_view import TableView

class TabView:
    def __init__(self) -> None:
        self.__main_window: Optional[QMainWindow] = None
        self.__tab_widget = QTabWidget(parent = self.__main_window)

        self.__basic_info_tab = QWidget(parent = self.__main_window)
        self.__basic_info_layout = QGridLayout(self.__basic_info_tab)
        
        self.__metadata_info_tab = QWidget(parent = self.__main_window)
        self.__metadata_info_layout = QGridLayout(self.__metadata_info_tab)

        self.__NOTHING_MSG = "Nothing to show. Open file to start."
        self.__is_debug = False

    def set_parent(self, main_window: QMainWindow) -> Self:
        self.__main_window = main_window

        return self
    
    def set_tab_layout(self) -> Self:
        self.__basic_info_layout.addWidget(QLabel(self.__NOTHING_MSG), 0 , 0)
        self.__metadata_info_layout.addWidget(QLabel(self.__NOTHING_MSG), 0, 0)
    
        return self 
    
    def set_tab_view(self, layout: QGridLayout) -> Self:
        self.__tab_widget.addTab(self.__basic_info_tab, "Basic Information")
        self.__tab_widget.addTab(self.__metadata_info_tab, "Metadata")
        layout.addWidget(self.__tab_widget, 0, 0, 1, 2)

        return self
    
    def show_basic_information_table(self, data: list[list[str]], header: list[str]) -> None:
        TableView() \
        .   if_enable_debug(self.__is_debug) \
        .   set_parent(self.__main_window if self.__main_window else None) \
        .   set_layout(self.__basic_info_layout) \
        .   set_table_view(data, header)
    
    def get_basic_info_layout(self) -> QGridLayout:

        return self.__basic_info_layout
    
    def get_metadata_info_layout(self) -> QGridLayout:
        
        return self.__metadata_info_layout