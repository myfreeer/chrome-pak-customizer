mkdir build_mingw64
cd build_mingw64
set MSYSTEM=MINGW64
C:\msys64\usr\bin\pacman -S --noconfirm --needed --ask=20 mingw-w64-x86_64-ninja
C:\msys64\usr\bin\bash -lc "cd \"$APPVEYOR_BUILD_FOLDER\build_mingw64\" && exec cmake -GNinja .."
C:\msys64\usr\bin\bash -lc "cd \"$APPVEYOR_BUILD_FOLDER\build_mingw64\" && exec ninja"
move /Y pak.exe ../pak_mingw64.exe
cd ..

mkdir build_mingw32
cd build_mingw32
set MSYSTEM=MINGW32
C:\msys64\usr\bin\pacman -S --noconfirm --needed --ask=20 mingw-w64-i686-ninja
C:\msys64\usr\bin\bash -lc "cd \"$APPVEYOR_BUILD_FOLDER\build_mingw32\" && exec cmake -GNinja .."
C:\msys64\usr\bin\bash -lc "cd \"$APPVEYOR_BUILD_FOLDER\build_mingw32\" && exec ninja"
move /Y pak.exe ../pak_mingw32.exe
cd ..

appveyor.exe PushArtifact pak_mingw64.exe
appveyor.exe PushArtifact pak_mingw32.exe