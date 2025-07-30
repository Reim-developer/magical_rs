from typing import Callable, Optional, overload
from lib_shared.venus_core import FilePath
from PySide6.QtWidgets import QMessageBox, QMainWindow

def __show_warning_dialog(main_window: Optional[QMainWindow], msg: str, title: str = "") -> None:
    QMessageBox.warning(
        main_window,
        title if title else "Warning",
        msg
    )

def __handle_read_tmp_file(
            file_path: str, main_window: Optional[QMainWindow],
            verbose: bool) -> str:
    try:
        recent_file = FilePath.new_with_path(file_path = file_path).read_file()
        return recent_file

    except Exception as error:
        full_error = f"Full error message: {error}"

        QMessageBox.critical(
            main_window,
            "Critical Error",
            f"Could not read Venus's temp file: {file_path}\n" \
            f"{full_error if verbose else ''}"
        )

        return ""

def fn_get_path_from_tmp_file(file_path: str, 
        main_window: Optional[QMainWindow], verbose: bool) -> Callable[[], str]:
    
    fn = lambda: __handle_read_tmp_file(
        file_path = file_path, 
        main_window = main_window, verbose = verbose)
    
    return fn

def fn_show_warning_dialog(main_window: Optional[QMainWindow], msg: str) -> Callable[[], None]:
    fn = lambda: __show_warning_dialog(main_window = main_window, msg = msg)

    return fn

def fn_file_exists(file_path_: str) -> Callable[[], bool]:
    return lambda: FilePath.new_with_path(file_path = file_path_).is_file_exists()


@overload
def fn_if_file_not_exists(
    path: str, fn: Callable[[], bool], main_window: Optional[QMainWindow], 
    message: str = "") -> bool: ...

@overload
def fn_if_file_not_exists(
    path: str, fn: Callable[[], bool], main_window: Optional[QMainWindow], 
    message: str = "", title: str = "") -> bool: ...

def fn_if_file_not_exists(
        path: str, fn: Callable[[], bool], 
        main_window: Optional[QMainWindow], 
        message: str = "", title: str = "") -> bool:
    
    file_exists = fn()
    default_msg = f"Cannot open: {path}\nFile doesn't exists\n"

    if not file_exists:
        __show_warning_dialog(
            main_window = main_window,
            msg = message if message else default_msg,
            title = title if title else ""
        )

        return False
    
    return True