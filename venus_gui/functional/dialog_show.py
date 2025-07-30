from enum import Enum
from typing import Callable, Optional, Self
from PySide6.QtWidgets import QMainWindow, QMessageBox

class DialogType(Enum):
    CRITICAL = 0
    WARNING = 1
    INFO = 2

def __show_dialog_by_type(
        message: str, dialog_type: DialogType, 
        title: str = "", parent: Optional[QMainWindow] = None) -> None:
    
    match dialog_type:
        case DialogType.CRITICAL:
            QMessageBox.critical(
                parent,
                message,
                title if title else "Critical"
            )

        case DialogType.WARNING:
            QMessageBox.warning(
                parent,
                message,
                title if title else "Warning"
            )

        case DialogType.INFO:
            QMessageBox.information(
                parent,
                message,
                title if title else "Information"
            )

def __show_dialog(
        message: str, title: str = "",
        parent: Optional[QMainWindow] = None,
        dialog_type: DialogType = DialogType.INFO) -> None:
    

    __show_dialog_by_type(
        message = message, dialog_type = dialog_type,
        title = title, parent = parent
    )

class WithDialogData:
    def __init__(self) -> None:
        self.__message = ""
        self.__title = ""
        self.__parent: Optional[QMainWindow] = None
        self.__dialog_type: DialogType = DialogType.INFO
        self.__error: Optional[Exception] = None
        self.__verbose = False

    def with_message(self, message: str) -> Self:
        self.__message = message

        return self
    
    def with_title(self, title: str) -> Self:
        self.__title = title

        return self
    
    def with_parent(self, parent: Optional[QMainWindow] = None) -> Self:
        self.__parent = parent

        return self
    
    def with_dialog_type(self, dialog_type: DialogType = DialogType.INFO) -> Self:
        self.__dialog_type = dialog_type

        return self
    
    def if_use_verbose(self, verbose: bool, error: Exception) -> Self:
        self.__verbose = verbose
        self.__error = error

        return self 
    
    def fn_show_dialog(self) -> Callable[[], None]:
        if self.__verbose:
            error_msg = f"Full error: {self.__error if self.__error else 'Unknown error.'}"

            fn = lambda: (
            __show_dialog(
                    message = f"{self.__message}\n{error_msg}", title = self.__title,
                    parent = self.__parent, dialog_type = self.__dialog_type
                )
            )

            return fn

        fn = lambda: (
            __show_dialog(
                message = self.__message, title = self.__title,
                parent = self.__parent, dialog_type = self.__dialog_type
            )
        )

        return fn