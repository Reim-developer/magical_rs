from PySide6.QtWidgets import QApplication
from sys import argv, exit
from gui import VenusMainWindow

def main_launch() -> None:
    app = QApplication(argv)

    if len(argv) > 1 and argv[1] == "--debug":
        venus_main_window = VenusMainWindow(is_debug = True)

    else:
        venus_main_window = VenusMainWindow()

    venus_main_window.show_gui()

    exit(app.exec())

if __name__ == "__main__":
    main_launch()