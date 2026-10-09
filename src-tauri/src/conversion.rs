//! Conversion d'images raster (JPEG, PNG, WebP) avec la crate `image`, entièrement en local.

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, ErrorKind, Write};
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
    fn extension(self) -> &'static str {
        match self {
            FormatImage::Jpeg => "jpg",
            FormatImage::Png => "png",
            FormatImage::Webp => "webp",
        }
    }

    fn depuis_extension(extension: &str) -> Option<Self> {
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
    let image = lire(source)?;

    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
    let nom = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".into());
    let (sortie, fichier) = creer_fichier_libre(dossier, &nom, format.extension())
        .map_err(|e| format!("Impossible de créer le fichier de sortie : {e}"))?;

    if let Err(e) = ecrire(&image, fichier, format, qualite) {
        // Ne pas laisser de fichier à moitié écrit
        let _ = std::fs::remove_file(&sortie);
        return Err(format!("Échec de l'encodage : {e}"));
    }

    let taille = std::fs::metadata(&sortie).map(|m| m.len()).unwrap_or(0);
    Ok(ResultatConversion { chemin: sortie, taille })
}

/// Décode une image en tenant compte de l'orientation EXIF (photos de téléphone).
fn lire(source: &Path) -> Result<DynamicImage, String> {
    let mut decodeur = ImageReader::open(source)
        .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?
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

fn ecrire(image: &DynamicImage, fichier: File, format: FormatImage, qualite: u8) -> image::ImageResult<()> {
    let mut sortie = BufWriter::new(fichier);
    match format {
        FormatImage::Jpeg => {
            // Le JPEG ne gère pas la transparence
            let rgb = DynamicImage::ImageRgb8(aplatir_sur_blanc(image));
            rgb.write_with_encoder(JpegEncoder::new_with_quality(&mut sortie, qualite.clamp(1, 100)))?;
        }
        FormatImage::Png => image.write_to(&mut sortie, ImageFormat::Png)?,
        FormatImage::Webp => {
            // L'encodeur WebP n'accepte que des pixels 8 bits RGB ou RGBA
            let image8 = if image.color().has_alpha() {
                DynamicImage::ImageRgba8(image.to_rgba8())
            } else {
                DynamicImage::ImageRgb8(image.to_rgb8())
            };
            image8.write_to(&mut sortie, ImageFormat::WebP)?;
        }
    }
    sortie.flush()?;
    Ok(())
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
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// Dossier temporaire propre à chaque test.
    fn dossier_test(nom: &str) -> PathBuf {
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
