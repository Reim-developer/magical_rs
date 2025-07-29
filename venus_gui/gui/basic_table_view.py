from PySide6.QtWidgets import (
    QGridLayout, QTableWidget
)

class TableView:
    def __init__(self) -> None:
        self.__table_widget = QTableWidget()

    def show_basic_view(self, layout: QGridLayout) -> None:
        self.__table_widget.setColumnCount(4)
        self.__table_widget.setHorizontalHeaderLabels(["File Name", "File Size", "Extension", "Content"])
        self.__table_widget.horizontalHeader().setStretchLastSection(True)
        self.__table_widget.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.__table_widget.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)

        self.__table_widget.setVisible(False)
        layout.addWidget(self.__table_widget, 0, 0)

    def get_table_view_widget(self) -> QTableWidget:
        return self.__table_widget    
