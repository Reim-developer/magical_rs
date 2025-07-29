from PySide6.QtWidgets import (
    QMainWindow, QApplication
)
from lib_shared.venus_core import debug
from __version__ import VENUS_APP_NAME, VENUS_GUI_VERSION
from gui.menu_bar import MenuBar

class VenusMainWindow(QMainWindow):
    def __init__(self, is_debug: bool = False):
        super().__init__()
        self.is_debug = is_debug
        self.primary_screen = QApplication.primaryScreen()
         
    def __debug_time(self, screen_w: int, screen_h: int) -> None:
        debug("Found primary_screen as: {}", self.primary_screen)
        debug("Current screen width: {}", screen_w)
        debug("Current screen heigt: {}", screen_h)

    def __setup_gui(self) -> None:
        MenuBar(main_window = self).show_menu_bar()

    def __center_window(self) -> None:
        if not self.primary_screen:
            debug("Could not find primary_screen. Maybe it's None")
            return
        
        screen = self.primary_screen.availableGeometry()
        window = self.geometry()

        window_width = window.width()
        window_height = window.height()

        screen_width = screen.width()
        screen_height = screen.height()

        x_loc = (screen_width - window_width) // 2
        y_loc = (screen_height - window_height) // 2

        if self.is_debug:
            self.__debug_time(screen_w = screen_width, screen_h = screen_height)

        self.move(x_loc, y_loc)

    def show_gui(self) -> None:
        self.resize(600, 600)
        self.setWindowTitle(f"{VENUS_APP_NAME} | v{VENUS_GUI_VERSION}")
        self.__center_window()
        self.__setup_gui()
        self.show()