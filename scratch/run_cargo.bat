@echo off
set "PATH=%TEMP%\llvm-mingw\llvm-mingw-20260616-ucrt-x86_64\bin;%PATH%"
"%USERPROFILE%\.cargo\bin\cargo.exe" %*
