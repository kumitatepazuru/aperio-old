pip download --no-deps --no-binary opencv-python-headless opencv-python-headless

tar xzf opencv-python-headless-*.tar.gz
mv opencv-python-headless-*/ opencv

cd opencv

# https://docs.opencv.org/4.x/db/d05/tutorial_config_reference.html
# windowsの場合はすでにCMAKE_ARGSが指定されているので、追記の形で対応

export CMAKE_ARGS="$CMAKE_ARGS \
    -DCMAKE_BUILD_TYPE=RELEASE \
    -DCPU_DISPATCH=AVX,AVX2 \
    -DWITH_OPENCL=ON \
    -DWITH_V4L=OFF \
    -DWITH_FFMPEG=OFF \
    -DWITH_GSTREAMER=ON \
    -DHIGHGUI_ENABLE_PLUGINS=OFF \
    -DWITH_GTK=OFF \
    -DWITH_WIN32API=OFF \
    -DBUILD_opencv_python3=ON \
    -DOPENCV_GAPI_GSTREAMER=ON \
    -DBUILD_SHARED_LIBS=ON \
    -DBUILD_opencv_world=ON \
    -DBUILD_opencv_gapi=OFF \
    -DBUILD_TESTS=OFF \
    -DBUILD_PERF_TESTS=OFF \
    -DBUILD_EXAMPLES=OFF"

export CMAKE_GENERATOR=Ninja
export CMAKE_BUILD_PARALLEL_LEVEL=$(nproc 2>/dev/null || echo 4)

# なぜかwindowsに対してのみplatform specificationを指定されるので消す
# aarch64はビルド対象外なので一旦無視
# https://github.com/opencv/opencv-python/issues/825#issuecomment-1503349866
sed -i 's/-DCMAKE_GENERATOR_PLATFORM=x64//g' setup.py

pip wheel . --verbose
mkdir -p ../src-tauri/binaries/wheels/
cp -v opencv_python_headless*.whl ../src-tauri/binaries/wheels/