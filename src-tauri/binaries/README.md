# Binaires externes (sidecars)

Exécutables embarqués avec l'application. Ils ne sont **pas** dans Git (trop lourds) :
`npm run ffmpeg` les télécharge (c'est aussi fait automatiquement avant `npm run dev` et `npm run build`).

- `ffmpeg-<cible>` : FFmpeg 9.0.2, build **LGPL** (sans x264 ni autre composant GPL).
  - Windows (`x86_64-pc-windows-msvc`) et Linux (`x86_64-unknown-linux-gnu`) : builds de
    [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) ; H.264 via OpenH264 (licence BSD).
  - macOS (`aarch64-apple-darwin`) : compilé par le workflow `.github/workflows/ffmpeg.yml` ;
    H.264 via VideoToolbox (encodeur d'Apple).
  - Archives et empreintes SHA-256 figées dans `scripts/telecharger-ffmpeg.mjs`. Copie permanente dans
    la publication GitHub `ffmpeg-9.0.2` du dépôt (BtbN supprime ses anciennes publications).
- `FFMPEG-LICENSE.txt` : licence de FFmpeg, livrée avec l'application.

Tauri exige que chaque binaire soit suffixé par la « target triple » de la plateforme.
