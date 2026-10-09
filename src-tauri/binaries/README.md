# Binaires externes (sidecars)

Exécutables embarqués avec l'application. Ils ne sont **pas** dans Git (trop lourds) :
`npm run ffmpeg` les télécharge (c'est aussi fait automatiquement avant `npm run dev` et `npm run build`).

- `ffmpeg-x86_64-pc-windows-msvc.exe` : FFmpeg 9.0.2, build **LGPL** de [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds)
  (sans x264 ni autre composant GPL ; H.264 via OpenH264, licence BSD). Version et empreinte SHA-256 figées
  dans `scripts/telecharger-ffmpeg.mjs`.
- `FFMPEG-LICENSE.txt` : licence de FFmpeg, livrée avec l'application.

Tauri exige que chaque binaire soit suffixé par la « target triple » de la plateforme
(`ffmpeg-aarch64-apple-darwin`, `ffmpeg-x86_64-unknown-linux-gnu`… pour macOS et Linux plus tard).
