<p align="center">
  <img src="assets/allotrope-logo.svg" alt="Allotrope" width="380">
</p>

<p align="center">
  <strong>Le même contenu, sous toutes ses formes.</strong><br>
  Convertissez et compressez vos images, vidéos, sons et PDF en un glisser-déposer,<br>
  sans qu'aucun fichier ne quitte votre ordinateur.
</p>

<p align="center">
  <a href="https://github.com/sAluTcMaNeK/allotrope/releases/latest"><strong>⬇ Télécharger Allotrope</strong></a>
  &nbsp;·&nbsp; Gratuit &nbsp;·&nbsp; Windows, macOS, Linux
</p>

<!-- Captures d'écran : déposer les images dans docs/captures/ puis décommenter.
<p align="center">
  <img src="docs/captures/accueil.png" alt="Accueil d'Allotrope : zone de dépôt" width="800">
</p>
-->

## 100 % local

Les convertisseurs en ligne envoient vos fichiers sur leurs serveurs. Allotrope fait tout **sur votre
ordinateur** : vos photos, contrats et vidéos de famille restent sur votre disque. L'application
fonctionne hors connexion ; internet ne sert qu'à vérifier, si vous le souhaitez, qu'une nouvelle
version est disponible.

## Ce que fait Allotrope

| | |
|---|---|
| **Images** | JPG, PNG, WEBP, TIFF, BMP, GIF, ICO, JPEG XL… Conversion, compression réglable (taux ou taille cible), redimensionnement. **PNG → SVG** : vectorisation d'un logo. |
| **Vidéo** | MOV, MP4, MKV, WEBM, GIF… Conversion, extraction du son, compression jusqu'à une taille cible (pour un e-mail par exemple), avec suivi en temps réel. |
| **Audio** | MP3, M4A, OPUS, OGG, FLAC, WAV… Conversion avec suivi de l'avancement. |
| **Outils PDF** | Fusionner, réorganiser ou supprimer des pages, compresser, extraire les images, PDF en images, PDF en Word / ODT, images en PDF, Word en PDF. |

Et aussi :

- **Traitement par lots** : déposez des dizaines de fichiers, ils sont traités en parallèle.
- **Aperçu avant / après** avec le poids gagné.
- **Préréglages** : enregistrez vos réglages favoris et retrouvez-les en un clic.
- **Clic droit « Convertir avec Allotrope »** dans l'explorateur Windows.
- **Vos originaux ne sont jamais modifiés** : chaque traitement crée un nouveau fichier.
- **Mises à jour automatiques**, signées et vérifiées avant installation (désactivables).

## Installation

Téléchargez le fichier correspondant à votre système sur la
[page de la dernière version](https://github.com/sAluTcMaNeK/allotrope/releases/latest) :

| Système | Fichier |
|---|---|
| **Windows 10 et 11** (64 bits) | `Allotrope_…_x64-setup.exe` |
| **Windows, version portable** | `Allotrope_…_x64_portable.exe` (voir ci-dessous) |
| **macOS 11 ou plus récent** (Mac à puce Apple : M1, M2…) | `Allotrope_…_aarch64.dmg` |
| **Linux** (64 bits) | `Allotrope_…_amd64.AppImage` (toutes distributions) |

### Version portable (Windows)

Un seul fichier à lancer depuis n'importe où (clé USB, dossier personnel…), sans installation.
Elle est **plus légère** car elle n'embarque pas FFmpeg : **images et outils PDF uniquement, ni audio ni
vidéo**. Pas de clic droit dans l'explorateur, et les mises à jour se font en téléchargeant le nouveau
fichier. Elle est indépendante de la version installée : chacune garde ses propres réglages, rangés
sur l'ordinateur utilisé.

> **Windows affiche « Windows a protégé votre ordinateur » ?** L'installateur n'est pas encore
> signé par un certificat payant. Cliquez sur *Informations complémentaires*, puis *Exécuter quand même*.
>
> **macOS refuse d'ouvrir l'application ?** Faites un clic droit sur Allotrope dans le dossier
> Applications, puis *Ouvrir*.

## Questions fréquentes

**Faut-il Microsoft Word pour convertir un document en PDF ?**
Allotrope utilise Word s'il est installé, sinon [LibreOffice](https://fr.libreoffice.org) (gratuit).
Toutes les autres fonctions marchent sans logiciel supplémentaire.

**Où sont enregistrés les fichiers produits ?**
Par défaut dans `Documents/Allotrope`. Vous pouvez choisir un autre dossier, ou le dossier d'origine
de chaque fichier, dans les Paramètres.

**Comment signaler un problème ou proposer une idée ?**
Ouvrez un [ticket (*issue*)](https://github.com/sAluTcMaNeK/allotrope/issues).

## Composants open source

Allotrope s'appuie notamment sur [Tauri](https://tauri.app), [Svelte](https://svelte.dev),
[FFmpeg](https://ffmpeg.org) (build LGPL, livrée comme programme séparé avec sa licence),
[pdf.js](https://mozilla.github.io/pdf.js/), [vtracer](https://github.com/visioncortex/vtracer),
[lopdf](https://github.com/J-F-Liu/lopdf), la police Nunito et les icônes [Lucide](https://lucide.dev).
La liste complète figure dans l'application, rubrique *Paramètres › À propos*.

## Contribuer

Le guide pour compiler Allotrope et publier une version se trouve dans [DEVELOPPEMENT.md](DEVELOPPEMENT.md).

## Licence

Allotrope est distribué sous [licence MIT](LICENSE).
