from aperio_plugin import PluginManager
from aperio_plugin.plugin_base import MainPluginBase


@PluginManager.plugin
class AperioBasePlugin(MainPluginBase):
    def __init__(self, manager):
        super().__init__(manager)
        self.name = "AperioBasePlugin"
        self.display_name = "Aperio Base Plugin"
        self.description = "This is a plugin that provides basic filters/objects for Aperio."
        self.version = "1.0.0"
        self.author = "Aperio"
