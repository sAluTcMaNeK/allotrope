# Allotrope

Convertisseur de fichiers tout-en-un pour le bureau. Toutes les conversions se font en local : aucun fichier n'est envoyé sur internet.

Construit avec [Tauri v2](https://v2.tauri.app) (cœur en Rust) et [Svelte](https://svelte.dev) + TypeScript (interface).

## Structure

```
allotrope/
├── index.html            Page d'entrée de l'interface
├── src/                  Interface (Svelte + TypeScript)
├── public/               Fichiers statiques servis tels quels
├── src-tauri/            Cœur de l'application (Rust)
│   ├── src/              Code Rust (main.rs, lib.rs)
│   ├── capabilities/     Permissions accordées à l'interface
│   ├── icons/            Icônes de l'application
│   ├── binaries/         Exécutables embarqués (FFmpeg plus tard)
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
