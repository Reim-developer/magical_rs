from PySide6.QtWidgets import QApplication
from sys import argv, exit
from gui import VenusMainWindow

def main_launch() -> None:
    app = QApplication(argv)

    if len(argv) > 1:
        match argv[1]:
            case "--debug":
                venus_main_window = VenusMainWindow(is_debug = True)
                venus_main_window.show_gui()
                exit(app.exec())

            case "--verbose":
                venus_main_window = VenusMainWindow(is_verbose = True)
                venus_main_window.show_gui()

                exit(app.exec())

            case "--dev-mode":
                venus_main_window = VenusMainWindow(is_debug = True, is_verbose = True)
                venus_main_window.show_gui()
                
                exit(app.exec())
                
            case _: ...

    else:
        venus_main_window  = VenusMainWindow()
        venus_main_window.show_gui()
        exit(app.exec())

if __name__ == "__main__":
    main_launch()