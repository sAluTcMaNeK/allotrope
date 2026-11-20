//! Analyse des fichiers reçus par l'interface (glisser-déposer ou sélecteur de fichiers).

use crate::compression::compressible;
use crate::conversion::{formats_sortie, FormatImage, EXTENSIONS_LISIBLES};
use crate::media::{self, format_compression, formats_sortie_media, FormatMedia, EXTENSIONS_AUDIO, EXTENSIONS_VIDEO};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// Grande famille d'un fichier : détermine sa couleur dans l'interface et les conversions possibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Famille {
    Image,
    Video,
    Audio,
}

/// Format de sortie proposé : image ou audio / vidéo. `untagged` : envoyé tel quel (« png », « mp4 »…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum FormatSortie {
    Image(FormatImage),
    Media(FormatMedia),
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
    /// Largeur et hauteur en pixels (images lisibles et vidéos).
    pub dimensions: Option<(u32, u32)>,
    /// Durée en secondes (audio et vidéo).
    pub duree: Option<f64>,
    /// Formats de conversion proposés, le format par défaut en premier (vide si non convertible).
    pub formats_sortie: Vec<FormatSortie>,
    /// Vrai si le fichier peut être compressé.
    pub compressible: bool,
}

/// Déduit la famille d'un fichier à partir de son extension.
pub fn famille_depuis_extension(extension: &str) -> Option<Famille> {
    let extension = extension.to_lowercase();
    match extension.as_str() {
        // Le GIF est classé en vidéo (souvent animé, voir DESIGN.md)
        e if EXTENSIONS_VIDEO.contains(&e) => Some(Famille::Video),
        e if EXTENSIONS_AUDIO.contains(&e) => Some(Famille::Audio),
        // HEIC / AVIF : reconnus mais pas encore lisibles (libheif / libavif plus tard)
        "heic" | "heif" | "avif" => Some(Famille::Image),
        e if EXTENSIONS_LISIBLES.contains(&e) => Some(Famille::Image),
        _ => None,
    }
}

/// Analyse un chemin. Renvoie `None` pour un dossier ou un chemin illisible.
/// La durée et la définition des vidéos sont ajoutées ensuite par FFmpeg (`analyser_fichiers`).
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
    let famille = famille_depuis_extension(&extension);

    let (formats_sortie, compressible, dimensions) = match famille {
        // Version portable : pas de FFmpeg, donc aucun traitement audio ou vidéo possible
        Some(Famille::Video | Famille::Audio) if crate::PORTABLE => (Vec::new(), false, None),
        Some(Famille::Video | Famille::Audio) => (
            formats_sortie_media(&extension).into_iter().map(FormatSortie::Media).collect(),
            format_compression(&extension).is_some(),
            None,
        ),
        _ => {
            let formats: Vec<FormatSortie> = formats_sortie(&extension).into_iter().map(FormatSortie::Image).collect();
            // Ne lit que l'en-tête de l'image : rapide même pour une grosse photo
            let dimensions = if formats.is_empty() { None } else { image::image_dimensions(chemin).ok() };
            (formats, compressible(&extension), dimensions)
        }
    };

    Some(InfoFichier {
        chemin: chemin.to_path_buf(),
        nom,
        famille,
        compressible,
        extension,
        taille: meta.len(),
        dimensions,
        duree: None,
        formats_sortie,
    })
}

/// Commande appelée depuis l'interface : `invoke("analyser_fichiers", { chemins })`.
/// Elle est `async` pour s'exécuter hors du fil principal et ne jamais figer la fenêtre.
#[tauri::command]
pub async fn analyser_fichiers(app: AppHandle, chemins: Vec<PathBuf>) -> Vec<InfoFichier> {
    let mut infos: Vec<InfoFichier> = chemins.iter().filter_map(|c| analyser(c)).collect();
    // Audio et vidéo : durée et définition lues par FFmpeg (quelques centièmes de seconde par fichier)
    // (sauf dans la version portable, qui n'embarque pas FFmpeg)
    let medias = infos.iter_mut().filter(|i| !crate::PORTABLE && matches!(i.famille, Some(Famille::Video | Famille::Audio)));
    for info in medias {
        if let Ok(media) = media::infos(&app, &info.chemin).await {
            info.duree = media.duree;
            info.dimensions = media.largeur.zip(media.hauteur);
        }
    }
    infos
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
