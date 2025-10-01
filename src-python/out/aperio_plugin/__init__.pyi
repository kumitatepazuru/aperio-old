from .plugin_base import MainPluginBase, SubPluginBase
from .plugin_base.generator_base import FilterGeneratorBase, ObjectGeneratorBase
from _typeshed import Incomplete
from typing import Callable

class PluginManager:
    """
    フレーム生成のプラグイン群を管理するクラス。このクラスは、フレーム生成系プラグイン管理の他、フレーム生成を行うためのインターフェースを提供する。
    """
    plugins: dict[str, MainPluginBase]
    object_plugins: dict[str, ObjectGeneratorBase]
    filter_plugins: dict[str, FilterGeneratorBase]
    data_dir: Incomplete
    plugin_dir_name: Incomplete
    def __init__(self, data_dir: str, plugin_dir_name: str = 'plugins') -> None:
        '''
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
        '''
    @classmethod
    def plugin(cls, func: type[MainPluginBase]) -> Callable:
        """
        オブジェクト生成プラグインを登録するためのデコレーター。関数に対して使用し、オブジェクト生成プラグインを登録する。

        Args:
            func (type[MainPluginBase]): オブジェクト生成プラグインのクラス

        Returns:
            Callable: 登録されたオブジェクト生成プラグインのクラス
        """
    def register_sub_plugin(self, plugin: SubPluginBase) -> None:
        """
        サブプラグインを登録するメソッド。サブプラグインはObjectGeneratorBaseまたはFilterGeneratorBaseのいずれかを継承している必要がある。

        Args:
            plugin (SubPluginBase): 登録するサブプラグインのインスタンス
        """
    def add_plugin(self, plugin_dir: str) -> bool:
        """
        プラグインを追加するメソッド。
        指定されたディレクトリからプラグインを追加する。既に同じ名前のプラグインが存在する場合は、__init__.pyのハッシュ値を比較して異なる場合のみ更新する。

        Args:
            plugin_dir (str): 追加するプラグインのディレクトリのパス

        Returns:
            bool: プラグインが正常に追加または更新された場合はTrue、それ以外の場合はFalse
        """
