import requests

def latest(experimental=False):
    r = requests.get("https://updater.factorio.com/get-available-versions", timeout=10.0)
    data = r.json()["core-linux_headless64"]

    if experimental:
        return max(Version(d["to"]) for d in data if "to" in d)
    else:
        return Version([d["stable"] for d in data if "stable" in d][0])

def resolve(version):
    assert isinstance(version, str)

    if version in ("stable", "experimental"):
        return latest()
    else:
        return Version(version)

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
            },
            timeout=10.0
        )
        return r.json()[0]

    @property
    def tempfilename(self):
        return "update_{}_{}".format(self.old, self.new)

    def __eq__(self, other):
        return self.old == other.old and self.new == other.new

    def __lt__(self, other):
        return self.new < other.new

    def __gt__(self, other):
        return self.new > other.new

    def __repr__(self):
        return "Update({}, {})".format(self.old, self.new)
