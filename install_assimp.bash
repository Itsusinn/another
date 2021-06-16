#!/bin/bash

path_apt_sourcelist=/etc/apt/sources.list
path_assimp_repo=/tmp/assimp
path_assimp_build="${path_assimp_repo}/build"

if ! grep -q "apt.llvm.org" ${path_apt_sourcelist}; then
	bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"
fi

apt install -y git cmake ninja-build

if [ ! -d ${path_assimp_repo} ]; then
	git clone --depth 1 --branch v5.0.1 https://github.com/assimp/assimp.git ${path_assimp_repo}
fi

if [ ! -d ${path_assimp_build} ]; then
    mkdir ${path_assimp_build}
	cd ${path_assimp_build}
	cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=/usr/bin/clang++-11 -DCMAKE_C_COMPILER=/usr/bin/clang-11 -DCMAKE_INSTALL_PREFIX=/usr -G Ninja ..
	cd ..
fi

cd ${path_assimp_build}
ninja
ninja install


