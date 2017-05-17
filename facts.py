import os
import sys
import json
import tempfile
import tarfile
import requests
import subprocess

def downdload_stream(url, existing_file=None):
    """Downloads and returns a temporary file containing the file."""
    if existing_file is None:
        tf = tempfile.TemporaryFile(mode="w+b")
    else:
        tf = existing_file

    r = requests.get(url, stream=True)

    try:
        for chunk in r.iter_content(chunk_size=1024):
            if chunk: # filter out keep-alive new chunks
                tf.write(chunk)
    except:
        tf.close()
        raise
    tf.seek(0)
    return tf

class Version(tuple):
    def __init__(self, data):
        if isinstance(data, str):
            self._parts = tuple([int(p) for p in data.split(".")])
        elif isinstance(data, (list, tuple)):
            self._parts = tuple(data)
        else:
            raise TypeError

        if len(self._parts) != 3:
            raise ValueError

        if not all(isinstance(p, int) for p in self._parts):
            raise ValueError

    @property
    def major(self):
        return self[0]

    @property
    def minor(self):
        return self[1]

    @property
    def patch(self):
        return self[2]

    def __str__(self):
        return ".".join(str(p) for p in self._parts)

    def __repr__(self):
        return "Version({})".format(str(self))

class Update(object):
    def __init__(self, old, new):
        self.old = old
        self.new = new

    @property
    def link(self):
        r = requests.get(
            "https://updater.factorio.com/get-download-link",
            params={
                "from": str(self.old),
                "to": str(self.new),
                "package": "core-linux_headless64"
            }
        )
        return r.json()[0]

    @property
    def foldername(self):
        return "core-linux_headless64-{}-{}-update".format(self.old, self.new)

    def __eq__(self, other):
        return self.old == other.old and self.new == other.new

    def __lt__(self, other):
        return self.new < other.new

    def __gt__(self, other):
        return self.new > other.new

    def __repr__(self):
        return "Update({}, {})".format(self.old, self.new)

class Server(object):
    @classmethod
    def create(cls, server_dir, version):
        """Creates a new server dir with given version."""
        assert isinstance(version, Version)
        assert not os.path.exists(server_dir)

        url = "https://www.factorio.com/get-download/{}/headless/linux64".format(str(version))
        with tarfile.open(fileobj=downdload_stream(url), mode="r:xz") as tar:
            # paranoid check
            names = (n.strip() for n in tar.getnames())
            assert not any(name.startswith("/") or ".." in name for name in names)
            assert all(name.startswith("factorio/") for name in names)

            with tempfile.TemporaryDirectory() as td:
                tar.extractall(td)
                os.rename(os.path.join(td, "factorio"), server_dir)
        return cls(server_dir)

    def __init__(self, path):
        assert os.path.isdir(path)
        self.path = path

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

    def get_update_path(self, experimental=True):
        r = requests.get("https://updater.factorio.com/get-available-versions")
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

    def resolve_update_path(self, upath):
        upath = upath.replace("__PATH__bin__", "bin")
        upath = upath.replace("__PATH__read-data__", "data")
        return self.get_file(upath)

    def apply_update(self, update):
        with tempfile.NamedTemporaryFile() as tf:
            downdload_stream(update.link, tf)
            subprocess.run([self.executable, "--apply-update", tf.name])

    def update(self, experimental=True):
        """Installs package with given version on top of existing server."""

        for update in self.get_update_path(experimental):
            self.apply_update(update)

if not os.path.exists("tuska"):
    Server.create("tuska", Version("0.15.9"))
Server("tuska").update()


# facts create tuska
# facts create tuska stable
# facts create tuska experimental
# facts create tuska 0
# facts create tuska 0.15
# facts create tuska 0.15.11

# facts update tuska
# facts update tuska stable
# facts update tuska experimental
