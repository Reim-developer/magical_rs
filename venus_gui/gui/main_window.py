from PySide6.QtWidgets import (
    QMainWindow, QApplication, QWidget, QGridLayout
)
from lib_shared.venus_core import debug
from __version__ import VENUS_APP_NAME, VENUS_GUI_VERSION
from gui.open_file import OpenFile
from gui.venus_info import VenusAbout
from gui.open_recent import OpenRecentFile
from gui.tab_state import TabStateManager

class VenusMainWindow(QMainWindow):
    def __init__(self, is_debug: bool = False, is_verbose: bool = False):
        super().__init__()
        self.__is_debug = is_debug
        self.__is_verbose = is_verbose
        self.__primary_screen = QApplication.primaryScreen()

        self.__central_widget = QWidget()
        self.__grid_layout = QGridLayout()
        self.__central_widget.setLayout(self.__grid_layout)
        self.setCentralWidget(self.__central_widget)

        self.__table_state = TabStateManager()
        self.__tab_view = self.__table_state.get_tab_view()

    def __debug_time(self, screen_w: int, screen_h: int) -> None:
        debug("Found primary_screen as: {}", self.__primary_screen)
        debug("Current screen width: {}", screen_w)
        debug("Current screen heigt: {}", screen_h)

    def __setup_gui(self) -> None:
        OpenFile() \
        .   set_parent(self) \
        .   set_open_file_button() \
        .   set_layout(self.__grid_layout) \
        .   set_event_listnener() \
        .   if_enable_debug(is_debug = self.__is_debug)

        VenusAbout() \
        .   enable_if_debug(self.__is_debug) \
        .   enable_if_verbose(self.__is_verbose) \
        .   set_parent(self) \
        .   set_about_button() \
        .   set_layout(self.__grid_layout) \
        .   set_clicked_event()

        OpenRecentFile() \
        .   if_enable_debug(self.__is_debug) \
        .   if_enable_verbose(self.__is_verbose) \
        .   set_widget_parent(self) \
        .   setup_layout(self.__grid_layout) \
        .   with_tab_state(self.__table_state) \
        .   setup_open_recent_button() \
        .   setup_clicked_event()


        self.__tab_view \
        .   set_parent(self) \
        .   set_tab_layout() \
        .   set_tab_view(self.__grid_layout)

    def __center_window(self) -> None:
        if not self.__primary_screen:
            debug("In: {}", self)
            debug("Could not find primary_screen. Maybe it's None")
            return
        
        screen = self.__primary_screen.availableGeometry()
        window = self.geometry()

        window_width = window.width()
        window_height = window.height()

        screen_width = screen.width()
        screen_height = screen.height()

        x_loc = (screen_width - window_width) // 2
        y_loc = (screen_height - window_height) // 2

        if self.__is_debug:
            self.__debug_time(screen_w = screen_width, screen_h = screen_height)

        self.move(x_loc, y_loc)

    def show_gui(self) -> None:
        self.resize(800, 600)
        self.setWindowTitle(f"{VENUS_APP_NAME} | v{VENUS_GUI_VERSION}")
        self.__center_window()
        self.__setup_gui()
        self.show()