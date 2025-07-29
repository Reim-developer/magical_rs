from typing import Optional, Self
from PySide6.QtWidgets import (
    QMainWindow, QPushButton, QGridLayout, QMessageBox,
    QLabel
)
from lib_shared.venus_core import (
    debug, get_user_home, is_file_exists, read_file
)
from gui.tab_state import TabStateManager

class OpenRecentFile:
    def __init__(self) -> None:
        self.__main_window: Optional[QMainWindow] = None
        self.__is_debug = False
        self.__verbose = False
        self.__button = QPushButton(parent = self.__main_window)

        self.__tab_state: Optional[TabStateManager] = None

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
        grid_layout.addWidget(self.__button, 2, 0)

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

        if not is_file_exists(tmp_file_path):
            QMessageBox.warning(
                self.__main_window,
                "Warning",
                f"Venus's temp file: {self.__VENUS_TEMP_FILE} doens't exists\n" \
                "Cannot open recent file."
            )

            return

        try:
            recent_file = read_file(tmp_file_path)

        except Exception as error:
            full_error = f"Full error message: {error}"

            QMessageBox.critical(
                self.__main_window,
                "Critical Error",
                f"Could not read Venus's temp file: {tmp_file_path}\n" \
                f"{full_error if self.__verbose else ''}"
            )
            return
        
        if not is_file_exists(recent_file.strip()):
            QMessageBox.warning(
                self.__main_window,
                "Warning",
                f"The file: {recent_file} doens't not exists."
            )

            if self.__is_debug:
                debug("Recent file not found detect in: {}", self)
                debug("Recent file: {}", recent_file)
            return

        if self.__tab_state:
            tab_view = self.__tab_state.get_tab_view()
            basic_info_layout = tab_view.get_basic_info_layout()

            if basic_info_layout:
                self.__clear_layout(basic_info_layout)
                basic_info_layout.addWidget(QLabel(f"{recent_file}"), 0, 0)
