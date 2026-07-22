# Sidecar binaries

Place compiled `audiocpp_server` executable here.

## Naming convention (Tauri v2 sidecar)

The binary must be named with the Rust target triple suffix:

- **Windows (GNU)**: `audiocpp_server-x86_64-pc-windows-gnu.exe`
- **Windows (MSVC)**: `audiocpp_server-x86_64-pc-windows-msvc.exe`

## How to compile

```bash
cd audio.cpp
mkdir build && cd build
cmake .. -DGGML_CUDA=ON  # or OFF for CPU-only
cmake --build . --config Release
```

Copy the output binary here with the correct name.

## Alternative: symlink / copy

During development, you can copy any existing `audiocpp_server.exe` here and rename it:

```powershell
copy E:\path\to\audiocpp_server.exe .\audiocpp_server-x86_64-pc-windows-gnu.exe
```
