from typing import Optional, Self
from PySide6.QtWidgets import (
    QTableView, QMainWindow, QGridLayout, QHeaderView
)
from gui.table_model import VenusTableModel

class TableView:
    def __init__(self) -> None:
        self.__table_view: QTableView = QTableView()
        self.__main_window: Optional[QMainWindow] = None
        self.__is_debug: bool = False

    def if_enable_debug(self, is_debug: bool) -> Self:
        self.__is_debug = is_debug

        return self
    
    def set_parent(self, main_window: Optional[QMainWindow]) -> Self:
        self.__main_window = main_window
        self.__table_view.setParent(self.__main_window if self.__main_window else None)   

        return self
    
    def set_layout(self, layout: QGridLayout) -> Self:
        layout.addWidget(self.__table_view, 0, 0)

        return self

    def set_table_view(self, data: list[list[str]], header: list[str]) -> None:
        self.__table_model = VenusTableModel[str](data, header_labels = header)
        self.__table_view.setModel(self.__table_model)
        self.__table_view.horizontalHeader().setStretchLastSection(True)
        self.__table_view.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.ResizeToContents)