cargo build --release
copy /y .\target\i686-pc-windows-msvc\release\haine.dll .\
pwsh.exe .\src\extract_surfaces.ps1
