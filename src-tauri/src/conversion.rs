//! Conversion d'images entre de nombreux formats, entièrement en local.
//! Les fonctions `lire`, `encoder` et `enregistrer` sont aussi utilisées par le module `compression`.

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Rgb, RgbImage, RgbaImage};
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
    Bmp,
    Tiff,
    Gif,
    Ico,
    Tga,
    Qoi,
    /// Image vectorielle, obtenue par vectorisation (crate `vtracer`).
    Svg,
}

/// Tous les formats de sortie, dans l'ordre où ils sont proposés.
const FORMATS_SORTIE: [FormatImage; 10] = [
    FormatImage::Png,
    FormatImage::Jpeg,
    FormatImage::Webp,
    FormatImage::Svg,
    FormatImage::Tiff,
    FormatImage::Bmp,
    FormatImage::Gif,
    FormatImage::Ico,
    FormatImage::Tga,
    FormatImage::Qoi,
];

impl FormatImage {
    pub fn extension(self) -> &'static str {
        match self {
            FormatImage::Jpeg => "jpg",
            FormatImage::Png => "png",
            FormatImage::Webp => "webp",
            FormatImage::Bmp => "bmp",
            FormatImage::Tiff => "tiff",
            FormatImage::Gif => "gif",
            FormatImage::Ico => "ico",
            FormatImage::Tga => "tga",
            FormatImage::Qoi => "qoi",
            FormatImage::Svg => "svg",
        }
    }

    /// Format correspondant à une extension (en minuscules), s'il fait partie des formats produits.
    pub fn depuis_extension(extension: &str) -> Option<Self> {
        match extension {
            "jpg" | "jpeg" | "jfif" | "jpe" => Some(FormatImage::Jpeg),
            "png" | "apng" => Some(FormatImage::Png),
            "webp" => Some(FormatImage::Webp),
            "bmp" | "dib" => Some(FormatImage::Bmp),
            "tif" | "tiff" => Some(FormatImage::Tiff),
            "gif" => Some(FormatImage::Gif),
            "ico" => Some(FormatImage::Ico),
            "tga" => Some(FormatImage::Tga),
            "qoi" => Some(FormatImage::Qoi),
            "svg" | "svgz" => Some(FormatImage::Svg),
            _ => None,
        }
    }

    /// Format équivalent de la crate `image`, pour les encodeurs génériques.
    /// Le SVG n'en a pas : il passe par la vectorisation (`vectoriser`).
    fn format_image(self) -> ImageFormat {
        match self {
            FormatImage::Svg => unreachable!("le SVG est produit par vectoriser()"),
            FormatImage::Jpeg => ImageFormat::Jpeg,
            FormatImage::Png => ImageFormat::Png,
            FormatImage::Webp => ImageFormat::WebP,
            FormatImage::Bmp => ImageFormat::Bmp,
            FormatImage::Tiff => ImageFormat::Tiff,
            FormatImage::Gif => ImageFormat::Gif,
            FormatImage::Ico => ImageFormat::Ico,
            FormatImage::Tga => ImageFormat::Tga,
            FormatImage::Qoi => ImageFormat::Qoi,
        }
    }
}

/// Extensions d'images que l'on sait lire (selon les fonctionnalités activées dans Cargo.toml).
pub const EXTENSIONS_LISIBLES: &[&str] = &[
    "jpg", "jpeg", "jfif", "jpe", "png", "apng", "webp", "bmp", "dib", "tif", "tiff", "gif", "ico",
    "tga", "qoi", "pbm", "pgm", "ppm", "pnm", "pam", "hdr", "exr", "dds", "ff", "svg", "svgz", "jxl",
];

/// Formats de sortie proposés pour une extension, le format par défaut en premier.
/// Liste vide si le fichier ne peut pas (encore) être converti.
pub fn formats_sortie(extension: &str) -> Vec<FormatImage> {
    let extension = extension.to_lowercase();
    if !EXTENSIONS_LISIBLES.contains(&extension.as_str()) {
        return Vec::new();
    }
    let source = FormatImage::depuis_extension(&extension);
    FORMATS_SORTIE.into_iter().filter(|f| Some(*f) != source).collect()
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
    let image = lire(&originaux, &extension_de(source))?;
    let octets = encoder(&image, format, qualite)?;
    let chemin = enregistrer(dossier, &nom_sans_extension(source), format.extension(), &octets)?;
    Ok(ResultatConversion { chemin, taille: octets.len() as u64 })
}

/// Miniature d'une image pour l'aperçu avant / après, au format « data:image/… » (affichable
/// directement par l'interface, sans lui ouvrir l'accès aux fichiers du disque).
#[tauri::command]
pub async fn apercu_image(chemin: PathBuf, cote_max: u32) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let octets = std::fs::read(&chemin).map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
        let image = lire(&octets, &extension_de(&chemin))?;
        Ok(miniature_data_url(&image, cote_max.clamp(64, 2048)))
    })
    .await
    .map_err(|e| format!("Aperçu interrompu : {e}"))?
}

/// Réduit l'image (sans jamais l'agrandir) puis l'encode : PNG si elle est transparente, JPEG sinon.
pub fn miniature_data_url(image: &DynamicImage, cote_max: u32) -> String {
    use base64::Engine;
    let reduite = if image.width() > cote_max || image.height() > cote_max {
        image.resize(cote_max, cote_max, FilterType::Triangle)
    } else {
        image.clone()
    };
    let (format, type_mime) = if reduite.color().has_alpha() {
        (FormatImage::Png, "image/png")
    } else {
        (FormatImage::Jpeg, "image/jpeg")
    };
    let octets = encoder(&en_8_bits(&reduite), format, 88).unwrap_or_default();
    format!("data:{type_mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(octets))
}

/// Nom du fichier sans son extension (« photo » pour « photo.png »).
pub fn nom_sans_extension(chemin: &Path) -> String {
    chemin
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".into())
}

/// Extension du fichier en minuscules, sans le point (chaîne vide s'il n'en a pas).
pub fn extension_de(chemin: &Path) -> String {
    chemin
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Décode une image en tenant compte de l'orientation EXIF (photos de téléphone).
/// `extension` aide à reconnaître les formats sans signature (TGA) et ceux qui ont leur propre lecteur.
pub fn lire(octets: &[u8], extension: &str) -> Result<DynamicImage, String> {
    match extension {
        "svg" | "svgz" => return lire_svg(octets),
        "jxl" => {
            let decodeur = jxl_oxide::integration::JxlDecoder::new(Cursor::new(octets))
                .map_err(|e| format!("Image JPEG XL illisible : {e}"))?;
            return DynamicImage::from_decoder(decodeur)
                .map_err(|e| format!("Image JPEG XL illisible : {e}"));
        }
        _ => {}
    }

    let mut lecteur = ImageReader::new(Cursor::new(octets))
        .with_guessed_format()
        .map_err(|e| format!("Impossible de lire le fichier : {e}"))?;
    // Certains formats (TGA) n'ont pas de signature : on se fie alors à l'extension
    if lecteur.format().is_none() {
        if let Some(format) = ImageFormat::from_extension(extension) {
            lecteur.set_format(format);
        }
    }
    let mut decodeur = lecteur
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

/// Côté le plus long (en pixels) en dessous duquel une image SVG est agrandie au rendu.
const TAILLE_MIN_SVG: f32 = 1024.0;

/// Rend une image vectorielle SVG en image matricielle (crate `resvg`).
/// Les petits SVG (icônes) sont agrandis pour obtenir une image exploitable.
fn lire_svg(octets: &[u8]) -> Result<DynamicImage, String> {
    use resvg::{tiny_skia, usvg};

    let mut options = usvg::Options::default();
    // Polices installées sur l'ordinateur, pour les SVG qui contiennent du texte
    options.fontdb_mut().load_system_fonts();
    let arbre = usvg::Tree::from_data(octets, &options).map_err(|e| format!("SVG illisible : {e}"))?;

    let taille = arbre.size();
    let echelle = (TAILLE_MIN_SVG / taille.width().max(taille.height())).max(1.0);
    let largeur = (taille.width() * echelle).round().max(1.0) as u32;
    let hauteur = (taille.height() * echelle).round().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(largeur, hauteur).ok_or("SVG de dimensions invalides")?;
    resvg::render(&arbre, tiny_skia::Transform::from_scale(echelle, echelle), &mut pixmap.as_mut());

    // tiny-skia stocke des couleurs « prémultipliées » par la transparence : on les remet à l'endroit
    let pixels: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    let rgba = RgbaImage::from_raw(largeur, hauteur, pixels).ok_or("Rendu SVG incomplet")?;
    Ok(DynamicImage::ImageRgba8(rgba))
}

/// Version 8 bits par canal (RGB, ou RGBA s'il y a de la transparence) : seule forme acceptée par
/// la plupart des encodeurs, et nécessaire pour les images HDR / EXR (32 bits flottants).
fn en_8_bits(image: &DynamicImage) -> DynamicImage {
    if image.color().has_alpha() {
        DynamicImage::ImageRgba8(image.to_rgba8())
    } else {
        DynamicImage::ImageRgb8(image.to_rgb8())
    }
}

/// Vrai pour les images en nombres flottants (HDR, EXR), que le PNG et le TIFF ne savent pas écrire.
fn est_flottante(image: &DynamicImage) -> bool {
    matches!(image, DynamicImage::ImageRgb32F(_) | DynamicImage::ImageRgba32F(_))
}

/// Encode une image en mémoire. `qualite` (1 à 100) s'applique au JPEG et au WebP.
pub fn encoder(image: &DynamicImage, format: FormatImage, qualite: u8) -> Result<Vec<u8>, String> {
    let qualite = qualite.clamp(1, 100);
    let mut octets = Vec::new();
    let mut curseur = Cursor::new(&mut octets);
    let resultat = match format {
        FormatImage::Jpeg => {
            // Le JPEG ne gère pas la transparence
            let rgb = DynamicImage::ImageRgb8(aplatir_sur_blanc(image));
            rgb.write_with_encoder(JpegEncoder::new_with_quality(&mut curseur, qualite))
        }
        // PNG et TIFF conservent les images 16 bits ; seules les images flottantes sont ramenées à 8 bits
        FormatImage::Png | FormatImage::Tiff if !est_flottante(image) => {
            image.write_to(&mut curseur, format.format_image())
        }
        FormatImage::Ico => {
            // Une icône Windows fait au plus 256 × 256 pixels
            let icone = if image.width() > 256 || image.height() > 256 {
                image.resize(256, 256, FilterType::Lanczos3)
            } else {
                image.clone()
            };
            DynamicImage::ImageRgba8(icone.to_rgba8()).write_to(&mut curseur, ImageFormat::Ico)
        }
        FormatImage::Webp => {
            // libwebp (crate `webp`) : compression avec perte, bien plus légère que le WebP sans perte
            let memoire = if image.color().has_alpha() {
                let rgba = image.to_rgba8();
                webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height()).encode(qualite as f32)
            } else {
                let rgb = image.to_rgb8();
                webp::Encoder::from_rgb(&rgb, rgb.width(), rgb.height()).encode(qualite as f32)
            };
            curseur.get_mut().extend_from_slice(&memoire);
            Ok(())
        }
        FormatImage::Svg => {
            curseur.get_mut().extend_from_slice(vectoriser(image)?.as_bytes());
            Ok(())
        }
        // BMP, GIF, TGA, QOI (et PNG / TIFF flottants) : pixels 8 bits
        _ => en_8_bits(image).write_to(&mut curseur, format.format_image()),
    };
    resultat.map_err(|e| format!("Échec de l'encodage : {e}"))?;
    Ok(octets)
}

/// Côté le plus long (en pixels) de l'image analysée pour la vectorisation : au-delà, le calcul
/// devient très long et le SVG énorme, sans gain visible.
const TAILLE_MAX_VECTORISATION: u32 = 2000;

/// Vectorise une image (tracé des contours de chaque zone de couleur) et renvoie le texte SVG.
/// Excellent sur les logos, icônes et dessins en aplats ; une photo devient une sorte d'affiche
/// aux couleurs simplifiées.
fn vectoriser(image: &DynamicImage) -> Result<String, String> {
    let (largeur, hauteur) = (image.width(), image.height());
    let analysee = if largeur.max(hauteur) > TAILLE_MAX_VECTORISATION {
        image.resize(TAILLE_MAX_VECTORISATION, TAILLE_MAX_VECTORISATION, FilterType::Lanczos3)
    } else {
        image.clone()
    };
    let rgba = analysee.to_rgba8();
    let couleurs = vtracer::ColorImage {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        pixels: rgba.into_raw(),
    };
    // Réglages par défaut de vtracer : couleur, courbes lissées (splines), zones empilées
    let svg = vtracer::convert(couleurs, vtracer::Config::default())
        .map_err(|e| format!("Échec de la vectorisation : {e}"))?;
    let (l, h) = (svg.width, svg.height);
    let texte = svg.to_string();
    // Le dessin garde les dimensions de l'image d'origine, même s'il a été calculé en plus petit
    Ok(texte.replacen(
        &format!(r#"width="{l}" height="{h}">"#),
        &format!(r#"width="{largeur}" height="{hauteur}" viewBox="0 0 {l} {h}">"#),
        1,
    ))
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
pub(crate) fn creer_fichier_libre(dossier: &Path, nom: &str, extension: &str) -> std::io::Result<(PathBuf, File)> {
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
        // Tous les formats de sortie sauf celui d'origine, PNG en premier sauf pour un PNG
        assert_eq!(formats_sortie("PNG")[0], FormatImage::Jpeg);
        assert!(!formats_sortie("png").contains(&FormatImage::Png));
        assert_eq!(formats_sortie("jpeg")[0], FormatImage::Png);
        assert_eq!(formats_sortie("bmp").len(), FORMATS_SORTIE.len() - 1);
        // Le SVG est déjà vectoriel : on ne le propose pas en sortie d'un SVG
        assert!(!formats_sortie("svg").contains(&FormatImage::Svg));
        assert!(formats_sortie("png").contains(&FormatImage::Svg));
        // Formats lus mais pas produits : tous les formats de sortie sont proposés
        assert_eq!(formats_sortie("exr").len(), FORMATS_SORTIE.len());
        assert!(formats_sortie("heic").is_empty());
        assert!(formats_sortie("mp4").is_empty());
    }

    #[test]
    fn convertit_vers_chaque_format() {
        let dossier = dossier_test("formats");
        let source = png_source(&dossier);
        let sortie = dossier.join("sortie");

        for format in FORMATS_SORTIE {
            let res = convertir(&source, &sortie, format, 85)
                .unwrap_or_else(|e| panic!("{format:?} : {e}"));
            assert_eq!(res.chemin.extension().unwrap(), format.extension());
            assert!(res.taille > 0);
            // Relecture du fichier produit (par son extension, nécessaire pour le TGA).
            // Le SVG relu est agrandi au rendu : seules ses proportions comptent.
            let relue = lire(&std::fs::read(&res.chemin).unwrap(), format.extension()).unwrap();
            let attendu = if format == FormatImage::Svg { (1024, 512) } else { (4, 2) };
            assert_eq!((relue.width(), relue.height()), attendu, "{format:?}");
        }
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn rend_un_svg_en_l_agrandissant() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
            <rect width="100" height="50" fill="#1F5BFF"/></svg>"##;
        let image = lire(svg, "svg").unwrap();
        assert_eq!((image.width(), image.height()), (1024, 512));
        let pixel = image.to_rgba8().get_pixel(10, 10).0;
        assert_eq!(pixel, [0x1F, 0x5B, 0xFF, 255]);
    }

    #[test]
    fn convertit_une_image_flottante() {
        // Une image 32 bits flottants (comme un HDR) doit pouvoir devenir PNG ou TIFF
        let dossier = dossier_test("flottante");
        let image = DynamicImage::ImageRgb32F(image::Rgb32FImage::from_pixel(3, 3, Rgb([0.5, 0.2, 1.0])));
        for format in [FormatImage::Png, FormatImage::Tiff, FormatImage::Jpeg] {
            let octets = encoder(&image, format, 90).unwrap_or_else(|e| panic!("{format:?} : {e}"));
            assert!(!octets.is_empty());
        }
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn vectorise_un_logo_en_svg() {
        // Disque bleu sur fond blanc : le SVG doit contenir un tracé bleu et garder la taille d'origine
        let mut img = RgbaImage::from_pixel(64, 64, Rgba([255, 255, 255, 255]));
        for (x, y, p) in img.enumerate_pixels_mut() {
            let (dx, dy) = (x as i32 - 32, y as i32 - 32);
            if dx * dx + dy * dy < 400 {
                *p = Rgba([0x1F, 0x5B, 0xFF, 255]);
            }
        }
        let octets = encoder(&DynamicImage::ImageRgba8(img), FormatImage::Svg, 85).unwrap();
        let svg = String::from_utf8(octets).unwrap();
        assert!(svg.contains(r#"width="64" height="64""#), "{svg}");
        assert!(svg.to_uppercase().contains("#1F5BFF"), "{svg}");
        // Le résultat est un SVG valide, relisible par resvg
        lire(svg.as_bytes(), "svg").unwrap();
    }

    #[test]
    fn vectorise_une_grande_image_en_gardant_ses_dimensions() {
        let grande = DynamicImage::ImageRgba8(RgbaImage::from_pixel(3000, 1500, Rgba([200, 30, 30, 255])));
        let svg = String::from_utf8(encoder(&grande, FormatImage::Svg, 85).unwrap()).unwrap();
        assert!(svg.contains(r#"width="3000" height="1500" viewBox="0 0 2000 1000""#), "{svg}");
    }

    #[test]
    fn produit_une_miniature_reduite() {
        let photo = DynamicImage::ImageRgb8(RgbImage::from_pixel(1200, 600, Rgb([10, 120, 200])));
        let url = miniature_data_url(&photo, 300);
        assert!(url.starts_with("data:image/jpeg;base64,"));
        use base64::Engine;
        let octets = base64::engine::general_purpose::STANDARD
            .decode(url.trim_start_matches("data:image/jpeg;base64,"))
            .unwrap();
        let relue = lire(&octets, "jpg").unwrap();
        assert_eq!((relue.width(), relue.height()), (300, 150));
        // Une image transparente reste en PNG, et une petite image n'est pas agrandie
        let icone = DynamicImage::ImageRgba8(RgbaImage::new(20, 10));
        assert!(miniature_data_url(&icone, 300).starts_with("data:image/png;base64,"));
    }

    #[test]
    fn reduit_une_icone_a_256_pixels() {
        let grande = DynamicImage::ImageRgba8(RgbaImage::new(600, 300));
        let octets = encoder(&grande, FormatImage::Ico, 85).unwrap();
        let relue = lire(&octets, "ico").unwrap();
        assert_eq!((relue.width(), relue.height()), (256, 128));
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
