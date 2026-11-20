// Édition d'Allotrope : classique (installée) ou portable (un seul .exe, sans FFmpeg).
// La valeur vient du cœur Rust (option de compilation `portable`), qui la déclare avant le
// chargement de l'interface (voir src-tauri/src/lib.rs) : elle est donc connue dès le départ.

export const PORTABLE: boolean = window.__ALLOTROPE_PORTABLE__ === true;

/** Page de téléchargement de la dernière version (la version portable s'y met à jour à la main). */
export const PAGE_TELECHARGEMENT = "https://github.com/sAluTcMaNeK/allotrope/releases/latest";
