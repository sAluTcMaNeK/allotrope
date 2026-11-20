# Développer Allotrope

Guide technique : installer l'environnement, compiler et publier une version. Pour la présentation du logiciel, voir le [README](README.md).


## Structure

```
allotrope/
├── index.html            Page d'entrée de l'interface
├── src/                  Interface (Svelte + TypeScript)
├── public/               Fichiers statiques servis tels quels
├── .github/workflows/    Vérifications, publication des versions, FFmpeg
├── src-tauri/            Cœur de l'application (Rust)
│   ├── src/              Code Rust (main.rs, lib.rs)
│   ├── capabilities/     Permissions accordées à l'interface
│   ├── icons/            Icônes de l'application
│   ├── binaries/         Exécutables embarqués (FFmpeg, téléchargé)
│   ├── windows/          Script de l'installateur (clic droit de l'explorateur)
│   ├── Cargo.toml        Dépendances Rust
│   └── tauri.conf.json   Configuration Tauri (fenêtre, installateur…)
├── package.json          Dépendances et scripts JavaScript
└── vite.config.ts        Serveur de développement de l'interface
```

## Installer l'environnement (Windows, une seule fois)

Dans un terminal PowerShell :

```powershell
winget install Git.Git
winget install OpenJS.NodeJS.LTS
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Fermer puis rouvrir le terminal, et vérifier :

```powershell
git --version
node --version
rustc --version
```

## Commandes

```powershell
npm install              # installe les dépendances JavaScript (la première fois)
npm run tauri dev        # lance l'application en mode développement
npm run tauri build      # produit l'installateur Windows (src-tauri/target/release/bundle/)
npm run tauri icon chemin/vers/logo.png   # génère toutes les icônes à partir d'un PNG carré (1024×1024 conseillé)
```

## Publier une version

Les installateurs Windows, macOS (puce Apple) et Linux sont construits par GitHub Actions
(`.github/workflows/release.yml`) quand une étiquette `v…` est envoyée :

1. Mettre le même numéro de version dans `package.json`, `src-tauri/Cargo.toml` et `src-tauri/tauri.conf.json`.
2. `git tag v0.2.0` puis `git push origin v0.2.0`.
3. Relire le brouillon de publication créé sur GitHub (onglet *Releases*) et le publier.
   Les copies d'Allotrope déjà installées proposeront alors la mise à jour.

### Préparatifs (une seule fois)

- **Secrets GitHub** (*Settings › Secrets and variables › Actions*) :
  - `TAURI_SIGNING_PRIVATE_KEY` : contenu du fichier `~/.tauri/allotrope-updater.key`
    (clé privée de signature des mises à jour ; **ne jamais la mettre dans Git**, la sauvegarder hors de l'ordinateur) ;
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` : laisser vide (clé sans mot de passe).
- **FFmpeg** : lancer une fois le workflow *FFmpeg* (onglet *Actions*), puis recopier l'empreinte macOS
  qu'il affiche dans `scripts/telecharger-ffmpeg.mjs`.
- **Signature du code Windows** (facultatif, recommandé pour éviter l'avertissement SmartScreen) :
  compte [Azure Trusted Signing](https://learn.microsoft.com/azure/trusted-signing/), puis secrets
  `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID` et variables `AZURE_SIGNING_ENDPOINT`,
  `AZURE_SIGNING_ACCOUNT`, `AZURE_SIGNING_PROFILE`. Sans eux, l'installateur est simplement non signé.

Pour produire localement un installateur avec les fichiers de mise à jour signés :

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$HOME\.tauri\allotrope-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build -- --config src-tauri/tauri.release.conf.json
```
