# Allotrope — Charte visuelle

Ce document décrit l'identité visuelle d'Allotrope. Toute interface de l'application doit s'y conformer.
Les maquettes de référence sont dans `maquettes/` (ouvrir dans un navigateur) et les fichiers du logo dans `assets/`.

## Principes

1. **Mode sombre par défaut.** Le fond principal est Abysse ; le mode clair viendra plus tard (tokens à définir).
2. **Dominante bleue.** Les bleus occupent l'essentiel de l'écran ; les couleurs chaudes ne sont que des touches.
3. **Coins très arrondis, contours marqués.** Pas d'ombres portées, pas de dégradés : le relief vient des contours et des aplats.
4. **Une couleur par famille de fichiers**, utilisée partout de la même façon (pastilles, badges, barres de progression).
5. **Un seul appel à l'action chaud par écran** : le bouton principal en Braise.

## Couleurs

À déclarer comme variables CSS globales (`src/styles/tokens.css`) et à utiliser exclusivement via ces variables.

```css
:root {
  /* Bleus — dominante */
  --abysse:   #0A1A3F;  /* fond de l'application */
  --surface:  #0E2150;  /* cartes, panneaux, zone de dépôt */
  --surface-active: #12296A; /* carte sélectionnée */
  --ligne:    #1C2E5C;  /* bordures discrètes, séparateurs */
  --ligne-forte: #2B4278; /* bordures de boutons secondaires, pointillés */
  --cobalt:   #1F5BFF;  /* couleur de marque, élément actif */
  --azur:     #4C9BFF;  /* images, accents bleus clairs */
  --lagon:    #14C4E6;
  --givre:    #EEF5FF;  /* texte principal, contours marqués */
  --texte-secondaire: #A9BCE0;
  --lien:     #8EC2FF;

  /* Accents — par petites touches */
  --braise:    #FF6A3D; /* documents, bouton principal */
  --amethyste: #7B3FF2; /* vidéo */
  --amethyste-clair: #9A72F6; /* barre de progression vidéo */
  --ambre:     #FFB020; /* audio */
  --pivoine:   #FF4F7B; /* réserve (erreurs secondaires, nouveautés) */
  --menthe:    #3DDC97; /* succès, « 100 % local » */
  --menthe-fond: #123A3A; /* fond des pastilles de succès */
}
```

### Couleur par famille de fichiers

| Famille | Couleur | Texte sur la couleur |
|---|---|---|
| Images (JPG, PNG, WEBP, SVG, HEIC…) | `--azur` | `--abysse` |
| Vidéo (MP4, MOV, MKV, GIF…) | `--amethyste` | blanc |
| Audio (MP3, WAV, FLAC…) | `--ambre` | `--abysse` |
| Documents (DOCX, PDF, ODT…) | `--braise` | `--abysse` |

### États

- Succès : texte `--menthe` sur fond `--menthe-fond`, icône coche.
- En cours : barre de progression de la couleur de la famille.
- En attente : texte `--texte-secondaire`.
- Erreur : `--braise` (à confirmer quand l'écran d'erreur sera dessiné).

**Contraste :** jamais de texte blanc sur Braise, Ambre ou Azur (contraste insuffisant) ; utiliser `--abysse`.

## Typographie

- Police de l'interface : **Nunito** (Google Fonts, licence OFL), graisses 500 à 900. L'embarquer localement dans l'application (pas de chargement réseau : l'application doit fonctionner hors ligne).
- Le mot « allotrope » du logo est **dessiné**, ce n'est pas une police : toujours utiliser les SVG de `assets/`, ne jamais le recomposer en texte.

| Usage | Taille | Graisse |
|---|---|---|
| Titre d'écran | 28–30 px | 900 |
| Titre de section | 18 px | 800 |
| Nom de fichier, libellé de bouton | 15 px | 800 |
| Texte courant | 14–15 px | 600–700 |
| Métadonnées (poids, durée) | 13 px | 600, `--texte-secondaire` |
| Sur-titre (« TYPES DE FICHIERS ») | 12 px | 800, majuscules, espacement 1,2 px |

## Formes

- **Rayons :** 999 px (boutons principaux, pastilles), 32 px (zone de dépôt), 22–26 px (cartes, panneaux), 14–16 px (boutons carrés, badges, éléments de navigation), 12 px (segments).
- **Bordures :** 2 px `--ligne` par défaut ; **3 px `--givre`** pour l'élément mis en avant (bouton principal, carte sélectionnée). C'est la signature visuelle de la marque, elle reprend le contour blanc du logo.
- **Pas d'ombre portée, pas de dégradé, pas de flou.**
- Espacements : multiples de 4 px (4, 8, 12, 16, 24, 28, 32, 40).
- Toute zone cliquable fait au moins **44 × 44 px**.

## Icônes

- Style trait, épaisseur 2,4 px, extrémités et angles arrondis, `stroke="currentColor"`.
- Bibliothèque conseillée : **Lucide** (licence ISC), cohérente avec ce style.
- Pas d'emoji dans l'interface.

## Composants

### Barre de titre (personnalisée)
- Fenêtre Tauri sans décoration système (`"decorations": false`) ; la barre est dessinée par l'application avec `data-tauri-drag-region`.
- Gauche : icône + mot « allotrope » (hauteur ~30 px). Droite : bouton Paramètres (carré, bordure), puis Réduire / Agrandir / Fermer (fond transparent, `--texte-secondaire`).
- Bordure inférieure 2 px `--ligne`.

### Barre latérale (220–260 px)
- Sur-titres en majuscules, puis éléments de navigation de 46 px de haut.
- Élément actif : fond `--cobalt`, bordure 2 px `--givre`, texte blanc 800.
- Élément de famille : petite pastille carrée 12 px de la couleur de la famille, bordure 2 px `--givre`.
- En bas : encart « 100 % local » (bouclier `--menthe`).

### Zone de dépôt
- Bordure 3 px en pointillés `--azur`, rayon 32 px, fond `--surface`.
- Icône aux trois formes au centre, titre 26 px 900, phrase d'aide, bouton « Parcourir les fichiers », puis pastilles des formats par famille.
- Au survol pendant un glisser-déposer : bordure pleine `--givre` et fond `--surface-active`.

### Carte de fichier (file d'attente)
- Badge 48 × 48 px, rayon 14 px, couleur de la famille, bordure 2 px `--givre`, extension en 12 px 900.
- Nom (15 px 800) + métadonnées (13 px), flèche →, sélecteur du format de sortie (bouton 44 px, bordure 2 px), état à droite.
- En cours : barre de progression de 12 px, rayon complet, fond `--ligne`, remplissage couleur de la famille.
- Sélectionnée : bordure 3 px `--givre`, fond `--surface-active`.

### Boutons
- **Principal (un seul par écran)** : fond `--braise`, texte `--abysse` 900, bordure 3 px `--givre`, hauteur 54 px, rayon 999 px.
- **Marque** : fond `--cobalt`, texte blanc 800, bordure 3 px `--givre`, hauteur 52 px, rayon 999 px.
- **Secondaire** : fond transparent, bordure 2 px `--ligne-forte`, texte `--givre` 800, hauteur 44 px.
- **Icône seule** : 44 × 44 px, rayon 14 px, bordure 2 px `--ligne`, avec `aria-label`.

### Contrôle segmenté (choix du format)
- Conteneur fond `--abysse`, bordure 2 px `--ligne`, rayon 16 px, marge interne 5 px.
- Segment actif : fond `--cobalt`, bordure 2 px `--givre`.

### Panneau d'options
- Carte de 280–340 px à droite de la file d'attente ; curseurs et cases à cocher avec `accent-color: var(--cobalt)`.

## Logo

- `assets/allotrope-logo.svg` : logo complet avec double contour (blanc + liseré noir), fond transparent — pour tout fond.
- `assets/allotrope-logo-complet.svg` : logo sur fond Abysse (écran « À propos », écran de démarrage).
- `assets/allotrope-icon-1024.png` : source des icônes de l'application → `npm run tauri icon assets/allotrope-icon-1024.png`.
- Ne jamais déformer, recolorer ou recomposer le logo.

## Mouvement

- Transitions courtes (150–200 ms, `ease-out`) sur les survols et les changements d'état.
- Respecter `prefers-reduced-motion`.
