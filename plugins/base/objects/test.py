from typing import Literal

import cv2
import numpy as np

from aperio_plugin.plugin_base.generator_base import ObjectGeneratorBase


class TestObject(ObjectGeneratorBase):
    """
    テストフレームを生成するオブジェクトプラグイン。OpencvとGStreamerのテストソースを利用してフレームを生成する。
    """

    frame = cv2.VideoCapture("videotestsrc ! videoconvert ! appsink")  # GStreamerのテストソースを利用

    def __init__(self):
        super().__init__()
        self.name = "TestObject"
        self.display_name = "Test Object"
        self.description = "This is a test object that generates frames using OpenCV and GStreamer videotestsrc."

    def generate(self, obj_args: dict, shape: tuple[int, int, Literal[1, 3, 4]]) -> np.ndarray:
        ret, img = self.frame.read()
        if not ret:
            raise RuntimeError("Failed to read frame from videotestsrc")

        img = cv2.resize(img, (shape[1], shape[0]))  # 指定された形状にリサイズ
        if shape[2] == 1:
            img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)  # グレースケールに変換
            img = img[:, :, np.newaxis]  # チャンネル次元を追加
        elif shape[2] == 4:
            img = cv2.cvtColor(img, cv2.COLOR_BGR2BGRA)  # BGRAに変換

        return img
