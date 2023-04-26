class GameTestScenario:

    def __init__(self, lib):
        self._lib = lib
        self._scenario = None

    def create(self):
        self._scenario = self._lib.create()

    def dispose(self):
        self._lib.dispose(self._scenario)

    def change_data(self, data: str):
        self._lib.change_data(self._scenario, data.encode('utf-8'))