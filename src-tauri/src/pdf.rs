//! Outils PDF (crate `lopdf`) : fusionner, réorganiser, supprimer ou faire pivoter des pages,
//! et créer un PDF à partir d'images. Tout se fait en local.
//!
//! Toutes les opérations sur les pages passent par une seule fonction, `assembler` :
//! on lui donne des documents sources et la liste des pages à garder, dans l'ordre voulu.
//! Fusionner = toutes les pages de plusieurs documents ; supprimer = garder les autres ; etc.

use crate::conversion::{encoder, enregistrer, extension_de, lire, FormatImage};
use image::{ImageDecoder, ImageReader};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Une page du document à produire.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageChoisie {
    /// Indice du document source dans la liste `sources`.
    pub source: usize,
    /// Numéro de la page dans ce document (à partir de 1).
    pub page: u32,
    /// Rotation à ajouter, en degrés (multiple de 90).
    #[serde(default)]
    pub rotation: i64,
}

/// Résultat renvoyé à l'interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatPdf {
    pub chemin: PathBuf,
    pub taille: u64,
    pub pages: u32,
}

/// Informations sur un PDF, pour la liste des documents à fusionner.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoPdf {
    pub pages: u32,
}

// --- Commandes appelées depuis l'interface ---

/// Nombre de pages d'un PDF.
#[tauri::command]
pub async fn infos_pdf(chemin: PathBuf) -> Result<InfoPdf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let document = charger(&chemin)?;
        Ok(InfoPdf { pages: document.get_pages().len() as u32 })
    })
    .await
    .map_err(|e| format!("Lecture interrompue : {e}"))?
}

/// Contenu brut d'un PDF, pour afficher les miniatures des pages avec pdf.js dans l'interface.
/// `Response` transmet les octets tels quels, sans les convertir en texte.
#[tauri::command]
pub async fn lire_pdf(chemin: PathBuf) -> Result<tauri::ipc::Response, String> {
    if extension_de(&chemin) != "pdf" {
        return Err("Ce fichier n'est pas un PDF".into());
    }
    let octets = std::fs::read(&chemin).map_err(|e| format!("Impossible d'ouvrir le PDF : {e}"))?;
    Ok(tauri::ipc::Response::new(octets))
}

/// Crée `dossier/nom.pdf` à partir des `pages` choisies dans les documents `sources`.
#[tauri::command]
pub async fn assembler_pdf(
    sources: Vec<PathBuf>,
    pages: Vec<PageChoisie>,
    dossier: PathBuf,
    nom: String,
) -> Result<ResultatPdf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut document = assembler(&sources, &pages)?;
        sauvegarder(&mut document, &dossier, &nom)
    })
    .await
    .map_err(|e| format!("Opération interrompue : {e}"))?
}

/// Format des pages d'un PDF créé à partir d'images.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatPage {
    /// La page prend les dimensions de l'image.
    Image,
    /// Page A4, en portrait ou en paysage selon l'image.
    A4,
}

/// Crée `dossier/nom.pdf` avec une image par page.
/// `marge` est exprimée en points (1 pt = 1/72 de pouce) et ne s'applique qu'au format A4.
#[tauri::command]
pub async fn images_en_pdf(
    images: Vec<PathBuf>,
    dossier: PathBuf,
    nom: String,
    format_page: FormatPage,
    marge: f32,
) -> Result<ResultatPdf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut document = creer_depuis_images(&images, format_page, marge)?;
        sauvegarder(&mut document, &dossier, &nom)
    })
    .await
    .map_err(|e| format!("Opération interrompue : {e}"))?
}

// --- Assemblage de pages ---

/// Ouvre un PDF. Les PDF protégés seulement contre la modification (sans mot de passe d'ouverture)
/// sont déchiffrés automatiquement par lopdf.
fn charger(chemin: &Path) -> Result<Document, String> {
    let nom = chemin.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let document = Document::load(chemin)
        .map_err(|e| format!("Impossible de lire « {nom} » (fichier abîmé ou protégé ?) : {e}"))?;
    if document.is_encrypted() {
        return Err(format!("« {nom} » est protégé par un mot de passe"));
    }
    Ok(document)
}

/// Attributs qu'une page peut hériter du nœud parent dans l'arbre des pages d'un PDF.
const ATTRIBUTS_HERITES: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

/// Recopie dans la page les attributs hérités de ses parents : indispensable avant de la déplacer
/// dans un autre arbre de pages, sinon elle perdrait par exemple ses polices ou ses dimensions.
fn recopier_heritage(document: &mut Document, page: ObjectId) -> Result<(), String> {
    let mut a_recopier: Vec<(&[u8], Object)> = Vec::new();
    let dictionnaire = document.get_dictionary(page).map_err(|e| e.to_string())?;
    let mut parent = dictionnaire.get(b"Parent").and_then(Object::as_reference).ok();
    // Garde-fou contre un arbre de pages circulaire (fichier abîmé)
    let mut profondeur = 0;
    while let (Some(id), true) = (parent, profondeur < 64) {
        let noeud = document.get_dictionary(id).map_err(|e| e.to_string())?;
        for cle in ATTRIBUTS_HERITES {
            let deja = dictionnaire.has(cle) || a_recopier.iter().any(|(c, _)| *c == cle);
            if let (false, Ok(valeur)) = (deja, noeud.get(cle)) {
                a_recopier.push((cle, valeur.clone()));
            }
        }
        parent = noeud.get(b"Parent").and_then(Object::as_reference).ok();
        profondeur += 1;
    }

    let dictionnaire = document
        .get_object_mut(page)
        .and_then(Object::as_dict_mut)
        .map_err(|e| e.to_string())?;
    for (cle, valeur) in a_recopier {
        dictionnaire.set(cle, valeur);
    }
    Ok(())
}

/// Construit un nouveau document avec les `pages` choisies, dans l'ordre donné.
pub fn assembler(sources: &[PathBuf], pages: &[PageChoisie]) -> Result<Document, String> {
    if pages.is_empty() {
        return Err("Le document produit doit contenir au moins une page".into());
    }

    let mut objets: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut pages_sources: Vec<BTreeMap<u32, ObjectId>> = Vec::new();
    let mut prochain_id = 1;

    for chemin in sources {
        let mut document = charger(chemin)?;
        // Numérotation sans chevauchement entre les documents
        document.renumber_objects_with(prochain_id);
        prochain_id = document.max_id + 1;

        let ids = document.get_pages();
        for id in ids.values() {
            recopier_heritage(&mut document, *id)?;
        }
        pages_sources.push(ids);

        // On garde tout sauf la structure (catalogue, arbre des pages, signets), reconstruite plus bas
        for (id, objet) in document.objects {
            match objet.type_name().unwrap_or_default() {
                b"Catalog" | b"Pages" | b"Outlines" | b"Outline" => {}
                _ => {
                    objets.insert(id, objet);
                }
            }
        }
    }

    let id_pages: ObjectId = (prochain_id, 0);
    let id_catalogue: ObjectId = (prochain_id + 1, 0);
    let mut enfants = Vec::with_capacity(pages.len());

    for choix in pages {
        let id = pages_sources
            .get(choix.source)
            .and_then(|p| p.get(&choix.page))
            .copied()
            .ok_or_else(|| format!("La page {} n'existe pas", choix.page))?;
        if enfants.contains(&Object::Reference(id)) {
            return Err(format!("La page {} est choisie deux fois", choix.page));
        }
        let page = objets
            .get_mut(&id)
            .and_then(|o| o.as_dict_mut().ok())
            .ok_or_else(|| format!("La page {} est illisible", choix.page))?;
        page.set("Parent", id_pages);
        if choix.rotation % 360 != 0 {
            let actuelle = page.get(b"Rotate").and_then(Object::as_i64).unwrap_or(0);
            page.set("Rotate", (actuelle + choix.rotation).rem_euclid(360));
        }
        enfants.push(Object::Reference(id));
    }

    let mut document = Document::with_version("1.7");
    document.objects = objets;
    document.objects.insert(
        id_pages,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Count" => enfants.len() as i64,
            "Kids" => enfants,
        }),
    );
    document.objects.insert(
        id_catalogue,
        Object::Dictionary(dictionary! { "Type" => "Catalog", "Pages" => id_pages }),
    );
    document.trailer.set("Root", id_catalogue);
    document.max_id = prochain_id + 1;
    // Supprime ce qui n'est plus utilisé (pages retirées, leurs images…) puis renumérote proprement
    document.prune_objects();
    document.renumber_objects();
    Ok(document)
}

/// Compresse et enregistre le document dans `dossier/nom.pdf` sans écraser de fichier existant.
fn sauvegarder(document: &mut Document, dossier: &Path, nom: &str) -> Result<ResultatPdf, String> {
    document.compress();
    let mut octets = Vec::new();
    document.save_to(&mut octets).map_err(|e| format!("Échec de l'écriture du PDF : {e}"))?;
    let chemin = enregistrer(dossier, nom, "pdf", &octets)?;
    Ok(ResultatPdf { chemin, taille: octets.len() as u64, pages: document.get_pages().len() as u32 })
}

// --- Images vers PDF ---

/// Dimensions d'une page A4 en points.
const A4: (f32, f32) = (595.28, 841.89);
/// Résolution supposée des images pour le format « taille de l'image » (96 points par pouce, comme Windows).
const PPP_ECRAN: f32 = 96.0;

/// Une image prête à être placée dans le PDF.
struct ImagePdf {
    largeur: u32,
    hauteur: u32,
    flux: Stream,
    /// Masque de transparence éventuel (images PNG avec canal alpha).
    masque: Option<Stream>,
}

/// Prépare une image pour le PDF. Les JPEG sont intégrés tels quels (aucune perte, fichier léger) ;
/// les autres formats sont décodés puis compressés sans perte.
fn preparer_image(chemin: &Path) -> Result<ImagePdf, String> {
    let extension = extension_de(chemin);
    let octets = std::fs::read(chemin).map_err(|e| format!("Impossible d'ouvrir l'image : {e}"))?;

    if FormatImage::depuis_extension(&extension) == Some(FormatImage::Jpeg) {
        if let Some(image) = jpeg_tel_quel(&octets) {
            return Ok(image);
        }
        // JPEG pivoté (EXIF) ou en CMJN : on le redresse puis on le réencode en JPEG de haute qualité
        let image = lire(&octets, &extension)?;
        let jpeg = encoder(&image, FormatImage::Jpeg, 92)?;
        return jpeg_tel_quel(&jpeg).ok_or_else(|| "JPEG inattendu".to_string());
    }

    let image = lire(&octets, &extension)?;
    let (largeur, hauteur) = (image.width(), image.height());
    let masque = image.color().has_alpha().then(|| {
        let alpha: Vec<u8> = image.to_rgba8().pixels().map(|p| p[3]).collect();
        flux_image(largeur, hauteur, "DeviceGray", alpha)
    });
    let rgb = image.to_rgb8().into_raw();
    Ok(ImagePdf { largeur, hauteur, flux: flux_image(largeur, hauteur, "DeviceRGB", rgb), masque })
}

/// Intègre un JPEG sans le décoder, s'il est en RVB ou en niveaux de gris et sans rotation EXIF.
fn jpeg_tel_quel(octets: &[u8]) -> Option<ImagePdf> {
    let mut decodeur = ImageReader::new(Cursor::new(octets)).with_guessed_format().ok()?.into_decoder().ok()?;
    let orientation = decodeur.orientation().ok()?;
    if orientation != image::metadata::Orientation::NoTransforms {
        return None;
    }
    let (largeur, hauteur) = decodeur.dimensions();
    let espace = match composantes_jpeg(octets)? {
        1 => "DeviceGray",
        3 => "DeviceRGB",
        _ => return None, // CMJN : laissé au chemin « décoder puis réencoder »
    };
    let flux = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => largeur as i64,
            "Height" => hauteur as i64,
            "ColorSpace" => espace,
            "BitsPerComponent" => 8,
            "Filter" => "DCTDecode",
        },
        octets.to_vec(),
    );
    Some(ImagePdf { largeur, hauteur, flux, masque: None })
}

/// Nombre de composantes de couleur d'un JPEG, lu dans son en-tête « SOF ».
fn composantes_jpeg(octets: &[u8]) -> Option<u8> {
    let mut i = 2;
    while i + 9 < octets.len() {
        if octets[i] != 0xFF {
            return None;
        }
        let marqueur = octets[i + 1];
        let longueur = u16::from_be_bytes([octets[i + 2], octets[i + 3]]) as usize;
        // SOF0 à SOF15, hors DHT (C4), JPG (C8) et DAC (CC)
        if (0xC0..=0xCF).contains(&marqueur) && ![0xC4, 0xC8, 0xCC].contains(&marqueur) {
            return octets.get(i + 9).copied();
        }
        i += 2 + longueur;
    }
    None
}

/// Image brute compressée sans perte (FlateDecode).
fn flux_image(largeur: u32, hauteur: u32, espace: &str, pixels: Vec<u8>) -> Stream {
    let mut flux = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => largeur as i64,
            "Height" => hauteur as i64,
            "ColorSpace" => espace,
            "BitsPerComponent" => 8,
        },
        pixels,
    );
    // Sans effet si la compression n'apporte rien
    let _ = flux.compress();
    flux
}

/// Crée un document avec une image par page.
pub fn creer_depuis_images(images: &[PathBuf], format_page: FormatPage, marge: f32) -> Result<Document, String> {
    if images.is_empty() {
        return Err("Ajoutez au moins une image".into());
    }

    let mut document = Document::with_version("1.7");
    let id_pages = document.new_object_id();
    let mut enfants = Vec::with_capacity(images.len());

    for chemin in images {
        let nom = chemin.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let image = preparer_image(chemin).map_err(|e| format!("« {nom} » : {e}"))?;

        // Taille de l'image en points, puis taille de la page
        let (l_img, h_img) = (
            image.largeur as f32 * 72.0 / PPP_ECRAN,
            image.hauteur as f32 * 72.0 / PPP_ECRAN,
        );
        let (l_page, h_page, marge) = match format_page {
            FormatPage::Image => (l_img, h_img, 0.0),
            // A4 en paysage pour les images plus larges que hautes
            FormatPage::A4 if l_img > h_img => (A4.1, A4.0, marge),
            FormatPage::A4 => (A4.0, A4.1, marge),
        };
        // L'image est agrandie ou réduite pour remplir la zone utile, puis centrée
        let echelle = ((l_page - 2.0 * marge) / l_img).min((h_page - 2.0 * marge) / h_img);
        let (l, h) = (l_img * echelle, h_img * echelle);
        let (x, y) = ((l_page - l) / 2.0, (h_page - h) / 2.0);

        let mut flux = image.flux;
        if let Some(masque) = image.masque {
            let id_masque = document.add_object(masque);
            flux.dict.set("SMask", id_masque);
        }
        let id_image = document.add_object(flux);
        let contenu = format!("q {l:.2} 0 0 {h:.2} {x:.2} {y:.2} cm /Im0 Do Q").into_bytes();
        let id_contenu = document.add_object(Stream::new(Dictionary::new(), contenu));
        let id_page = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => id_pages,
            "MediaBox" => vec![0.into(), 0.into(), l_page.into(), h_page.into()],
            "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => id_image } },
            "Contents" => id_contenu,
        });
        enfants.push(Object::Reference(id_page));
    }

    document.objects.insert(
        id_pages,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Count" => enfants.len() as i64,
            "Kids" => enfants,
        }),
    );
    let id_catalogue = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => id_pages });
    document.trailer.set("Root", id_catalogue);
    Ok(document)
}

// --- Compression de PDF ---

/// Niveau de compression d'un PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NiveauPdf {
    Legere,
    Recommandee,
    Forte,
}

impl NiveauPdf {
    /// Qualité JPEG des images réencodées.
    fn qualite(self) -> u8 {
        match self {
            NiveauPdf::Legere => 80,
            NiveauPdf::Recommandee => 65,
            NiveauPdf::Forte => 45,
        }
    }

    /// Plus grand côté autorisé pour une image, en pixels (au-delà, elle est réduite).
    fn cote_max(self) -> u32 {
        match self {
            NiveauPdf::Legere => 3000,
            NiveauPdf::Recommandee => 2000,
            NiveauPdf::Forte => 1400,
        }
    }
}

/// Résultat renvoyé à l'interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatCompressionPdf {
    /// `None` si le PDF n'a pas pu être allégé : aucun fichier n'est alors créé.
    pub chemin: Option<PathBuf>,
    pub taille: u64,
    /// Nombre d'images réencodées.
    pub images: u32,
}

/// Compresse `source` dans `dossier/nom (compressé).pdf`.
#[tauri::command]
pub async fn compresser_pdf(
    source: PathBuf,
    dossier: PathBuf,
    niveau: NiveauPdf,
) -> Result<ResultatCompressionPdf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let taille_originale = std::fs::metadata(&source).map(|m| m.len()).unwrap_or(u64::MAX);
        let (octets, images) = compresser_document(&source, niveau)?;
        if octets.len() as u64 >= taille_originale {
            return Ok(ResultatCompressionPdf { chemin: None, taille: taille_originale, images: 0 });
        }
        let nom = format!("{} (compressé)", crate::conversion::nom_sans_extension(&source));
        let chemin = enregistrer(&dossier, &nom, "pdf", &octets)?;
        Ok(ResultatCompressionPdf { chemin: Some(chemin), taille: octets.len() as u64, images })
    })
    .await
    .map_err(|e| format!("Compression interrompue : {e}"))?
}

/// Réencode les images du PDF puis l'enregistre de façon compacte. Renvoie les octets et le
/// nombre d'images allégées. Le texte, les polices et les dessins vectoriels ne sont pas touchés.
pub fn compresser_document(source: &Path, niveau: NiveauPdf) -> Result<(Vec<u8>, u32), String> {
    let mut document = charger(source)?;

    // Les masques de transparence (SMask) restent intacts : les passer en JPEG créerait des halos
    let masques: Vec<ObjectId> = document
        .objects
        .values()
        .filter_map(|o| o.as_stream().ok())
        .filter_map(|s| s.dict.get(b"SMask").and_then(Object::as_reference).ok())
        .collect();
    let images: Vec<ObjectId> = document
        .objects
        .iter()
        .filter(|(id, o)| {
            !masques.contains(id)
                && o.as_stream()
                    .map(|s| s.dict.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Image".as_slice()))
                    .unwrap_or(false)
        })
        .map(|(id, _)| *id)
        .collect();

    let mut allegees = 0;
    for id in images {
        let espace = espace_couleur(&document, id);
        if let Ok(Object::Stream(flux)) = document.get_object_mut(id) {
            if let Some(composantes) = espace {
                if recompresser_image(flux, composantes, niveau) {
                    allegees += 1;
                }
            }
        }
    }

    // Supprime les objets inutilisés, compresse les flux restants sans perte,
    // puis regroupe les petits objets dans des « flux d'objets » (PDF 1.5+)
    document.prune_objects();
    document.compress();
    if document.version.as_str() < "1.5" {
        document.version = "1.5".into();
    }
    let mut octets = Vec::new();
    let options = lopdf::SaveOptions::builder().use_object_streams(true).use_xref_streams(true).build();
    document
        .save_with_options(&mut octets, options)
        .map_err(|e| format!("Échec de l'écriture du PDF : {e}"))?;
    Ok((octets, allegees))
}

/// Nombre de composantes de couleur de l'image `id` (1 = gris, 3 = RVB), si elle peut être
/// réencodée en JPEG sans changer son espace de couleur. `None` pour le CMJN, les palettes, etc.
fn espace_couleur(document: &Document, id: ObjectId) -> Option<u8> {
    let flux = document.get_object(id).ok()?.as_stream().ok()?;
    let espace = flux.dict.get(b"ColorSpace").ok()?;
    let espace = match espace {
        Object::Reference(r) => document.get_object(*r).ok()?,
        autre => autre,
    };
    match espace {
        Object::Name(nom) if nom == b"DeviceRGB" => Some(3),
        Object::Name(nom) if nom == b"DeviceGray" => Some(1),
        // Profil ICC : on garde le profil, seul compte le nombre de composantes
        Object::Array(tableau) if tableau.first().and_then(|o| o.as_name().ok()) == Some(b"ICCBased") => {
            let profil = tableau.get(1)?.as_reference().ok()?;
            let n = document.get_object(profil).ok()?.as_stream().ok()?.dict.get(b"N").ok()?.as_i64().ok()?;
            matches!(n, 1 | 3).then_some(n as u8)
        }
        _ => None,
    }
}

/// Réencode une image du PDF en JPEG (réduite si elle est démesurée).
/// Ne remplace l'image que si le résultat est nettement plus léger. Renvoie vrai si elle a été remplacée.
fn recompresser_image(flux: &mut Stream, composantes: u8, niveau: NiveauPdf) -> bool {
    let entier = |cle: &[u8]| flux.dict.get(cle).and_then(Object::as_i64).ok();
    let (Some(largeur), Some(hauteur)) = (entier(b"Width"), entier(b"Height")) else {
        return false;
    };
    // Images 8 bits seulement, sans masque binaire ni table de décodage particulière
    let bits = entier(b"BitsPerComponent").unwrap_or(8);
    if bits != 8
        || flux.dict.has(b"Decode")
        || flux.dict.get(b"ImageMask").and_then(Object::as_bool).unwrap_or(false)
        || largeur * hauteur < 96 * 96
    {
        return false;
    }
    let (largeur, hauteur) = (largeur as u32, hauteur as u32);
    let filtres: Vec<Vec<u8>> = flux.filters().map(|f| f.iter().map(|n| n.to_vec()).collect()).unwrap_or_default();

    // Pixels de l'image : JPEG d'origine décodé, ou pixels bruts (compressés sans perte ou non)
    let image = match filtres.iter().map(Vec::as_slice).collect::<Vec<_>>().as_slice() {
        [b"DCTDecode"] => image::load_from_memory_with_format(&flux.content, image::ImageFormat::Jpeg).ok(),
        [] | [b"FlateDecode"] => {
            let attendu = (largeur * hauteur * composantes as u32) as usize;
            // Limite de décompression : protège contre un fichier piégé (« bombe de décompression »)
            let pixels = flux.get_plain_content_with_limit(attendu + 1024).ok().filter(|p| p.len() == attendu);
            pixels.and_then(|p| match composantes {
                1 => image::GrayImage::from_raw(largeur, hauteur, p).map(image::DynamicImage::ImageLuma8),
                _ => image::RgbImage::from_raw(largeur, hauteur, p).map(image::DynamicImage::ImageRgb8),
            })
        }
        _ => None,
    };
    let Some(mut image) = image else {
        return false;
    };

    let cote_max = niveau.cote_max();
    let reduite = largeur > cote_max || hauteur > cote_max;
    if reduite {
        image = image.resize(cote_max, cote_max, image::imageops::FilterType::Lanczos3);
    }

    // JPEG en niveaux de gris ou en RVB, selon l'espace de couleur d'origine
    let mut jpeg = Vec::new();
    let encodeur = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, niveau.qualite());
    let encode = match composantes {
        1 => image::DynamicImage::ImageLuma8(image.to_luma8()).write_with_encoder(encodeur),
        _ => image::DynamicImage::ImageRgb8(image.to_rgb8()).write_with_encoder(encodeur),
    };
    if encode.is_err() {
        return false;
    }

    // On ne remplace que si le gain est réel (au moins 10 %)
    if !reduite && jpeg.len() * 10 >= flux.content.len() * 9 {
        return false;
    }
    flux.dict.set("Filter", "DCTDecode");
    flux.dict.remove(b"DecodeParms");
    flux.dict.set("Width", image.width() as i64);
    flux.dict.set("Height", image.height() as i64);
    flux.dict.set("BitsPerComponent", 8);
    flux.set_content(jpeg);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::tests::dossier_test;
    use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};

    /// Crée un PDF de `n` pages ; la page i (à partir de 1) mesure 100 + i points de large,
    /// ce qui permet de reconnaître chaque page après l'opération.
    fn pdf_test(dossier: &Path, nom: &str, n: u32, decalage: u32) -> PathBuf {
        let mut document = Document::with_version("1.7");
        let id_pages = document.new_object_id();
        let enfants: Vec<Object> = (1..=n)
            .map(|i| {
                let contenu = document.add_object(Stream::new(Dictionary::new(), b"".to_vec()));
                document
                    .add_object(dictionary! {
                        "Type" => "Page",
                        "Parent" => id_pages,
                        "MediaBox" => vec![0.into(), 0.into(), ((100 + decalage + i) as i64).into(), 200.into()],
                        "Contents" => contenu,
                    })
                    .into()
            })
            .collect();
        document.objects.insert(
            id_pages,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => n as i64, "Kids" => enfants }),
        );
        let catalogue = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => id_pages });
        document.trailer.set("Root", catalogue);
        let chemin = dossier.join(nom);
        document.save(&chemin).unwrap();
        chemin
    }

    /// Largeur (MediaBox) de chaque page du PDF, dans l'ordre.
    fn largeurs(chemin: &Path) -> Vec<i64> {
        let document = Document::load(chemin).unwrap();
        document
            .get_pages()
            .values()
            .map(|id| {
                let page = document.get_dictionary(*id).unwrap();
                page.get(b"MediaBox").unwrap().as_array().unwrap()[2].as_i64().unwrap()
            })
            .collect()
    }

    fn page(source: usize, page: u32) -> PageChoisie {
        PageChoisie { source, page, rotation: 0 }
    }

    #[test]
    fn fusionne_deux_documents() {
        let dossier = dossier_test("pdf-fusion");
        let a = pdf_test(&dossier, "a.pdf", 2, 0);
        let b = pdf_test(&dossier, "b.pdf", 3, 10);
        let pages = vec![page(0, 1), page(0, 2), page(1, 1), page(1, 2), page(1, 3)];

        let mut document = assembler(&[a, b], &pages).unwrap();
        let res = sauvegarder(&mut document, &dossier, "fusion").unwrap();
        assert_eq!(res.pages, 5);
        assert_eq!(largeurs(&res.chemin), vec![101, 102, 111, 112, 113]);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn reorganise_supprime_et_fait_pivoter() {
        let dossier = dossier_test("pdf-pages");
        let source = pdf_test(&dossier, "source.pdf", 4, 0);
        let mut pages = vec![page(0, 4), page(0, 1), page(0, 3)]; // la page 2 est supprimée
        pages[1].rotation = 90;

        let mut document = assembler(&[source], &pages).unwrap();
        let res = sauvegarder(&mut document, &dossier, "resultat").unwrap();
        assert_eq!(largeurs(&res.chemin), vec![104, 101, 103]);

        let relu = Document::load(&res.chemin).unwrap();
        let deuxieme = *relu.get_pages().get(&2).unwrap();
        assert_eq!(relu.get_dictionary(deuxieme).unwrap().get(b"Rotate").unwrap().as_i64().unwrap(), 90);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn refuse_un_document_vide() {
        let dossier = dossier_test("pdf-vide");
        let source = pdf_test(&dossier, "source.pdf", 2, 0);
        assert!(assembler(&[source], &[]).is_err());
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn cree_un_pdf_a_partir_d_images() {
        let dossier = dossier_test("pdf-images");
        let jpeg = dossier.join("photo.jpg");
        let png = dossier.join("logo.png");
        let rgb = DynamicImage::ImageRgb8(RgbImage::from_pixel(300, 200, Rgb([200, 30, 30])));
        std::fs::write(&jpeg, encoder(&rgb, FormatImage::Jpeg, 90).unwrap()).unwrap();
        RgbaImage::from_pixel(50, 80, Rgba([0, 0, 255, 128])).save(&png).unwrap();

        // Format « image » : la page prend la taille de l'image (300 px à 96 ppp = 225 pt)
        let mut document = creer_depuis_images(&[jpeg.clone(), png.clone()], FormatPage::Image, 0.0).unwrap();
        let res = sauvegarder(&mut document, &dossier, "images").unwrap();
        assert_eq!(res.pages, 2);
        let relu = Document::load(&res.chemin).unwrap();
        let premiere = *relu.get_pages().get(&1).unwrap();
        let boite = relu.get_dictionary(premiere).unwrap().get(b"MediaBox").unwrap().as_array().unwrap().clone();
        assert!((boite[2].as_float().unwrap() - 225.0).abs() < 0.1);

        // Format A4 : la photo plus large que haute passe en paysage
        let mut document = creer_depuis_images(&[jpeg], FormatPage::A4, 36.0).unwrap();
        let res = sauvegarder(&mut document, &dossier, "a4").unwrap();
        let relu = Document::load(&res.chemin).unwrap();
        let premiere = *relu.get_pages().get(&1).unwrap();
        let boite = relu.get_dictionary(premiere).unwrap().get(b"MediaBox").unwrap().as_array().unwrap().clone();
        assert!((boite[2].as_float().unwrap() - A4.1).abs() < 0.1);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn compresse_les_images_d_un_pdf() {
        let dossier = dossier_test("pdf-compression");
        // Grande image détaillée en PNG : intégrée sans perte (FlateDecode), donc lourde
        let png = dossier.join("scan.png");
        RgbImage::from_fn(2600, 1800, |x, y| Rgb([(x * 7 + y) as u8, (x ^ y) as u8, (x * y / 7) as u8]))
            .save(&png)
            .unwrap();
        let mut document = creer_depuis_images(&[png], FormatPage::A4, 0.0).unwrap();
        let source = sauvegarder(&mut document, &dossier, "lourd").unwrap();

        let (octets, images) = compresser_document(&source.chemin, NiveauPdf::Forte).unwrap();
        assert_eq!(images, 1);
        assert!(octets.len() * 2 < source.taille as usize, "{} devrait être bien < {}", octets.len(), source.taille);

        // Le PDF produit reste lisible, avec la même page et une image réduite à 1400 px
        let relu = Document::load_mem(&octets).unwrap();
        assert_eq!(relu.get_pages().len(), 1);
        let image = relu
            .objects
            .values()
            .filter_map(|o| o.as_stream().ok())
            .find(|s| s.dict.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Image".as_slice()))
            .unwrap();
        assert_eq!(image.dict.get(b"Width").unwrap().as_i64().unwrap(), 1400);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn lit_les_composantes_d_un_jpeg() {
        let rgb = DynamicImage::ImageRgb8(RgbImage::new(8, 8));
        assert_eq!(composantes_jpeg(&encoder(&rgb, FormatImage::Jpeg, 80).unwrap()), Some(3));
    }
}
