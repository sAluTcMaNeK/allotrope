# Binaires externes (sidecars)

Ce dossier accueillera les exécutables embarqués avec l'application, comme FFmpeg (prévu en v0.3).

Tauri exige que chaque binaire soit suffixé par la « target triple » de la plateforme, par exemple :

- `ffmpeg-x86_64-pc-windows-msvc.exe`
- `ffmpeg-aarch64-apple-darwin`
- `ffmpeg-x86_64-unknown-linux-gnu`

Rappel : utiliser une build FFmpeg **LGPL** (sans x264 ni autres composants GPL).
