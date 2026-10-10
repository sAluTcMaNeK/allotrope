//! Export des pages d'un PDF en images (outil « PDF en images »).
//!
//! Les pages sont dessinées par pdf.js dans l'interface, puis envoyées une à une au cœur Rust
//! (en PNG, sans perte) qui les encode dans le format choisi et les enregistre :
//! `export_pages_debut` → `export_pages_ajouter` (une fois par page) → `export_pages_fin`.

use crate::conversion::{creer_fichier_libre, encoder, nom_sans_extension, FormatImage};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex};
use tauri::ipc::{InvokeBody, Request};

/// Où vont les images d'un export en cours.
enum Destination {
    Dossier(PathBuf),
    Zip { archive: zip::ZipWriter<File>, chemin: PathBuf },
}

struct Export {
    destination: Destination,
    nombre: u32,
}

/// Exports en cours, par numéro.
static EXPORTS: LazyLock<Mutex<HashMap<u32, Export>>> = LazyLock::new(Default::default);
static PROCHAIN: AtomicU32 = AtomicU32::new(1);

/// Résultat renvoyé à l'interface : dossier créé (ou archive ZIP) et nombre d'images.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatExport {
    pub chemin: PathBuf,
    pub nombre: u32,
}

/// Crée `dossier/nom`, ou `nom (2)`, `nom (3)`… si le nom est déjà pris.
pub(crate) fn creer_dossier_libre(dossier: &Path, nom: &str) -> Result<PathBuf, String> {
    for numero in 1.. {
        let chemin = if numero == 1 { dossier.join(nom) } else { dossier.join(format!("{nom} ({numero})")) };
        match std::fs::create_dir(&chemin) {
            Ok(()) => return Ok(chemin),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Impossible de créer le dossier : {e}")),
        }
    }
    unreachable!()
}

/// Commence un export : crée le dossier « nom (pages) » ou l'archive « nom (pages).zip ».
/// Renvoie le numéro de l'export, à rappeler pour chaque page.
#[tauri::command]
pub fn export_pages_debut(source: PathBuf, dossier: PathBuf, zip: bool) -> Result<u32, String> {
    std::fs::create_dir_all(&dossier).map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
    let base = format!("{} (pages)", nom_sans_extension(&source));
    let destination = if zip {
        let (chemin, fichier) =
            creer_fichier_libre(&dossier, &base, "zip").map_err(|e| format!("Impossible de créer l'archive : {e}"))?;
        Destination::Zip { archive: zip::ZipWriter::new(fichier), chemin }
    } else {
        Destination::Dossier(creer_dossier_libre(&dossier, &base)?)
    };
    let numero = PROCHAIN.fetch_add(1, Ordering::Relaxed);
    EXPORTS.lock().unwrap().insert(numero, Export { destination, nombre: 0 });
    Ok(numero)
}

/// Format demandé par l'interface (« png », « jpeg », « webp »).
fn format_page(nom: &str) -> Result<FormatImage, String> {
    match nom {
        "png" => Ok(FormatImage::Png),
        "jpeg" => Ok(FormatImage::Jpeg),
        "webp" => Ok(FormatImage::Webp),
        autre => Err(format!("Format d'image inattendu : {autre}")),
    }
}

/// Encode une page (reçue en PNG) dans le format voulu. Le PNG est gardé tel quel.
pub fn encoder_page(png: Vec<u8>, format: FormatImage, qualite: u8) -> Result<Vec<u8>, String> {
    if format == FormatImage::Png {
        return Ok(png);
    }
    let image = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .map_err(|e| format!("Image de la page illisible : {e}"))?;
    encoder(&image, format, qualite)
}

/// Ajoute une page à l'export, dans « page 001.jpg ».
pub fn ajouter_page(export: u32, page: u32, format: FormatImage, octets: &[u8]) -> Result<(), String> {
    let mut exports = EXPORTS.lock().unwrap();
    let export = exports.get_mut(&export).ok_or("Export inconnu ou déjà terminé")?;
    let nom = format!("page {page:03}.{}", format.extension());
    match &mut export.destination {
        Destination::Dossier(dossier) => {
            std::fs::write(dossier.join(&nom), octets).map_err(|e| format!("Échec de l'écriture de « {nom} » : {e}"))?;
        }
        Destination::Zip { archive, .. } => {
            // Images déjà compressées : rangées telles quelles dans l'archive
            let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            archive.start_file(nom.as_str(), options).map_err(|e| e.to_string())?;
            archive.write_all(octets).map_err(|e| e.to_string())?;
        }
    }
    export.nombre += 1;
    Ok(())
}

/// Reçoit une page dessinée par pdf.js. Le corps de la requête contient l'image PNG brute ;
/// les en-têtes donnent le numéro d'export, le numéro de page, le format et la qualité.
#[tauri::command]
pub async fn export_pages_ajouter(requete: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(png) = requete.body() else {
        return Err("Image de la page manquante".into());
    };
    let entete = |nom: &str| requete.headers().get(nom).and_then(|v| v.to_str().ok()).map(str::to_owned);
    let export: u32 = entete("export").and_then(|v| v.parse().ok()).ok_or("Numéro d'export manquant")?;
    let page: u32 = entete("page").and_then(|v| v.parse().ok()).ok_or("Numéro de page manquant")?;
    let format = format_page(&entete("format").unwrap_or_default())?;
    let qualite: u8 = entete("qualite").and_then(|v| v.parse().ok()).unwrap_or(85);

    // L'encodage (JPEG, WebP) se fait hors du fil principal
    let png = png.clone();
    let octets = tauri::async_runtime::spawn_blocking(move || encoder_page(png, format, qualite))
        .await
        .map_err(|e| format!("Encodage interrompu : {e}"))??;
    ajouter_page(export, page, format, &octets)
}

/// Termine l'export (ferme l'archive ZIP) et renvoie son emplacement.
#[tauri::command]
pub fn export_pages_fin(export: u32) -> Result<ResultatExport, String> {
    let export = EXPORTS.lock().unwrap().remove(&export).ok_or("Export inconnu ou déjà terminé")?;
    let chemin = match export.destination {
        Destination::Dossier(dossier) => dossier,
        Destination::Zip { archive, chemin } => {
            archive.finish().map_err(|e| format!("Échec de l'écriture de l'archive : {e}"))?;
            chemin
        }
    };
    Ok(ResultatExport { chemin, nombre: export.nombre })
}

/// Abandonne un export (erreur ou annulation) : supprime le dossier ou l'archive commencés.
#[tauri::command]
pub fn export_pages_annuler(export: u32) {
    if let Some(export) = EXPORTS.lock().unwrap().remove(&export) {
        match export.destination {
            Destination::Dossier(dossier) => {
                let _ = std::fs::remove_dir_all(dossier);
            }
            Destination::Zip { archive, chemin } => {
                drop(archive);
                let _ = std::fs::remove_file(chemin);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::tests::dossier_test;
    use image::{DynamicImage, Rgb, RgbImage};
    use std::io::Cursor;

    fn png_test() -> Vec<u8> {
        let mut png = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::from_pixel(40, 60, Rgb([20, 90, 200])))
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    #[test]
    fn exporte_dans_un_dossier_puis_dans_une_archive() {
        let dossier = dossier_test("export-pages");
        let source = dossier.join("rapport.pdf");

        // Dossier « rapport (pages) » avec deux pages en JPEG
        let export = export_pages_debut(source.clone(), dossier.clone(), false).unwrap();
        for page in [1, 2] {
            let jpeg = encoder_page(png_test(), FormatImage::Jpeg, 80).unwrap();
            ajouter_page(export, page, FormatImage::Jpeg, &jpeg).unwrap();
        }
        let resultat = export_pages_fin(export).unwrap();
        assert_eq!(resultat.nombre, 2);
        assert_eq!(resultat.chemin.file_name().unwrap(), "rapport (pages)");
        let relue = image::open(resultat.chemin.join("page 002.jpg")).unwrap();
        assert_eq!((relue.width(), relue.height()), (40, 60));

        // Archive « rapport (pages).zip » avec une page en WebP
        let export = export_pages_debut(source.clone(), dossier.clone(), true).unwrap();
        let webp = encoder_page(png_test(), FormatImage::Webp, 80).unwrap();
        ajouter_page(export, 7, FormatImage::Webp, &webp).unwrap();
        let resultat = export_pages_fin(export).unwrap();
        let archive = zip::ZipArchive::new(File::open(&resultat.chemin).unwrap()).unwrap();
        assert_eq!(archive.file_names().map(|n| n.unwrap().into_owned()).collect::<Vec<_>>(), vec!["page 007.webp"]);

        // Un export annulé ne laisse rien derrière lui
        let export = export_pages_debut(source, dossier.clone(), false).unwrap();
        export_pages_annuler(export);
        assert!(!dossier.join("rapport (pages) (2)").exists());
        assert!(export_pages_fin(export).is_err());
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
