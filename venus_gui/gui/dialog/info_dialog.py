from typing import Optional, Self
from PySide6.QtWidgets import (
    QDialog, QMainWindow, QGridLayout, QLabel,
    QPushButton, QApplication, QMessageBox
)
from PySide6.QtCore import QTimer, QEvent
from lib_shared.venus_core import debug
from __version__ import VENUS_APP_NAME, VENUS_CORE_VERSION, VENUS_GUI_VERSION, VENUS_LICENSE
from sys import version
from lib_shared.venus_core import open_browser

class InfoDialog(QDialog):
    def __init__(self, main_window: Optional[QMainWindow]) -> None:
        super().__init__(main_window)
        self.__main_window: Optional[QMainWindow] = main_window
        self.__debug = False
        self.__verbose = False

        self.__GITHUB_URL = "https://github.com/reim-developer/venus"
        self.__ISSUES_URL = "https://github.com/reim-developer/venus/issues"
        self.__PULL_REQUEST_URL = "https://github.com/reim-developer/venus/pulls"
        
        self.__MIN_W = 300
        self.__MIN_H = 300
        self.__WIN_TITLE = "About"

        self.__dialog_layout = QGridLayout(self)

        self.__app_basic_info_label = QLabel(parent = self)
        self.__copy_info_button = QPushButton(parent = self)
        self.__source_code_button = QPushButton(parent = self)
        self.__open_issue_button = QPushButton(parent = self)
        self.__pull_request_button = QPushButton(parent = self)

    def enable_if_debug(self, is_debug: bool) -> Self:
        self.__debug = is_debug

        return self
    
    def enable_if_verbose(self, verbose: bool) -> Self:
        self.__verbose = verbose

        return self
    
    def __setup_gui_components(self) -> None:
        APP_INFO = \
            f"Application Name: {VENUS_APP_NAME}\n\n" \
            f"Application GUI Version: {VENUS_GUI_VERSION}\n\n" \
            f"Application Core Version: {VENUS_CORE_VERSION}\n\n" \
            f"Built-in Python: {version.split(" ")[0]}\n\n" \
            F"License: {VENUS_LICENSE}"
        
        self.__app_basic_info_label.setText(APP_INFO)

        self.__copy_info_button.setText("Copy Information")
        self.__copy_info_button.clicked.connect(lambda: self.__on_copy_information(APP_INFO))

        self.__source_code_button.setText("Source Code")
        self.__source_code_button.clicked.connect(lambda: self.__on_open_browser(self.__GITHUB_URL))

        self.__open_issue_button.setText("Open Issue")
        self.__open_issue_button.clicked.connect(lambda: self.__on_open_browser(self.__ISSUES_URL))

        self.__pull_request_button.setText("Open Pull Request")
        self.__pull_request_button.clicked.connect(lambda: self.__on_open_browser(self.__PULL_REQUEST_URL))

        self.__dialog_layout.addWidget(self.__app_basic_info_label, 0, 0)
        self.__dialog_layout.addWidget(self.__copy_info_button, 0, 2)
        self.__dialog_layout.addWidget(self.__source_code_button, 1, 0)
        self.__dialog_layout.addWidget(self.__open_issue_button, 1, 1)
        self.__dialog_layout.addWidget(self.__pull_request_button, 1, 2)

       
    def set_dialog_gui(self) -> Self:
        self.setWindowTitle(self.__WIN_TITLE)
        self.resize(self.__MIN_W, self.__MIN_H)
        self.setModal(True)

        if self.__debug:
            debug("In Window: {}", self.__WIN_TITLE)
            debug("Size width: {}", self.__MIN_W)
            debug("Size height: {}", self.__MIN_H)

        return self
    
    def show_dialog(self) -> None:
        self.__setup_gui_components()
        self.exec()


    def __on_copy_information(self, info: str) -> None:
        clipboard = QApplication.clipboard()
        clipboard.setText(info)

        self.__copy_info_button.setText("Copied!")
        QTimer.singleShot(500, lambda: self.__copy_info_button.setText("Copy Information"))
    
    def __on_open_browser(self, url: str) -> None:
        try:
            open_browser(url)

        except Exception as error:
            verbose_msg = f"With error: {error}"

            QMessageBox.critical(
                self,
                "Critical Error",
                f"Could not open your browser\n{verbose_msg if self.__verbose else ''}"
            )

    def closeEvent(self, event: QEvent) -> None:
        if self.__main_window:
            event.accept()
            self.__main_window.show()