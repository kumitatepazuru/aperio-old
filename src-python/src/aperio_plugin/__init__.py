import glob
import hashlib
import os.path
import shutil
from typing import Callable

from .plugin_base import MainPluginBase, SubPluginBase
from .plugin_base.generator_base import FilterGeneratorBase, ObjectGeneratorBase


class PluginManager:
    """
    フレーム生成のプラグイン群を管理するクラス。このクラスは、フレーム生成系プラグイン管理の他、フレーム生成を行うためのインターフェースを提供する。
    """

    __plugins: dict[str, type[MainPluginBase]] = {}  # 登録されたプラグインのクラスを保持する辞書
    plugins: dict[str, MainPluginBase] = {}  # 登録されたプラグインのインスタンスを保持する辞書
    object_plugins: dict[str, ObjectGeneratorBase] = {}
    filter_plugins: dict[str, FilterGeneratorBase] = {}

    def __init__(self, data_dir: str, plugin_dir_name="plugins"):
        """
        フレーム生成マネージャーの初期化をする。data_dirはデータディレクトリのパス(通常はget_data_dirによるもの)、plugin_dir_nameはプラグインディレクトリの名前を指定する。
        プラグインディレクトリの構造は以下のようになることを想定している。

        data_dir/
            plugins/
                plugin1/
                    __init__.py
                    (他のプラグインファイル)
                plugin2/
                    __init__.py
                    (他のプラグインファイル)
                ...

        Args:
            data_dir (str): データディレクトリのパス
            plugin_dir_name (str): プラグインディレクトリの名前 (デフォルト: "plugins")
        """

        self.data_dir = data_dir
        self.plugin_dir_name = plugin_dir_name

        dirs = glob.glob(f"{self.data_dir}/{self.plugin_dir_name}/*")

        # プラグインディレクトリ内の各プラグインをインポートしてデコレータを実行する
        # これにより、self.pluginsにプラグインが自動的に登録される
        for d in dirs:
            plugin_name = d.split("/")[-1]
            if not os.path.exists(f"{d}/__init__.py"):
                print(f"Plugin {plugin_name} does not have an __init__.py file. Skipping.")
                continue
            __import__(f"{self.plugin_dir_name}.{plugin_name}")

        for name, plugin_cls in self.__plugins.items():
            plugin_instance = plugin_cls(self)
            self.plugins[name] = plugin_instance

        print("Loaded Plugins:")
        print("\n".join(list(map(lambda n: f"{n[0]}(Object): {n[1].get_info()}", self.object_plugins.items()))))
        print("\n".join(list(map(lambda n: f"{n[0]}(Filter): {n[1].get_info()}", self.filter_plugins.items()))))

    @classmethod
    def plugin(cls, func: type[MainPluginBase]) -> Callable:
        """
        オブジェクト生成プラグインを登録するためのデコレーター。関数に対して使用し、オブジェクト生成プラグインを登録する。

        Args:
            func (type[MainPluginBase]): オブジェクト生成プラグインのクラス

        Returns:
            Callable: 登録されたオブジェクト生成プラグインのクラス
        """

        if not issubclass(func, MainPluginBase):
            raise TypeError("The decorated class must be a subclass of MainPluginBase")

        cls.__plugins[func.__name__] = func

        def wrapper(*_args, **_kwargs):
            raise RuntimeError("This function is a plugin for Aperio and cannot be called directly")

        return wrapper

    def register_sub_plugin(self, plugin: SubPluginBase) -> None:
        """
        サブプラグインを登録するメソッド。サブプラグインはObjectGeneratorBaseまたはFilterGeneratorBaseのいずれかを継承している必要がある。

        Args:
            plugin (SubPluginBase): 登録するサブプラグインのインスタンス
        """

        if isinstance(plugin, ObjectGeneratorBase):
            self.object_plugins[plugin.name] = plugin
        elif isinstance(plugin, FilterGeneratorBase):
            self.filter_plugins[plugin.name] = plugin
        else:
            raise TypeError("The plugin must be a subclass of ObjectGeneratorBase or FilterGeneratorBase")

    def check_plugin_exists(self, plugin_name: str) -> bool:
        """
        指定された名前のプラグインが存在するかどうかを確認するメソッド。

        Args:
            plugin_name (str): 確認するプラグインの名前

        Returns:
            bool: プラグインが存在する場合はTrue、存在しない場合はFalse
        """
        return plugin_name in self.plugins

    def add_plugin(self, plugin_dir: str) -> bool:
        """
        プラグインを追加するメソッド。
        指定されたディレクトリからプラグインを追加する。既に同じ名前のプラグインが存在する場合は、__init__.pyのハッシュ値を比較して異なる場合のみ更新する。

        Args:
            plugin_dir (str): 追加するプラグインのディレクトリのパス

        Returns:
            bool: プラグインが正常に追加または更新された場合はTrue、それ以外の場合はFalse
        """
        # TODO: URLからのダウンロードや、zipファイルの解凍などもここで行う

        if not os.path.exists(plugin_dir) or not os.path.isdir(plugin_dir):
            print(f"Plugin directory {plugin_dir} does not exist.")
            return False

        plugin_name = plugin_dir.split("/")[-1]
        if plugin_name in self.plugins:
            # 既に登録されている場合は__init__.pyのハッシュ値を比較して、異なる場合のみ更新する
            # TODO: バージョン確認で新しければアップデート、古ければ確認みたいにしたい
            print(f"Plugin {plugin_name} is already registered. Trying to update to specified version.")
            if not os.path.exists(f"{plugin_dir}/__init__.py"):
                print(f"Plugin {plugin_name} does not have an __init__.py file. Skipping.")
                return False

            with open(f"{plugin_dir}/__init__.py", "rb") as f:
                new_hash = hashlib.sha256(f.read()).hexdigest()
                with open(f"{self.data_dir}/{self.plugin_dir_name}/{plugin_name}/__init__.py", "rb") as ef:
                    existing_hash = hashlib.sha256(ef.read()).hexdigest()
                    if new_hash == existing_hash:
                        print(f"Plugin {plugin_name} is completely same. Skipping.")
                        return True

        shutil.copytree(plugin_dir, f"{self.data_dir}/{self.plugin_dir_name}/{plugin_name}", dirs_exist_ok=True)
        print(f"Plugin {plugin_name} has been added/updated.")
        return True
