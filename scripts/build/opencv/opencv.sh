pip download --no-deps --no-binary opencv-python-headless opencv-python-headless

tar xzf opencv-python-headless-*.tar.gz
mv opencv-python-headless-*/ opencv

cd opencv

# https://docs.opencv.org/4.x/db/d05/tutorial_config_reference.html
export CMAKE_ARGS="-D CMAKE_BUILD_TYPE=RELEASE \
    -D CPU_DISPATCH=AVX,AVX2 \
    -D WITH_OPENCL=ON \
    -D WITH_V4L=OFF \
    -D WITH_FFMPEG=OFF \
    -D WITH_GSTREAMER=ON \
    -D HIGHGUI_ENABLE_PLUGINS=OFF \
    -D WITH_GTK=OFF \
    -D WITH_WIN32API=OFF \
    -D BUILD_opencv_python3=ON"

MAKEFLAGS="-j$(nproc)" pip wheel . --verbose

mkdir -p ../src-tauri/binaries/wheels/
cp -v opencv_python_headless*.whl ../src-tauri/binaries/wheels/