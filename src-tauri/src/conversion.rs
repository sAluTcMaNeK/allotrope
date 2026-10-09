//! Conversion d'images raster (JPEG, PNG, WebP), entièrement en local.
//! Les fonctions `lire`, `encoder` et `enregistrer` sont aussi utilisées par le module `compression`.

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// Formats d'image que l'on sait produire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatImage {
    Jpeg,
    Png,
    Webp,
}

impl FormatImage {
    pub fn extension(self) -> &'static str {
        match self {
            FormatImage::Jpeg => "jpg",
            FormatImage::Png => "png",
            FormatImage::Webp => "webp",
        }
    }

    /// Format correspondant à une extension (en minuscules), s'il fait partie des formats produits.
    pub fn depuis_extension(extension: &str) -> Option<Self> {
        match extension {
            "jpg" | "jpeg" => Some(FormatImage::Jpeg),
            "png" => Some(FormatImage::Png),
            "webp" => Some(FormatImage::Webp),
            _ => None,
        }
    }
}

/// Extensions d'images que l'on sait lire (selon les fonctionnalités activées dans Cargo.toml).
const EXTENSIONS_LISIBLES: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff", "ico"];

/// Formats de sortie proposés pour une extension, le format par défaut en premier.
/// Liste vide si le fichier ne peut pas (encore) être converti.
pub fn formats_sortie(extension: &str) -> Vec<FormatImage> {
    let extension = extension.to_lowercase();
    if !EXTENSIONS_LISIBLES.contains(&extension.as_str()) {
        return Vec::new();
    }
    let source = FormatImage::depuis_extension(&extension);
    [FormatImage::Png, FormatImage::Jpeg, FormatImage::Webp]
        .into_iter()
        .filter(|f| Some(*f) != source)
        .collect()
}

/// Résultat renvoyé à l'interface après une conversion réussie.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatConversion {
    pub chemin: PathBuf,
    pub taille: u64,
}

/// Commande appelée depuis l'interface : `invoke("convertir_image", { source, dossier, format, qualite })`.
/// Le travail (lourd pour le processeur) est confié à un fil dédié avec `spawn_blocking`.
#[tauri::command]
pub async fn convertir_image(
    source: PathBuf,
    dossier: PathBuf,
    format: FormatImage,
    qualite: u8,
) -> Result<ResultatConversion, String> {
    tauri::async_runtime::spawn_blocking(move || convertir(&source, &dossier, format, qualite))
        .await
        .map_err(|e| format!("La conversion a été interrompue : {e}"))?
}

/// Convertit `source` vers `format` dans `dossier`, sans jamais écraser un fichier existant.
pub fn convertir(
    source: &Path,
    dossier: &Path,
    format: FormatImage,
    qualite: u8,
) -> Result<ResultatConversion, String> {
    let originaux = std::fs::read(source).map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    let image = lire(&originaux)?;
    let octets = encoder(&image, format, qualite)?;
    let chemin = enregistrer(dossier, &nom_sans_extension(source), format.extension(), &octets)?;
    Ok(ResultatConversion { chemin, taille: octets.len() as u64 })
}

/// Nom du fichier sans son extension (« photo » pour « photo.png »).
pub fn nom_sans_extension(chemin: &Path) -> String {
    chemin
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".into())
}

/// Décode une image en tenant compte de l'orientation EXIF (photos de téléphone).
pub fn lire(octets: &[u8]) -> Result<DynamicImage, String> {
    let mut decodeur = ImageReader::new(Cursor::new(octets))
        .with_guessed_format()
        .map_err(|e| format!("Impossible de lire le fichier : {e}"))?
        .into_decoder()
        .map_err(|e| format!("Format d'image non reconnu : {e}"))?;
    let orientation = decodeur.orientation().ok();
    let mut image =
        DynamicImage::from_decoder(decodeur).map_err(|e| format!("Image illisible : {e}"))?;
    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }
    Ok(image)
}

/// Encode une image en mémoire. `qualite` (1 à 100) s'applique au JPEG et au WebP.
pub fn encoder(image: &DynamicImage, format: FormatImage, qualite: u8) -> Result<Vec<u8>, String> {
    let qualite = qualite.clamp(1, 100);
    let mut octets = Vec::new();
    let resultat = match format {
        FormatImage::Jpeg => {
            // Le JPEG ne gère pas la transparence
            let rgb = DynamicImage::ImageRgb8(aplatir_sur_blanc(image));
            rgb.write_with_encoder(JpegEncoder::new_with_quality(&mut octets, qualite))
        }
        FormatImage::Png => image.write_to(&mut Cursor::new(&mut octets), ImageFormat::Png),
        FormatImage::Webp => {
            // libwebp (crate `webp`) : compression avec perte, bien plus légère que le WebP sans perte
            let memoire = if image.color().has_alpha() {
                let rgba = image.to_rgba8();
                webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height()).encode(qualite as f32)
            } else {
                let rgb = image.to_rgb8();
                webp::Encoder::from_rgb(&rgb, rgb.width(), rgb.height()).encode(qualite as f32)
            };
            octets.extend_from_slice(&memoire);
            Ok(())
        }
    };
    resultat.map_err(|e| format!("Échec de l'encodage : {e}"))?;
    Ok(octets)
}

/// Écrit `octets` dans `dossier/nom.extension` (ou `nom (2).extension`…) et renvoie le chemin choisi.
pub fn enregistrer(dossier: &Path, nom: &str, extension: &str, octets: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
    let (chemin, mut fichier) = creer_fichier_libre(dossier, nom, extension)
        .map_err(|e| format!("Impossible de créer le fichier de sortie : {e}"))?;
    if let Err(e) = fichier.write_all(octets) {
        // Ne pas laisser de fichier à moitié écrit
        drop(fichier);
        let _ = std::fs::remove_file(&chemin);
        return Err(format!("Échec de l'écriture : {e}"));
    }
    Ok(chemin)
}

/// Crée `nom.ext`, ou `nom (2).ext`, `nom (3).ext`… si le nom est déjà pris.
/// `create_new` réserve le nom de façon atomique, même si plusieurs conversions tournent en parallèle.
fn creer_fichier_libre(dossier: &Path, nom: &str, extension: &str) -> std::io::Result<(PathBuf, File)> {
    let mut numero = 1;
    loop {
        let chemin = if numero == 1 {
            dossier.join(format!("{nom}.{extension}"))
        } else {
            dossier.join(format!("{nom} ({numero}).{extension}"))
        };
        match File::options().write(true).create_new(true).open(&chemin) {
            Ok(fichier) => return Ok((chemin, fichier)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => numero += 1,
            Err(e) => return Err(e),
        }
    }
}

/// Remplace la transparence par du blanc (sinon les zones transparentes deviendraient noires en JPEG).
fn aplatir_sur_blanc(image: &DynamicImage) -> RgbImage {
    if !image.color().has_alpha() {
        return image.to_rgb8();
    }
    let rgba = image.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as u32;
        let melange = |c: u8| ((c as u32 * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        rgb.put_pixel(x, y, Rgb([melange(pixel[0]), melange(pixel[1]), melange(pixel[2])]));
    }
    rgb
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// Dossier temporaire propre à chaque test.
    pub fn dossier_test(nom: &str) -> PathBuf {
        let dossier = std::env::temp_dir().join(format!("allotrope-test-{nom}"));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).unwrap();
        dossier
    }

    /// Image PNG 4 × 2 : moitié rouge opaque, moitié transparente.
    fn png_source(dossier: &Path) -> PathBuf {
        let mut img = RgbaImage::new(4, 2);
        for (x, _, p) in img.enumerate_pixels_mut() {
            *p = if x < 2 { Rgba([255, 0, 0, 255]) } else { Rgba([0, 0, 0, 0]) };
        }
        let chemin = dossier.join("source.png");
        img.save(&chemin).unwrap();
        chemin
    }

    #[test]
    fn formats_proposes() {
        assert_eq!(formats_sortie("PNG"), vec![FormatImage::Jpeg, FormatImage::Webp]);
        assert_eq!(formats_sortie("jpeg"), vec![FormatImage::Png, FormatImage::Webp]);
        assert_eq!(formats_sortie("bmp").len(), 3);
        assert!(formats_sortie("heic").is_empty());
        assert!(formats_sortie("mp4").is_empty());
    }

    #[test]
    fn convertit_vers_chaque_format() {
        let dossier = dossier_test("formats");
        let source = png_source(&dossier);
        let sortie = dossier.join("sortie");

        for format in [FormatImage::Jpeg, FormatImage::Png, FormatImage::Webp] {
            let res = convertir(&source, &sortie, format, 85).unwrap();
            assert_eq!(res.chemin.extension().unwrap(), format.extension());
            assert!(res.taille > 0);
            let relue = image::open(&res.chemin).unwrap();
            assert_eq!((relue.width(), relue.height()), (4, 2));
        }
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn jpeg_remplace_la_transparence_par_du_blanc() {
        let dossier = dossier_test("blanc");
        let source = png_source(&dossier);
        let res = convertir(&source, &dossier, FormatImage::Jpeg, 100).unwrap();
        let pixel = image::open(&res.chemin).unwrap().to_rgb8().get_pixel(3, 0).0;
        assert!(pixel.iter().all(|&c| c > 240), "pixel attendu blanc, obtenu {pixel:?}");
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn n_ecrase_jamais_un_fichier_existant() {
        let dossier = dossier_test("nommage");
        let source = png_source(&dossier);
        let premier = convertir(&source, &dossier, FormatImage::Webp, 85).unwrap();
        let second = convertir(&source, &dossier, FormatImage::Webp, 85).unwrap();
        assert_eq!(premier.chemin.file_name().unwrap(), "source.webp");
        assert_eq!(second.chemin.file_name().unwrap(), "source (2).webp");
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
