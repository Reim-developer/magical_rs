from typing import Optional, Self
from PySide6.QtWidgets import (
    QMainWindow, QPushButton, QGridLayout,
)
from lib_shared.venus_core import (
    debug, get_user_home, 
)
from gui.tab_state import TabStateManager
from functional.open_recent import (
    fn_file_exists, fn_get_path_from_tmp_file,
    fn_if_file_not_exists
)

class OpenRecentFile:
    def __init__(self) -> None:
        self.__main_window: Optional[QMainWindow] = None
        self.__is_debug = False
        self.__verbose = False
        self.__button = QPushButton(parent = self.__main_window)

        self.__tab_state: Optional[TabStateManager] = None
        self.__grid_layout: Optional[QGridLayout] = None

        self.__OPEN_RECENT_TEXT = "Open Recent File"
        self.__VENUS_TEMP_FILE = ".venus.tmp"

    def if_enable_debug(self, is_debug: bool) -> Self:
        self.__is_debug = is_debug

        return self
    
    def if_enable_verbose(self, is_verbose: bool) -> Self:
        self.__verbose = is_verbose

        return self

    def set_widget_parent(self, main_window: QMainWindow) -> Self:
        self.__main_window = main_window
        self.__button.setParent(self.__main_window if self.__main_window else None)
        
        if not self.__main_window:
            debug("In class: {}", self)
            debug("Could not fin 'MainWindow', maybe it's None")

        return self
    
    def setup_layout(self, grid_layout: QGridLayout) -> Self:
        self.__grid_layout = grid_layout

        if self.__grid_layout:
            self.__grid_layout.addWidget(self.__button, 2, 0)

        return self 

    def setup_open_recent_button(self) -> Self:
        self.__button.setText(self.__OPEN_RECENT_TEXT)

        if self.__is_debug:
            debug("In class: '{}'", self)
            debug("Found '{}' widget.", self.__OPEN_RECENT_TEXT)

        return self
    
    def with_tab_state(self, tab_state: TabStateManager) -> Self:
        self.__tab_state = tab_state

        return self

    def setup_clicked_event(self) -> None:
        self.__button.clicked.connect(lambda: self.__on_recent_button_clicked())
    
    def __clear_layout(self, layout: QGridLayout) -> None:
        if layout:
            while layout.count():
                widget_child = layout.takeAt(0)

                if widget_child:
                    widget_child.widget().deleteLater()

    def __on_recent_button_clicked(self) -> None:
        user_home = get_user_home()
        tmp_file_path = f"{user_home}/{self.__VENUS_TEMP_FILE}"

        tmp_file_exists = fn_file_exists(tmp_file_path)
        is_exists = fn_if_file_not_exists(
            path = tmp_file_path, fn = tmp_file_exists,
            main_window = self.__main_window
        )

        if not is_exists: return
    
        try_get_recent_file = fn_get_path_from_tmp_file(
            file_path = tmp_file_path, main_window = self.__main_window,
            verbose = self.__verbose
        )

        recent_file_path = try_get_recent_file()
        recent_file_exists = fn_file_exists(recent_file_path.strip())

        is_recent_file_exists = fn_if_file_not_exists(
            path = recent_file_path,
            fn = recent_file_exists,
            main_window = self.__main_window
        )

        if not is_recent_file_exists: return

        if self.__tab_state:
            tab_view = self.__tab_state.get_tab_view()
            self.__basic_info_layout = tab_view.get_basic_info_layout()

            if self.__basic_info_layout:
                self.__clear_layout(self.__basic_info_layout)

                data: list[list[str]] = [
                    [f"{recent_file_path}", "test_2", "test_3", "test_4"]
                ]

                header: list[str] = ["Full Path:", "Name", "Type", "Size"]
                tab_view.show_basic_information_table(data, header)
                