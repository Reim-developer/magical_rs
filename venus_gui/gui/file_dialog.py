from PySide6.QtWidgets import QFileDialog, QMainWindow
from typing import Optional, Self
from lib_shared.venus_core import (
    get_user_home, debug, FilePath
)
from functional.dialog_show import DialogType, WithDialogData

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
            abs_path = FilePath.new_with_path(file_path = self.__file_choose).get_abs_path()
            user_home = get_user_home()

            if abs_path and user_home:
                
                temp_file = f"{user_home}/{self.__TEMP_FILE}"
                try:
                    FilePath.new_with_path(file_path = temp_file) \
                    .   write_to_temp_file(self.__file_choose)

                except Exception as error:
                    error_dialog = ( 
                        WithDialogData() 
                        .   with_parent(self.__main_window) 
                        .   with_message(f"Could not write to: {temp_file}\n") 
                        .   if_use_verbose(self.__verbose, error) 
                        .   with_dialog_type(DialogType.CRITICAL) 
                        .   fn_show_dialog()
                    )
                    error_dialog()
                
        return self
    
    def if_enable_debug(self, is_debug: bool) -> None:
        self.__debug = is_debug

        if self.__debug:
            debug("In class: {}", self)
            debug("Field: {}", self.__main_window)