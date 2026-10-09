//! Analyse des fichiers reçus par l'interface (glisser-déposer ou sélecteur de fichiers).

use crate::compression::compressible;
use crate::conversion::{formats_sortie, FormatImage, EXTENSIONS_LISIBLES};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Grande famille d'un fichier : détermine sa couleur dans l'interface et les conversions possibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Famille {
    Image,
    Video,
    Audio,
}

/// Informations sur un fichier, envoyées à l'interface.
/// `rename_all = "camelCase"` adapte les noms de champs aux conventions de TypeScript.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoFichier {
    pub chemin: PathBuf,
    pub nom: String,
    /// Extension en minuscules, sans le point (chaîne vide si le fichier n'en a pas).
    pub extension: String,
    /// Taille en octets.
    pub taille: u64,
    /// `None` si le format n'est pas reconnu (devient `null` côté interface).
    pub famille: Option<Famille>,
    /// Largeur et hauteur en pixels, pour les images lisibles.
    pub dimensions: Option<(u32, u32)>,
    /// Formats de conversion proposés, le format par défaut en premier (vide si non convertible).
    pub formats_sortie: Vec<FormatImage>,
    /// Vrai si le fichier peut être compressé.
    pub compressible: bool,
}

/// Déduit la famille d'un fichier à partir de son extension.
pub fn famille_depuis_extension(extension: &str) -> Option<Famille> {
    let extension = extension.to_lowercase();
    match extension.as_str() {
        // Le GIF est classé en vidéo (souvent animé, voir DESIGN.md), même s'il peut être converti en image
        "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" | "gif" => Some(Famille::Video),
        // HEIC / AVIF : reconnus mais pas encore lisibles (libheif / libavif plus tard)
        "heic" | "heif" | "avif" => Some(Famille::Image),
        e if EXTENSIONS_LISIBLES.contains(&e) => Some(Famille::Image),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" => Some(Famille::Audio),
        _ => None,
    }
}

/// Analyse un chemin. Renvoie `None` pour un dossier ou un chemin illisible.
fn analyser(chemin: &Path) -> Option<InfoFichier> {
    let meta = std::fs::metadata(chemin).ok()?;
    if !meta.is_file() {
        return None;
    }

    let extension = chemin
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let nom = chemin
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let formats_sortie = formats_sortie(&extension);
    // Ne lit que l'en-tête de l'image : rapide même pour une grosse photo
    let dimensions = if formats_sortie.is_empty() {
        None
    } else {
        image::image_dimensions(chemin).ok()
    };

    Some(InfoFichier {
        chemin: chemin.to_path_buf(),
        nom,
        famille: famille_depuis_extension(&extension),
        compressible: compressible(&extension),
        extension,
        taille: meta.len(),
        dimensions,
        formats_sortie,
    })
}

/// Commande appelée depuis l'interface : `invoke("analyser_fichiers", { chemins })`.
/// Elle est `async` pour s'exécuter hors du fil principal et ne jamais figer la fenêtre.
#[tauri::command]
pub async fn analyser_fichiers(chemins: Vec<PathBuf>) -> Vec<InfoFichier> {
    chemins.iter().filter_map(|c| analyser(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnait_les_familles() {
        assert_eq!(famille_depuis_extension("jpg"), Some(Famille::Image));
        assert_eq!(famille_depuis_extension("TGA"), Some(Famille::Image));
        assert_eq!(famille_depuis_extension("svg"), Some(Famille::Image));
        assert_eq!(famille_depuis_extension("gif"), Some(Famille::Video));
        assert_eq!(famille_depuis_extension("MOV"), Some(Famille::Video));
        assert_eq!(famille_depuis_extension("flac"), Some(Famille::Audio));
        // Les PDF ont leur propre page (Outils PDF) : pas de famille
        assert_eq!(famille_depuis_extension("pdf"), None);
        assert_eq!(famille_depuis_extension("exe"), None);
        assert_eq!(famille_depuis_extension(""), None);
    }

    #[test]
    fn analyse_un_vrai_fichier() {
        let dossier = std::env::temp_dir().join("allotrope-test-analyse");
        std::fs::create_dir_all(&dossier).unwrap();
        let fichier = dossier.join("Photo.PNG");
        std::fs::write(&fichier, [0u8; 42]).unwrap();

        let info = analyser(&fichier).expect("le fichier devrait être analysé");
        assert_eq!(info.nom, "Photo.PNG");
        assert_eq!(info.extension, "png");
        assert_eq!(info.taille, 42);
        assert_eq!(info.famille, Some(Famille::Image));

        // Un dossier est ignoré
        assert!(analyser(&dossier).is_none());

        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
