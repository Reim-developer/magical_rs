from gui.tab_view import TabView

class TabStateManager:
    def __init__(self) -> None:
        self.tab_view = TabView()

    def get_tab_view(self) -> TabView:
        
        return self.tab_view        