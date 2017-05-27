class Size(str):
    VALUES = {"none", "very-low", "low", "normal", "high", "very-high"}
    def __init__(self):
        assert self in self.VALUES

    @classmethod
    def __getattribute__(self, attr):
        if attr.lower() in self.VALUES:
            return attr.lower()
        raise AttributeError

class AutoplaceControl(object):
    KEYS = {"coal", "copper-ore", "crude-oil", "enemy-base", "iron-ore", "stone", "uranium-ore"}

    @classmethod
    def normal(cls, key):
        return cls(key, Size.NORMAL, Size.NORMAL, Size.NORMAL)

    def __init__(self, key, frequency, size, richness):
        assert frequency in Size.VALUES
        assert size in Size.VALUES
        assert richness in Size.VALUES

        self.key = key
        self.data = {"frequency": frequency, "size": size, "richness": richness}

class TerrainSettings(object):
    def __init__(self, ap_controls, water=Size.NORMAL, starting_area=Size.NORMAL, segmentation=Size.NORMAL, seed=None):
        assert {c.key for c in ap_controls} == AutoplaceControl.KEYS
        assert all([isinstance(v, AutoplaceControl) for v in ap_controls])
        assert isinstance(water, Size)
        assert isinstance(starting_area, Size)
        assert isinstance(segmentation, Size)
        assert seed is None or (isinstance(seed, int) and seed >= 0)

        self.water = water
        self.starting_area = starting_area
        self.segmentation = segmentation
        self.seed = seed
        self.autoplace_controls = {c.key: c.data for c in ap_controls}

class MapSize(object):
    INFINITE = MapSize(0, 0)

    def __init__(self, width, height):
        assert isinstance(width, int) and width >= 0
        assert isinstance(height, int) and height >= 0
        self.width = width
        self.height = height

class MapGenSettings(object):
    def __init__(self, size, terrain, peaceful_mode=False):
        assert isinstance(size, MapSize)
        assert isinstance(terrain, TerrainSettings)
        assert isinstance(peaceful_mode, bool)

        self.data = {
            "peaceful_mode": peaceful_mode,
            "width": size.width,
            "height": size.height,
            "terrain_segmentation": terrain.segmentation,
            "starting_area": terrain.starting_area,
            "autoplace_controls": terrain.autoplace_controls,
            "water": terrain.water,
            "seed": terrain.seed
        }

    def save(self, path):
        with open(path, "w") as f:
            json.dump(self.data, f)
