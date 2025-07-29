from typing import Optional
from PySide6.QtWidgets import QTableWidget, QTableWidgetItem, QHeaderView
from gui.basic_table_view import TableView
from gui.menu_bar import MenuBar

class SetupSignals:
    def __init__(self) -> None:
        self.__table_view_widget: Optional[QTableWidget] = None

    def set_open_file_signals(self, table_view: TableView, menu_bar: MenuBar) -> None:
        table_view_widget: QTableWidget = table_view.get_table_view_widget()
        self.__table_view_widget = table_view_widget
        
        menu_bar.signals.file_open_signal.connect(self.__on_file_open)

    def __on_file_open(self, file_path: str) -> None:
        if self.__table_view_widget:
            self.__show_file_info_to_table(
                file_path = file_path, 
                table_widget = self.__table_view_widget)
            
    def __show_file_info_to_table(self, file_path: str, table_widget: QTableWidget) -> None:
        import os

        header = table_widget.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        table_widget.setRowCount(1)
        table_widget.setItem(0, 0, QTableWidgetItem(f"{os.path.basename(file_path)}"))

        table_widget.setVisible(True)
