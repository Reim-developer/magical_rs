from PySide6.QtWidgets import QFileDialog, QMainWindow, QMessageBox
from typing import Optional, Self
from lib_shared.venus_core import (
    get_user_home, debug, get_abs_path, write_to_temp_file
)

class FileDialog:
    def __init__(self) -> None:
        self.__main_window: Optional[QMainWindow] = None
        self.__debug: bool = False
        self.__verbose: bool = False
        self.__file_choose: Optional[str] = None
        self.__TEMP_FILE: str = ".venus.tmp"

    def with_parent(self, main_window: Optional[QMainWindow]) -> Self:
        self.__main_window = main_window if main_window else None

        return self

    def show_file_dialog(self) -> Self:
        user_home: Optional[str] = get_user_home()

        file_choose, _ = QFileDialog.getOpenFileName(
            parent = self.__main_window if self.__main_window else None,
            caption = "Open File",
            dir = user_home if user_home else "",
            filter = "All Files (*.*)"
        )
        self.__file_choose = file_choose

        return self
    
    def write_file_choose_path(self) -> Self:
        if self.__file_choose:
            abs_path = get_abs_path(self.__file_choose)
            user_home = get_user_home()

            if abs_path and user_home:
                temp_file = f"{user_home}/{self.__TEMP_FILE}"

                try:
                    write_to_temp_file(temp_file, self.__file_choose)

                except Exception as error:
                    error_msg: str = f"Full error message: {error}"

                    QMessageBox.critical(
                        self.__main_window if self.__main_window else None,
                        "Critical Error:",
                        f"Could not write to {temp_file}\n" +
                        f"{error_msg if self.__verbose else ''}"
                    )

        return self
    
    def finnaly_if_enable_debug(self, is_debug: bool) -> None:
        self.__debug = is_debug

        if self.__debug:
            debug("In class: {}", self)
            debug("Field: {}", self.__main_window)