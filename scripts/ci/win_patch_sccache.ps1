# /Zi (implies also /DEBUG) flag is removed as sccache is not supporting it
# Replacing with /Z7 (as in https://gitlab.kitware.com/cmake/cmake/-/issues/22529) is tricky as well
# as CMakeLists.txt are referencing .pdb files which would not be generated
# We therefore disable pdb generation alltogether, and disable installation of those in cmake
            
patch vendor/support/windows/cmake/c-flag-overrides.cmake scripts/ci/win_disable_debug_c_libcec.patch
patch vendor/src/platform/support/windows/cmake/c-flag-overrides.cmake scripts/ci/win_disable_debug_c_platform.patch
patch vendor/support/windows/cmake/cxx-flag-overrides.cmake scripts/ci/win_disable_debug_cxx_libcec.patch
patch vendor/src/platform/support/windows/cmake/cxx-flag-overrides.cmake scripts/ci/win_disable_debug_cxx_platform.patch

# Remove pdb references from cmake
# pdb is not generated with /Zi
patch vendor/src/libcec/CMakeLists.txt scripts/ci/win_disable_pdb_install_libcec.patch
patch vendor/src/platform/CMakeLists.txt scripts/ci/win_disable_pdb_install_platform.patch

# Experimental: try to ensure sccache use with CMAKE_C_COMPILER_LAUNCHER and CMAKE_CXX_COMPILER_LAUNCHER
# Not sure if this is really needed
patch vendor/CMakeLists.txt scripts/ci/win_cmake_launcher.patch
patch vendor/support/windows/cmake/generate.cmd scripts/ci/win_cmake_launcher2.patch