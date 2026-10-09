# Allotrope

Logiciel de bureau de conversion de fichiers tout-en-un. Le nom vient de la chimie : des allotropes sont un même élément sous des formes différentes (diamant / graphite). Ici, le même contenu change de format.

## Vision du produit

- Application **installable** (pas de version portable pour l'instant), légère et rapide.
- **Toute la conversion se fait en local** : aucun fichier n'est envoyé sur internet. C'est un argument central du produit.
- Interface **visuellement soignée** et moderne (style Windows 11 / Fluent, thème clair et sombre).
- **Windows d'abord**, puis macOS et Linux à partir du même code.

## Stack technique

- **Tauri v2** : framework d'application de bureau.
  - Cœur en **Rust** (`src-tauri/`) : lecture des fichiers, conversions, pilotage des moteurs externes.
  - Interface en **HTML / CSS / TypeScript** (`src/`), framework front à choisir (Svelte ou React).
- Installateur Windows généré par Tauri (NSIS ou MSI), mises à jour automatiques via le plugin updater de Tauri.
- Éditeur : VS Code (extensions rust-analyzer, Tauri, ESLint/Prettier, Error Lens).
- Versionnement Git + GitHub ; à terme, builds Windows/macOS/Linux via GitHub Actions.

## Moteurs de conversion

| Conversion                                       | Moteur                                                                         | Remarque                                                                                                    |
| ------------------------------------------------ | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| Images raster (JPEG, PNG, WebP, BMP…)            | crate Rust `image` (ou libvips)                                                | Intégré, sans dépendance externe                                                                            |
| PNG → SVG (vectorisation)                        | `vtracer` (Rust, licence MIT)                                                  | Gère la couleur                                                                                             |
| Vidéo / audio (MOV → MP4, extraction audio, GIF) | FFmpeg en **sidecar** Tauri                                                    | Utiliser une build **LGPL** (éviter x264/GPL si code fermé) ; envisager une build réduite aux codecs utiles |
| Word → PDF                                       | Automatisation de Microsoft Word (COM) si installé, sinon LibreOffice headless | C'est le point le plus délicat                                                                              |
| HEIC / AVIF                                      | libheif / libavif                                                              | Plus tard                                                                                                   |

## Fonctionnalités prévues

- Grande zone de glisser-déposer avec détection automatique du type de fichier et proposition des formats de sortie possibles.
- Traitement par lots : file d'attente, progression par fichier, conversions en parallèle.
- Préréglages (« Image pour le web », « Vidéo légère pour email », etc.).
- Aperçu avant / après avec la taille gagnée.
- Intégration au clic droit de l'explorateur Windows (« Convertir avec Allotrope »).

## Feuille de route

1. **MVP** : fenêtre Tauri, glisser-déposer, conversion d'images (JPEG / PNG / WebP) sans dépendance externe.
2. **v0.2** : PNG → SVG (vtracer), traitement par lots, préréglages.
3. **v0.3** : module vidéo FFmpeg avec progression en temps réel.
4. **v0.4** : module documents (Word → PDF, fusion PDF, PDF → images).
5. **v1.0** : polissage de l'interface, signature de code (ex. Azure Trusted Signing), site de présentation, builds macOS et Linux.

## Prérequis de développement (Windows)

- Rust via `rustup`
- Node.js LTS
- Visual Studio Build Tools (composant « Développement Desktop en C++ »)
- WebView2 (déjà présent sur Windows 10/11)
- Git

## Consignes pour l'assistant

- Le porteur du projet débute avec Rust et Tauri : expliquer les choix et les commandes, avancer par petites étapes vérifiables.
- Répondre en français.
- Garder le code multiplateforme dès le départ (chemins avec `std::path`, pas de code spécifique Windows hors modules dédiés).
- Privilégier des dépendances aux licences permissives (MIT / Apache / LGPL) ; signaler toute dépendance GPL.

## Identité visuelle — à respecter pour toute interface

@DESIGN.md

- `maquettes/` : maquettes HTML de référence (`accueil.html`, `file-attente.html`). Les reproduire fidèlement sous forme de composants réutilisables, sans copier le HTML tel quel.
- `assets/` : logo et icônes officiels. Ne jamais redessiner ni recomposer le logo.
- Toutes les couleurs passent par les variables CSS de `src/styles/tokens.css` ; aucune couleur codée en dur.
- Pour toute interface, suivre `DESIGN.md` et les maquettes ; signaler tout écart plutôt que d'improviser un style.
