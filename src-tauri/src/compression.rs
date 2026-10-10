//! Compression d'images : réduit le poids en gardant le format d'origine (JPEG, PNG, WebP).
//! Deux façons de régler : un taux de compression, ou une taille de fichier cible.

use crate::conversion::{encoder, enregistrer, extension_de, lire, nom_sans_extension, FormatImage};
use image::imageops::FilterType;
use image::DynamicImage;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Format compressible correspondant à une extension : seuls JPEG, PNG et WebP ont un sens ici.
fn format_compressible(extension: &str) -> Option<FormatImage> {
    FormatImage::depuis_extension(&extension.to_lowercase())
        .filter(|f| matches!(f, FormatImage::Jpeg | FormatImage::Png | FormatImage::Webp))
}

/// Vrai si un fichier de cette extension peut être compressé.
pub fn compressible(extension: &str) -> bool {
    format_compressible(extension).is_some()
}

/// Résultat renvoyé à l'interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatCompression {
    /// `None` si l'image était déjà optimisée (ou déjà sous la taille cible) : aucun fichier n'est créé.
    pub chemin: Option<PathBuf>,
    pub taille: u64,
}

/// Commande appelée depuis l'interface :
/// `invoke("compresser_image", { source, dossier, taux, tailleCible, tailleMax })`.
#[tauri::command]
pub async fn compresser_image(
    source: PathBuf,
    dossier: PathBuf,
    taux: u8,
    taille_cible: Option<u64>,
    taille_max: Option<u32>,
) -> Result<ResultatCompression, String> {
    tauri::async_runtime::spawn_blocking(move || {
        compresser(&source, &dossier, taux, taille_cible, taille_max)
    })
    .await
    .map_err(|e| format!("La compression a été interrompue : {e}"))?
}

/// Compresse `source` dans `dossier` sous le nom « nom (compressé).ext ».
/// - `taux` (0 à 100) : force de la compression, utilisé quand il n'y a pas de taille cible ;
/// - `taille_cible` : poids maximal visé, en octets ;
/// - `taille_max` : limite le plus grand côté de l'image, en pixels.
pub fn compresser(
    source: &Path,
    dossier: &Path,
    taux: u8,
    taille_cible: Option<u64>,
    taille_max: Option<u32>,
) -> Result<ResultatCompression, String> {
    let extension = extension_de(source);
    let format = format_compressible(&extension).ok_or("Ce format ne peut pas encore être compressé")?;

    let originaux = std::fs::read(source).map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    let taille_originale = originaux.len() as u64;
    let mut image = lire(&originaux, &extension)?;

    let redimensionnee = match taille_max {
        Some(max) if image.width() > max || image.height() > max => {
            // `resize` conserve les proportions ; Lanczos3 donne le rendu le plus net
            image = image.resize(max, max, FilterType::Lanczos3);
            true
        }
        _ => false,
    };
    // Sans redimensionnement, le PNG d'origine peut être optimisé tel quel (aucune perte)
    let png_d_origine = (!redimensionnee).then_some(originaux.as_slice());

    let octets = match taille_cible {
        Some(cible) => {
            if !redimensionnee && taille_originale <= cible {
                return Ok(ResultatCompression { chemin: None, taille: taille_originale });
            }
            compresser_sous(image, format, cible, png_d_origine)?
        }
        None => {
            let octets = compresser_au_taux(&image, format, taux, png_d_origine)?;
            // Si l'on ne gagne rien, on ne crée pas de fichier plus lourd que l'original
            if !redimensionnee && octets.len() as u64 >= taille_originale {
                return Ok(ResultatCompression { chemin: None, taille: taille_originale });
            }
            octets
        }
    };

    let nom = format!("{} (compressé)", nom_sans_extension(source));
    let chemin = enregistrer(dossier, &nom, format.extension(), &octets)?;
    Ok(ResultatCompression { chemin: Some(chemin), taille: octets.len() as u64 })
}

/// Qualité JPEG / WebP correspondant à un taux de compression (taux 0 → qualité 95, taux 100 → 5).
pub(crate) fn qualite_pour(taux: u8) -> u8 {
    (100 - taux.min(100)).clamp(5, 95)
}

/// Optimise un PNG sans perte avec oxipng. `effort` va de 0 (rapide) à 6 (le plus poussé).
fn optimiser_png(png: &[u8], effort: u8) -> Result<Vec<u8>, String> {
    let mut options = oxipng::Options::from_preset(effort.min(6));
    // Retire les métadonnées superflues, en gardant celles qui affectent le rendu (profil couleur)
    options.strip = oxipng::StripChunks::Safe;
    oxipng::optimize_from_memory(png, &options).map_err(|e| format!("Échec de l'optimisation PNG : {e}"))
}

fn compresser_au_taux(
    image: &DynamicImage,
    format: FormatImage,
    taux: u8,
    png_d_origine: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    match format {
        // Le PNG reste sans perte : le taux règle l'effort d'optimisation
        FormatImage::Png => {
            let effort = (taux.min(100) as u32 * 6 / 100).max(1) as u8;
            match png_d_origine {
                Some(png) => optimiser_png(png, effort),
                None => optimiser_png(&encoder(image, FormatImage::Png, 100)?, effort),
            }
        }
        _ => encoder(image, format, qualite_pour(taux)),
    }
}

/// Cherche le fichier le plus fidèle qui tienne dans `cible` octets.
/// On baisse d'abord la qualité ; si cela ne suffit pas, on réduit les dimensions par paliers de 20 %.
fn compresser_sous(
    mut image: DynamicImage,
    format: FormatImage,
    cible: u64,
    png_d_origine: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut plus_petit = u64::MAX;
    for palier in 0..12 {
        let essai = match format {
            FormatImage::Png => {
                let png = match (palier, png_d_origine) {
                    (0, Some(png)) => optimiser_png(png, 4)?,
                    _ => optimiser_png(&encoder(&image, FormatImage::Png, 100)?, 2)?,
                };
                plus_petit = plus_petit.min(png.len() as u64);
                (png.len() as u64 <= cible).then_some(png)
            }
            _ => {
                let (octets, minimum) = meilleure_qualite_sous(&image, format, cible)?;
                plus_petit = plus_petit.min(minimum);
                octets
            }
        };
        if let Some(octets) = essai {
            return Ok(octets);
        }

        let (largeur, hauteur) = (image.width() * 4 / 5, image.height() * 4 / 5);
        if largeur < 16 || hauteur < 16 {
            break;
        }
        image = image.resize(largeur, hauteur, FilterType::Lanczos3);
    }
    Err(format!(
        "Impossible d'atteindre {} : le plus petit résultat obtenu pèse {}",
        taille_lisible(cible),
        taille_lisible(plus_petit)
    ))
}

/// Recherche par dichotomie la qualité la plus haute dont le résultat tient dans `cible`.
/// Renvoie aussi le poids obtenu à la qualité minimale (pour le message d'erreur).
fn meilleure_qualite_sous(
    image: &DynamicImage,
    format: FormatImage,
    cible: u64,
) -> Result<(Option<Vec<u8>>, u64), String> {
    let (mut bas, mut haut) = (5u8, 95u8);
    let mut meilleur = None;
    let mut minimum = u64::MAX;
    while bas <= haut {
        let milieu = (bas + haut) / 2;
        let octets = encoder(image, format, milieu)?;
        minimum = minimum.min(octets.len() as u64);
        if octets.len() as u64 <= cible {
            meilleur = Some(octets);
            bas = milieu + 1;
        } else if milieu == 5 {
            break;
        } else {
            haut = milieu - 1;
        }
    }
    Ok((meilleur, minimum))
}

/// « 512 Ko », « 1,5 Mo » : pour les messages d'erreur.
fn taille_lisible(octets: u64) -> String {
    let (valeur, unite) = if octets >= 1024 * 1024 {
        (octets as f64 / (1024.0 * 1024.0), "Mo")
    } else {
        (octets as f64 / 1024.0, "Ko")
    };
    format!("{valeur:.1} {unite}").replace('.', ",").replace(",0 ", " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::tests::dossier_test;
    use image::{Rgb, RgbImage};

    /// Image avec un motif détaillé (difficile à compresser).
    fn motif(largeur: u32, hauteur: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(largeur, hauteur, |x, y| {
            Rgb([(x * 7 + y * 3) as u8, (x ^ y) as u8, (x * y) as u8])
        }))
    }

    fn ecrire(dossier: &Path, nom: &str, image: &DynamicImage, format: FormatImage, qualite: u8) -> PathBuf {
        let chemin = dossier.join(nom);
        std::fs::write(&chemin, encoder(image, format, qualite).unwrap()).unwrap();
        chemin
    }

    #[test]
    fn reconnait_les_formats_compressibles() {
        assert!(compressible("JPG"));
        assert!(compressible("png"));
        assert!(compressible("webp"));
        assert!(!compressible("bmp"));
        assert!(!compressible("mp4"));
    }

    #[test]
    fn convertit_le_taux_en_qualite() {
        assert_eq!(qualite_pour(0), 95);
        assert_eq!(qualite_pour(40), 60);
        assert_eq!(qualite_pour(100), 5);
    }

    #[test]
    fn compresse_un_jpeg_au_taux() {
        let dossier = dossier_test("compression-jpeg");
        let source = ecrire(&dossier, "photo.jpg", &motif(256, 128), FormatImage::Jpeg, 100);
        let taille_source = std::fs::metadata(&source).unwrap().len();

        let res = compresser(&source, &dossier, 40, None, None).unwrap();
        let chemin = res.chemin.expect("un fichier compressé devrait être créé");
        assert_eq!(chemin.file_name().unwrap(), "photo (compressé).jpg");
        assert!(res.taille < taille_source, "{} devrait être < {taille_source}", res.taille);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn atteint_une_taille_cible() {
        let dossier = dossier_test("compression-cible");
        for (nom, format) in [("photo.jpg", FormatImage::Jpeg), ("photo.webp", FormatImage::Webp), ("photo.png", FormatImage::Png)] {
            let source = ecrire(&dossier, nom, &motif(400, 300), format, 100);
            let cible = std::fs::metadata(&source).unwrap().len() / 3;

            let res = compresser(&source, &dossier, 0, Some(cible), None).unwrap();
            assert!(res.chemin.is_some(), "{nom} : un fichier devrait être créé");
            assert!(res.taille <= cible, "{nom} : {} devrait être ≤ {cible}", res.taille);
        }
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn signale_une_taille_cible_impossible() {
        let dossier = dossier_test("compression-impossible");
        let source = ecrire(&dossier, "photo.jpg", &motif(200, 200), FormatImage::Jpeg, 100);
        let erreur = compresser(&source, &dossier, 0, Some(10), None).unwrap_err();
        assert!(erreur.starts_with("Impossible d'atteindre"), "{erreur}");
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn deja_sous_la_taille_cible() {
        let dossier = dossier_test("compression-deja-cible");
        let source = ecrire(&dossier, "petit.jpg", &motif(32, 32), FormatImage::Jpeg, 50);
        let res = compresser(&source, &dossier, 0, Some(10 * 1024 * 1024), None).unwrap();
        assert!(res.chemin.is_none());
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn redimensionne_en_gardant_les_proportions() {
        let dossier = dossier_test("compression-taille");
        let source = ecrire(&dossier, "grand.png", &motif(400, 200), FormatImage::Png, 100);

        let res = compresser(&source, &dossier, 50, None, Some(100)).unwrap();
        let relue = image::open(res.chemin.unwrap()).unwrap();
        assert_eq!((relue.width(), relue.height()), (100, 50));
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn ne_cree_pas_de_fichier_plus_lourd() {
        let dossier = dossier_test("compression-optimal");
        // JPEG déjà très compressé : le réencoder en qualité 85 ne le rendra pas plus léger
        let source = ecrire(&dossier, "leger.jpg", &motif(64, 64), FormatImage::Jpeg, 10);

        let res = compresser(&source, &dossier, 15, None, None).unwrap();
        assert!(res.chemin.is_none());
        assert_eq!(std::fs::read_dir(&dossier).unwrap().count(), 1, "aucun fichier ne doit être ajouté");
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
