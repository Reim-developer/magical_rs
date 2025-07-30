from typing import Generic, Optional, Sequence, TypeVar
from PySide6.QtCore import (
    QAbstractTableModel, QModelIndex, QPersistentModelIndex, Qt
)

T = TypeVar('T')

__Row__ = Sequence[T]
__Model_Index__ = QModelIndex | QPersistentModelIndex
__Table_Data__ = list[list[T]]

class VenusTableModel(QAbstractTableModel, Generic[T]):
    def __init__(
            self, table_data: __Table_Data__[T], 
            header_labels: list[str] | None = None) -> None:
        
        super().__init__()

        self.__ZERO = 0
        self.__table_data = table_data
        self.__Qt_DISPLAY_ROLE = Qt.ItemDataRole.DisplayRole

        if len(table_data) == self.__ZERO or len(table_data[0]) == self.__ZERO:
            self.__table_data_len = self.__ZERO

        else:
            self.__table_data_len = len(table_data[0])

        self.__range_index = range(self.__table_data_len if self.__table_data_len else self.__ZERO)
        self.__header_labels = header_labels or [f"Column {index}" for index in  self.__range_index]

    def rowCount(self, parent: Optional[__Model_Index__] = None) -> int:
        return len(self.__table_data)
    
    def columnCount(self, parent: Optional[__Model_Index__] = None) -> int:
        if self.rowCount() == self.__ZERO:
            return self.__ZERO
        
        return len(self.__table_data[0])


    def data(self, index: __Model_Index__, role: Optional[int] = None) -> T | None:
        if not index.isValid():
            return None

        if role == self.__Qt_DISPLAY_ROLE:

            row = index.row()
            column = index.column()

            try:
                return self.__table_data[row][column]
            
            except IndexError:
                return None
            
        return None
    
    def headerData(
            self, section: int, 
            orientation: Qt.Orientation, 
            role: int = Qt.ItemDataRole.DisplayRole) -> str | None:

        if role != self.__Qt_DISPLAY_ROLE:
            return super().headerData(section, orientation, role)
        
        if orientation == Qt.Orientation.Horizontal:
            if self.__ZERO <= section < len(self.__header_labels):
                    return self.__header_labels[section]

        elif orientation == Qt.Orientation.Vertical:
            return str(section + 1)     

        return None
    
    def flags(self, index: __Model_Index__) -> Qt.ItemFlag:
        if not index.isValid():
            return Qt.ItemFlag.NoItemFlags
        
        return super().flags(index)
    
    def setData(
            self, index: __Model_Index__, 
            value: T, role: int = Qt.ItemDataRole.EditRole) -> bool:
        
        if not index.isValid() or role != Qt.ItemDataRole.EditRole:
            return False

        row = index.row()
        colum  = index.column()

        try:
            self.__table_data[row][colum] = value
            self.dataChanged.emit(index, index, [role])

            return True

        except (IndexError, TypeError, ValueError):
            return False
        
        
    