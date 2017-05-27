import os.path
import json

from .version import latest

class ServerSettings(object):
    """Server settings file, lazily loaded."""

    FILENAME = ".facts.json"
    DEFAULT_SETTINGS = {
        "experimental": False
    }

    def __init__(self, server):
        self.__server = server
        self.__data = None
        self.filename = self.__server.get_file(ServerSettings.FILENAME)

    def _create(self):
        self.__data = self.DEFAULT_SETTINGS.copy()
        if server.version > latest(experimental=False):
            self.__data["experimental"] = True
        self._save()

    def _load(self):
        if not os.path.exists(self.filename):
            self._create()

        with open(self.filename) as f:
            try:
                self.__data = json.load(f)
            except json.decoder.JSONDecodeError:
                self._create()

    def _save(self):
        assert self.__data is not None

        with open(self.filename, "w") as f:
            json.dump(self.__data, f)

    @property
    def _data(self):
        if self.__data is None:
            self._load()
        return self.__data

    @_data.setter
    def _data(self, value):
        if self.__data is None:
            self.load()
        self.__data = value

    @property
    def experimental(self):
        return self._data["experimental"]

    @experimental.setter
    def experimental(self, value):
        assert isinstance(value, bool)
        self._data["experimental"] = value
        self._save()
