import os
import shutil
import json
import tempfile
import tarfile
import subprocess
import requests

from .server_settings import ServerSettings
from .download import stream as downdload_stream
from .version import Version, Update
from .exceptions import *

class Server(object):
    @classmethod
    def create(cls, server_dir, version, dl_callback=(lambda a, b: None)):
        """Creates a new server dir with given version."""
        assert isinstance(version, Version)
        assert not os.path.exists(server_dir)

        url = "https://www.factorio.com/get-download/{}/headless/linux64".format(str(version))
        with tarfile.open(fileobj=downdload_stream(url, callback=dl_callback)) as tar:
            # "paranoid" check
            names = (n.strip() for n in tar.getnames())
            assert not any(name.startswith("/") or ".." in name for name in names)
            assert all(name.startswith("factorio/") for name in names)

            with tempfile.TemporaryDirectory() as td:
                tar.extractall(td)
                shutil.move(os.path.join(td, "factorio"), server_dir)
        return cls(server_dir)

    def __init__(self, path):
        assert os.path.isdir(path)
        self.path = path
        self.settings = ServerSettings(self)

    def get_file(self, path):
        assert not path.startswith("/")
        return os.path.join(self.path, path)

    @property
    def executable(self):
        return self.get_file("bin/x64/factorio")

    @property
    def version(self):
        with open(self.get_file("data/base/info.json")) as f:
            return Version(json.load(f)["version"])

    def get_update_path(self, experimental):
        r = requests.get("https://updater.factorio.com/get-available-versions", timeout=10.0)
        data = r.json()["core-linux_headless64"]
        avaliable_updates = []
        for d in data:
            if set(d.keys()) == {"from", "to"}:
                avaliable_updates.append(Update(Version(d["from"]), Version(d["to"])))

        stable = Version([d["stable"] for d in data if "stable" in d][0])

        updates = []
        version = self.version
        for update in sorted(avaliable_updates):
            if (not experimental) and version == stable:
                break

            if update.old == version:
                updates.append(update)
                version = update.new

        return updates

    def apply_update(self, update_file_path):
        p = subprocess.run(
            [self.executable, "--apply-update", update_file_path],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )

        if p.returncode != 0:
            lines = p.stdout.decode().split("\n")
            for i, line in enumerate(lines):
                if "Applying update" in line:
                    return ErrorStatusMessage("Could not update:\n"+"\n".join(lines[i:-2]))
            return ErrorStatusMessage("Could not update. Factorio error log:\n"+"\n".join(lines))

    def update(self, experimental):
        update_path = self.get_update_path(experimental)
        if update_path != []:
            callback(EndStatusMessage("Updating to {}".format(update_path[-1].new)))

            with tempfile.TemporaryDirectory() as td:

                callback(StartStatusMessage("Downloading "))

                for i, update in enumerate(update_path):
                    callback(ProgressStatusMessage(0, i, len(update_path)))
                    with open(os.path.join(td, update.tempfilename), "wb") as tf:
                        downdload_stream(update.link, tf, callback)

                callback(ProgressStatusMessage(0, len(update_path), len(update_path)))

                callback(EndStatusMessage("[ok]"))

                callback(EndStatusMessage("Applying changes "))

                for i, update in enumerate(update_path):
                    callback(ProgressStatusMessage(2, i, len(update_path)))

                    update_error = self.apply_update(os.path.join(td, update.tempfilename))
                    if update_error:
                        callback(update_error)

                # callback(ProgressStatusMessage(2, len(update_path), len(update_path)))
                callback(EndStatusMessage("Update successful"))

    @property
    def saves(self):
        """A list of all saved games."""
        return

    def start(self):
        os.execl(self.executable, "")
