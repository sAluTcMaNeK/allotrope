//! Images contenues dans les PDF : compression (réencodage) et extraction.
//!
//! - `decoder` transforme une image du PDF en pixels (gris, RVB, CMJN, palette, transparence) ;
//! - `compresser_pdf` réencode les images, par taux ou pour viser une taille de fichier ;
//! - `analyser_images_pdf` / `extraire_images_pdf` listent les images (avec miniatures) et les
//!   enregistrent, une par une dans un dossier ou toutes dans une archive ZIP.

use crate::compression::qualite_pour;
use crate::conversion::{creer_fichier_libre, encoder, enregistrer, nom_sans_extension, FormatImage};
use crate::pdf::charger;
use base64::Engine;
use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, ImageFormat, RgbImage};
use lopdf::{Document, Object, ObjectId, Stream};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

// --- Lecture des images d'un PDF ---

/// Espace de couleur d'une image, ramené à ce que l'on sait décoder.
#[derive(Debug, Clone, PartialEq)]
enum Espace {
    Gris,
    Rvb,
    Cmjn,
    /// Couleurs indexées : chaque pixel est un numéro dans une table de couleurs.
    Palette { base: Box<Espace>, couleurs: Vec<u8> },
}

impl Espace {
    /// Nombre de valeurs par pixel dans les données de l'image.
    fn composantes(&self) -> usize {
        match self {
            Espace::Gris | Espace::Palette { .. } => 1,
            Espace::Rvb => 3,
            Espace::Cmjn => 4,
        }
    }
}

/// Suit une référence (« 12 0 R ») jusqu'à l'objet désigné.
fn resoudre<'a>(document: &'a Document, objet: &'a Object) -> &'a Object {
    match objet {
        Object::Reference(id) => document.get_object(*id).unwrap_or(objet),
        _ => objet,
    }
}

fn lire_espace(document: &Document, objet: &Object) -> Option<Espace> {
    match resoudre(document, objet) {
        Object::Name(nom) => match nom.as_slice() {
            b"DeviceGray" | b"G" | b"CalGray" => Some(Espace::Gris),
            b"DeviceRGB" | b"RGB" | b"CalRGB" => Some(Espace::Rvb),
            b"DeviceCMYK" | b"CMYK" => Some(Espace::Cmjn),
            _ => None,
        },
        Object::Array(tableau) => match tableau.first()?.as_name().ok()? {
            b"ICCBased" => {
                let profil = resoudre(document, tableau.get(1)?).as_stream().ok()?;
                match profil.dict.get(b"N").and_then(Object::as_i64).ok()? {
                    1 => Some(Espace::Gris),
                    3 => Some(Espace::Rvb),
                    4 => Some(Espace::Cmjn),
                    _ => None,
                }
            }
            b"CalRGB" => Some(Espace::Rvb),
            b"CalGray" => Some(Espace::Gris),
            b"Indexed" | b"I" => {
                let base = lire_espace(document, tableau.get(1)?)?;
                let couleurs = match resoudre(document, tableau.get(3)?) {
                    Object::String(octets, _) => octets.clone(),
                    Object::Stream(flux) => flux.get_plain_content().ok()?,
                    _ => return None,
                };
                Some(Espace::Palette { base: Box::new(base), couleurs })
            }
            _ => None,
        },
        _ => None,
    }
}

/// Filtres de compression d'un flux (« DCTDecode », « FlateDecode »…), du premier au dernier.
fn filtres(flux: &Stream) -> Vec<Vec<u8>> {
    flux.filters().map(|f| f.iter().map(|n| n.to_vec()).collect()).unwrap_or_default()
}

/// Vrai si le flux est une image (et pas un masque binaire « ImageMask »).
fn est_image(flux: &Stream) -> bool {
    flux.dict.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Image".as_slice())
        && !flux.dict.get(b"ImageMask").and_then(Object::as_bool).unwrap_or(false)
}

/// Objets qui servent de masques de transparence à d'autres images : ce ne sont pas de « vraies » images.
fn ids_masques(document: &Document) -> HashSet<ObjectId> {
    document
        .objects
        .values()
        .filter_map(|o| o.as_stream().ok())
        .flat_map(|s| {
            [b"SMask".as_slice(), b"Mask".as_slice()]
                .into_iter()
                .filter_map(|cle| s.dict.get(cle).and_then(Object::as_reference).ok())
        })
        .collect()
}

/// Convertit des pixels bruts (1 à 16 bits par valeur) en image RVB ou en niveaux de gris.
fn brut_vers_image(pixels: &[u8], largeur: u32, hauteur: u32, bits: u32, espace: &Espace) -> Option<DynamicImage> {
    let n = espace.composantes();
    let octets_par_ligne = (largeur as usize * n * bits as usize).div_ceil(8);
    if pixels.len() < octets_par_ligne * hauteur as usize || !matches!(bits, 1 | 2 | 4 | 8 | 16) {
        return None;
    }
    let max = (1u32 << bits.min(8)) - 1;
    // i-ème valeur de la ligne y (pour 16 bits, on garde l'octet de poids fort)
    let valeur = |y: usize, i: usize| -> u32 {
        let ligne = &pixels[y * octets_par_ligne..];
        match bits {
            8 => ligne[i] as u32,
            16 => ligne[i * 2] as u32,
            _ => {
                let bit = i * bits as usize;
                ((ligne[bit / 8] >> (8 - bits as usize - bit % 8)) as u32) & max
            }
        }
    };
    // Une valeur ramenée sur 0–255 (sauf pour une palette, où c'est un numéro de couleur)
    let echelle = |v: u32| if bits >= 8 { v as u8 } else { (v * 255 / max) as u8 };
    let cmjn_vers_rvb = |c: u8, m: u8, j: u8, k: u8| {
        let canal = |x: u8| 255u8.saturating_sub(x.saturating_add(k));
        [canal(c), canal(m), canal(j)]
    };

    let (l, h) = (largeur as usize, hauteur as usize);
    match espace {
        Espace::Gris => {
            let gris: Vec<u8> = (0..h).flat_map(|y| (0..l).map(move |x| (y, x))).map(|(y, x)| echelle(valeur(y, x))).collect();
            GrayImage::from_raw(largeur, hauteur, gris).map(DynamicImage::ImageLuma8)
        }
        _ => {
            let mut rvb = Vec::with_capacity(l * h * 3);
            for y in 0..h {
                for x in 0..l {
                    let pixel = match espace {
                        Espace::Rvb => [0, 1, 2].map(|c| echelle(valeur(y, x * 3 + c))),
                        Espace::Cmjn => {
                            let [c, m, j, k] = [0, 1, 2, 3].map(|i| echelle(valeur(y, x * 4 + i)));
                            cmjn_vers_rvb(c, m, j, k)
                        }
                        Espace::Palette { base, couleurs } => {
                            let nb = base.composantes();
                            let debut = valeur(y, x) as usize * nb;
                            let couleur = couleurs.get(debut..debut + nb).unwrap_or(&[0, 0, 0, 0][..nb]);
                            match base.as_ref() {
                                Espace::Gris => [couleur[0]; 3],
                                Espace::Cmjn => cmjn_vers_rvb(couleur[0], couleur[1], couleur[2], couleur[3]),
                                _ => [couleur[0], couleur[1], couleur[2]],
                            }
                        }
                        Espace::Gris => unreachable!(),
                    };
                    rvb.extend_from_slice(&pixel);
                }
            }
            RgbImage::from_raw(largeur, hauteur, rvb).map(DynamicImage::ImageRgb8)
        }
    }
}

/// Décode une image du PDF en pixels, sans sa transparence.
fn decoder_sans_masque(document: &Document, flux: &Stream) -> Option<DynamicImage> {
    let entier = |cle: &[u8]| flux.dict.get(cle).and_then(Object::as_i64).ok();
    let (largeur, hauteur) = (entier(b"Width")? as u32, entier(b"Height")? as u32);
    if largeur == 0 || hauteur == 0 || largeur > 30_000 || hauteur > 30_000 {
        return None;
    }
    let filtres = filtres(flux);
    match filtres.last().map(Vec::as_slice) {
        Some(b"DCTDecode") => {
            // JPEG, éventuellement enveloppé dans un autre filtre
            let jpeg = if filtres.len() == 1 { flux.content.clone() } else { flux.decompressed_content().ok()? };
            image::load_from_memory_with_format(&jpeg, ImageFormat::Jpeg).ok()
        }
        // JPEG 2000 : extrait tel quel, mais pas décodé (pas de miniature)
        Some(b"JPXDecode") => None,
        _ => {
            let espace = lire_espace(document, flux.dict.get(b"ColorSpace").ok()?)?;
            let bits = entier(b"BitsPerComponent").unwrap_or(8) as u32;
            let attendu = (largeur as usize * espace.composantes() * bits as usize).div_ceil(8) * hauteur as usize;
            // Limite de décompression : protège contre un fichier piégé (« bombe de décompression »)
            let pixels = flux.get_plain_content_with_limit(attendu + 4096).ok()?;
            brut_vers_image(&pixels, largeur, hauteur, bits, &espace)
        }
    }
}

/// Décode une image du PDF, avec sa transparence (masque « SMask ») s'il y en a une.
fn decoder(document: &Document, flux: &Stream) -> Option<DynamicImage> {
    let image = decoder_sans_masque(document, flux)?;
    let masque = flux
        .dict
        .get(b"SMask")
        .and_then(Object::as_reference)
        .ok()
        .and_then(|id| document.get_object(id).ok()?.as_stream().ok())
        .and_then(|m| decoder_sans_masque(document, m));
    let Some(masque) = masque else { return Some(image) };
    // Le masque peut avoir d'autres dimensions que l'image : on l'ajuste
    let masque = masque.resize_exact(image.width(), image.height(), FilterType::Triangle).to_luma8();
    let mut rgba = image.to_rgba8();
    for (pixel, alpha) in rgba.pixels_mut().zip(masque.pixels()) {
        pixel[3] = alpha[0];
    }
    Some(DynamicImage::ImageRgba8(rgba))
}

// --- Compression ---

/// Image du PDF que l'on peut réencoder en JPEG sans changer son espace de couleur (gris ou RVB).
struct Candidat {
    id: ObjectId,
    image: DynamicImage,
    gris: bool,
    taille: usize,
}

/// Images à réencoder : 8 bits, gris ou RVB, compressées sans perte ou en JPEG, sans masque binaire
/// ni table de décodage, et assez grandes pour que cela en vaille la peine. Décodées en parallèle.
fn candidats(document: &Document) -> Vec<Candidat> {
    let masques = ids_masques(document);
    let ids: Vec<ObjectId> = document
        .objects
        .iter()
        .filter(|(id, o)| !masques.contains(id) && o.as_stream().map(est_image).unwrap_or(false))
        .map(|(id, _)| *id)
        .collect();
    ids.par_iter()
        .filter_map(|id| {
            let flux = document.get_object(*id).ok()?.as_stream().ok()?;
            let entier = |cle: &[u8]| flux.dict.get(cle).and_then(Object::as_i64).ok();
            let gris = match lire_espace(document, flux.dict.get(b"ColorSpace").ok()?)? {
                Espace::Gris => true,
                Espace::Rvb => false,
                _ => return None,
            };
            let filtres = filtres(flux);
            let filtres_ok = matches!(filtres.as_slice(), [] | [_]) && filtres.iter().all(|f| f == b"DCTDecode" || f == b"FlateDecode");
            if entier(b"BitsPerComponent").unwrap_or(8) != 8
                || flux.dict.has(b"Decode")
                || !filtres_ok
                || entier(b"Width")? * entier(b"Height")? < 96 * 96
            {
                return None;
            }
            Some(Candidat { id: *id, image: decoder_sans_masque(document, flux)?, gris, taille: flux.content.len() })
        })
        .collect()
}

/// Copie le document, réencode les images en JPEG (`qualite`, plus grand côté limité à `cote_max`),
/// puis l'enregistre de façon compacte. Renvoie les octets et le nombre d'images allégées.
fn appliquer(base: &Document, candidats: &[Candidat], qualite: u8, cote_max: Option<u32>) -> Result<(Vec<u8>, u32), String> {
    let remplacements: Vec<(ObjectId, Vec<u8>, u32, u32)> = candidats
        .par_iter()
        .filter_map(|c| {
            let reduite = cote_max.is_some_and(|max| c.image.width() > max || c.image.height() > max);
            let image = match cote_max {
                Some(max) if reduite => c.image.resize(max, max, FilterType::Lanczos3),
                _ => c.image.clone(),
            };
            let image = if c.gris { DynamicImage::ImageLuma8(image.to_luma8()) } else { DynamicImage::ImageRgb8(image.to_rgb8()) };
            let mut jpeg = Vec::new();
            image
                .write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, qualite))
                .ok()?;
            // On ne remplace que si le gain est réel (au moins 10 %) ou si l'image a été réduite
            (reduite || jpeg.len() * 10 < c.taille * 9).then(|| (c.id, jpeg, image.width(), image.height()))
        })
        .collect();

    let mut document = base.clone();
    let nombre = remplacements.len() as u32;
    for (id, jpeg, largeur, hauteur) in remplacements {
        if let Ok(Object::Stream(flux)) = document.get_object_mut(id) {
            flux.dict.set("Filter", "DCTDecode");
            flux.dict.remove(b"DecodeParms");
            flux.dict.set("Width", largeur as i64);
            flux.dict.set("Height", hauteur as i64);
            flux.dict.set("BitsPerComponent", 8);
            flux.set_content(jpeg);
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
    document.save_with_options(&mut octets, options).map_err(|e| format!("Échec de l'écriture du PDF : {e}"))?;
    Ok((octets, nombre))
}

/// Cherche le PDF le plus fidèle qui tienne dans `cible` octets : on baisse d'abord la qualité des images,
/// puis, si cela ne suffit pas, leurs dimensions (par paliers de 25 %).
fn viser(base: &Document, candidats: &[Candidat], cible: u64, taille_max: Option<u32>) -> Result<(Vec<u8>, u32), String> {
    let (sans_perte, _) = appliquer(base, &[], 0, None)?;
    let mut plus_petit = sans_perte.len() as u64;
    if plus_petit <= cible {
        return Ok((sans_perte, 0));
    }
    let impossible = |plus_petit: u64| {
        format!(
            "Impossible d'atteindre {} : le plus petit résultat obtenu pèse {} (le texte et les polices ne se réduisent pas)",
            taille_lisible(cible),
            taille_lisible(plus_petit)
        )
    };
    if candidats.is_empty() {
        return Err(impossible(plus_petit));
    }

    let plus_grand_cote = candidats.iter().map(|c| c.image.width().max(c.image.height())).max().unwrap_or(0);
    let mut cote = taille_max.unwrap_or(plus_grand_cote).min(plus_grand_cote);
    for _ in 0..7 {
        // Dichotomie sur la qualité (pas de 5) : on garde la plus haute qui tient dans la cible
        let (mut bas, mut haut) = (2u8, 18u8); // qualité = 5 × palier : de 10 à 90
        let mut meilleur = None;
        while bas <= haut {
            let palier = (bas + haut) / 2;
            let (octets, nombre) = appliquer(base, candidats, palier * 5, Some(cote))?;
            plus_petit = plus_petit.min(octets.len() as u64);
            if octets.len() as u64 <= cible {
                meilleur = Some((octets, nombre));
                bas = palier + 1;
            } else {
                haut = palier - 1;
            }
        }
        if let Some(resultat) = meilleur {
            return Ok(resultat);
        }
        cote = cote * 3 / 4;
        if cote < 200 {
            break;
        }
    }
    Err(impossible(plus_petit))
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

/// Résultat renvoyé à l'interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatCompressionPdf {
    /// `None` si le PDF n'a pas pu être allégé (ou était déjà sous la cible) : aucun fichier créé.
    pub chemin: Option<PathBuf>,
    pub taille: u64,
    /// Nombre d'images réencodées.
    pub images: u32,
}

/// Compresse `source` dans `dossier/nom (compressé).pdf`, comme pour les images :
/// - `taux` (5 à 95) règle la qualité des images quand il n'y a pas de taille cible ;
/// - `taille_cible` : poids visé pour le PDF, en octets ;
/// - `taille_max` : plus grand côté autorisé pour les images, en pixels.
#[tauri::command]
pub async fn compresser_pdf(
    source: PathBuf,
    dossier: PathBuf,
    taux: u8,
    taille_cible: Option<u64>,
    taille_max: Option<u32>,
) -> Result<ResultatCompressionPdf, String> {
    tauri::async_runtime::spawn_blocking(move || compresser(&source, &dossier, taux, taille_cible, taille_max))
        .await
        .map_err(|e| format!("Compression interrompue : {e}"))?
}

pub fn compresser(
    source: &Path,
    dossier: &Path,
    taux: u8,
    taille_cible: Option<u64>,
    taille_max: Option<u32>,
) -> Result<ResultatCompressionPdf, String> {
    let taille_originale = std::fs::metadata(source).map(|m| m.len()).unwrap_or(u64::MAX);
    if taille_cible.is_some_and(|cible| taille_originale <= cible) {
        return Ok(ResultatCompressionPdf { chemin: None, taille: taille_originale, images: 0 });
    }
    let base = charger(source)?;
    let candidats = candidats(&base);
    let (octets, images) = match taille_cible {
        Some(cible) => viser(&base, &candidats, cible, taille_max)?,
        None => appliquer(&base, &candidats, qualite_pour(taux), taille_max)?,
    };
    if octets.len() as u64 >= taille_originale {
        return Ok(ResultatCompressionPdf { chemin: None, taille: taille_originale, images: 0 });
    }
    let chemin = enregistrer(dossier, &format!("{} (compressé)", nom_sans_extension(source)), "pdf", &octets)?;
    Ok(ResultatCompressionPdf { chemin: Some(chemin), taille: octets.len() as u64, images })
}

// --- Extraction ---

/// Une image trouvée dans le PDF.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTrouvee {
    /// Identifiant de l'objet dans le PDF (« 12-0 »), à renvoyer pour l'extraction.
    pub id: String,
    /// Première page où l'image apparaît (`None` si elle n'est utilisée que dans un dessin imbriqué).
    pub page: Option<u32>,
    pub largeur: u32,
    pub hauteur: u32,
    /// Format du fichier extrait : « jpg » (JPEG d'origine, sans perte), « jp2 » (JPEG 2000) ou « png ».
    pub format: String,
    /// Miniature JPEG en « data URL », prête à afficher (`None` si l'image n'a pas pu être décodée).
    pub apercu: Option<String>,
}

/// Première page où chaque image apparaît.
fn pages_des_images(document: &Document) -> HashMap<ObjectId, u32> {
    let mut pages = HashMap::new();
    for (numero, id_page) in document.get_pages() {
        let Ok((ressources, ids_ressources)) = document.get_page_resources(id_page) else { continue };
        let dictionnaires = ressources
            .into_iter()
            .chain(ids_ressources.iter().filter_map(|id| document.get_dictionary(*id).ok()));
        for dictionnaire in dictionnaires {
            let Ok(xobjets) = dictionnaire.get(b"XObject") else { continue };
            let Ok(xobjets) = resoudre(document, xobjets).as_dict() else { continue };
            for (_, valeur) in xobjets.iter() {
                if let Ok(id) = valeur.as_reference() {
                    pages.entry(id).or_insert(numero);
                }
            }
        }
    }
    pages
}

/// Images « réelles » du PDF (sans les masques ni les icônes minuscules), dans l'ordre des pages.
fn images_du_document(document: &Document) -> Vec<(ObjectId, Option<u32>)> {
    let masques = ids_masques(document);
    let pages = pages_des_images(document);
    let mut images: Vec<(ObjectId, Option<u32>)> = document
        .objects
        .iter()
        .filter_map(|(id, objet)| {
            let flux = objet.as_stream().ok()?;
            let dimension = |cle: &[u8]| flux.dict.get(cle).and_then(Object::as_i64).unwrap_or(0);
            (est_image(flux) && !masques.contains(id) && dimension(b"Width") >= 16 && dimension(b"Height") >= 16)
                .then(|| (*id, pages.get(id).copied()))
        })
        .collect();
    images.sort_by_key(|(id, page)| (page.unwrap_or(u32::MAX), *id));
    images
}

fn format_extrait(flux: &Stream) -> &'static str {
    match filtres(flux).last().map(Vec::as_slice) {
        Some(b"DCTDecode") => "jpg",
        Some(b"JPXDecode") => "jp2",
        _ => "png",
    }
}

/// Liste les images d'un PDF, avec une miniature pour chacune (calculées en parallèle).
#[tauri::command]
pub async fn analyser_images_pdf(source: PathBuf) -> Result<Vec<ImageTrouvee>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let document = charger(&source)?;
        let images = images_du_document(&document);
        Ok(images
            .par_iter()
            .filter_map(|(id, page)| {
                let flux = document.get_object(*id).ok()?.as_stream().ok()?;
                let entier = |cle: &[u8]| flux.dict.get(cle).and_then(Object::as_i64).ok();
                let apercu = decoder(&document, flux).and_then(|image| {
                    // Fond blanc pour les images transparentes (encoder() le fait pour le JPEG)
                    let jpeg = encoder(&image.thumbnail(320, 320), FormatImage::Jpeg, 75).ok()?;
                    Some(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(jpeg)))
                });
                Some(ImageTrouvee {
                    id: format!("{}-{}", id.0, id.1),
                    page: *page,
                    largeur: entier(b"Width")? as u32,
                    hauteur: entier(b"Height")? as u32,
                    format: format_extrait(flux).into(),
                    apercu,
                })
            })
            .collect())
    })
    .await
    .map_err(|e| format!("Analyse interrompue : {e}"))?
}

/// Résultat de l'extraction.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatExtraction {
    /// Dossier créé (images séparées) ou archive ZIP.
    pub chemin: PathBuf,
    /// Nombre d'images enregistrées.
    pub nombre: u32,
    /// Images qui n'ont pas pu être lues.
    pub echecs: u32,
}

/// Octets du fichier à enregistrer pour une image : le JPEG (ou JPEG 2000) d'origine tel quel,
/// sans perte ; sinon un PNG (avec sa transparence).
fn fichier_image(document: &Document, flux: &Stream) -> Option<Vec<u8>> {
    match format_extrait(flux) {
        "jpg" | "jp2" if filtres(flux).len() == 1 => Some(flux.content.clone()),
        "jpg" | "jp2" => flux.decompressed_content().ok(),
        _ => {
            let mut png = Vec::new();
            decoder(document, flux)?.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).ok()?;
            Some(png)
        }
    }
}

/// Enregistre les images `ids` du PDF : dans un nouveau dossier « nom (images) », ou dans
/// l'archive « nom (images).zip » si `zip` est vrai.
#[tauri::command]
pub async fn extraire_images_pdf(
    source: PathBuf,
    ids: Vec<String>,
    dossier: PathBuf,
    zip: bool,
) -> Result<ResultatExtraction, String> {
    tauri::async_runtime::spawn_blocking(move || extraire(&source, &ids, &dossier, zip))
        .await
        .map_err(|e| format!("Extraction interrompue : {e}"))?
}

pub fn extraire(source: &Path, ids: &[String], dossier: &Path, zip: bool) -> Result<ResultatExtraction, String> {
    if ids.is_empty() {
        return Err("Choisissez au moins une image".into());
    }
    let document = charger(source)?;
    let voulues: HashSet<ObjectId> = ids
        .iter()
        .filter_map(|id| {
            let (numero, generation) = id.split_once('-')?;
            Some((numero.parse().ok()?, generation.parse().ok()?))
        })
        .collect();
    let images: Vec<(ObjectId, Option<u32>)> =
        images_du_document(&document).into_iter().filter(|(id, _)| voulues.contains(id)).collect();

    // Noms « page 003 - image 1.jpg » (numérotation par page, triable dans l'explorateur)
    let mut par_page: HashMap<Option<u32>, u32> = HashMap::new();
    let noms: Vec<String> = images
        .iter()
        .map(|(id, page)| {
            let rang = par_page.entry(*page).or_insert(0);
            *rang += 1;
            let extension = document
                .get_object(*id)
                .ok()
                .and_then(|o| o.as_stream().ok())
                .map(format_extrait)
                .unwrap_or("png");
            match page {
                Some(p) => format!("page {p:03} - image {rang}.{extension}"),
                None => format!("image {rang}.{extension}"),
            }
        })
        .collect();
    // Lecture (et décodage éventuel) des images en parallèle
    let contenus: Vec<Option<Vec<u8>>> = images
        .par_iter()
        .map(|(id, _)| fichier_image(&document, document.get_object(*id).ok()?.as_stream().ok()?))
        .collect();

    std::fs::create_dir_all(dossier).map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
    let base = format!("{} (images)", nom_sans_extension(source));
    let mut nombre = 0;
    let echecs = contenus.iter().filter(|c| c.is_none()).count() as u32;

    let chemin = if zip {
        let (chemin, fichier) = creer_fichier_libre(dossier, &base, "zip")
            .map_err(|e| format!("Impossible de créer l'archive : {e}"))?;
        // Images déjà compressées (JPEG, PNG) : rangées telles quelles, sans recompression inutile
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut archive = zip::ZipWriter::new(fichier);
        let resultat: Result<(), String> = (|| {
            for (nom, contenu) in noms.iter().zip(&contenus) {
                if let Some(contenu) = contenu {
                    archive.start_file(nom.as_str(), options).map_err(|e| e.to_string())?;
                    archive.write_all(contenu).map_err(|e| e.to_string())?;
                    nombre += 1;
                }
            }
            archive.finish().map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(e) = resultat {
            let _ = std::fs::remove_file(&chemin);
            return Err(format!("Échec de l'écriture de l'archive : {e}"));
        }
        chemin
    } else {
        let sous_dossier = creer_dossier_libre(dossier, &base)?;
        for (nom, contenu) in noms.iter().zip(&contenus) {
            if let Some(contenu) = contenu {
                std::fs::write(sous_dossier.join(nom), contenu).map_err(|e| format!("Échec de l'écriture de « {nom} » : {e}"))?;
                nombre += 1;
            }
        }
        sous_dossier
    };
    Ok(ResultatExtraction { chemin, nombre, echecs })
}

/// Crée `dossier/nom`, ou `nom (2)`, `nom (3)`… si le nom est déjà pris.
fn creer_dossier_libre(dossier: &Path, nom: &str) -> Result<PathBuf, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::tests::dossier_test;
    use crate::pdf::{creer_depuis_images, sauvegarder, FormatPage};
    use image::{Rgb, Rgba, RgbaImage};

    /// PDF d'une page contenant un grand scan détaillé, intégré sans perte (donc lourd).
    fn pdf_lourd(dossier: &Path) -> (PathBuf, u64) {
        let png = dossier.join("scan.png");
        RgbImage::from_fn(2600, 1800, |x, y| Rgb([(x * 7 + y) as u8, (x ^ y) as u8, (x * y / 7) as u8]))
            .save(&png)
            .unwrap();
        let mut document = creer_depuis_images(&[png], FormatPage::A4, 0.0).unwrap();
        let resultat = sauvegarder(&mut document, dossier, "lourd").unwrap();
        (resultat.chemin, resultat.taille)
    }

    fn largeur_image(octets: &[u8]) -> i64 {
        let document = Document::load_mem(octets).unwrap();
        let image = document.objects.values().filter_map(|o| o.as_stream().ok()).find(|s| est_image(s)).unwrap();
        image.dict.get(b"Width").unwrap().as_i64().unwrap()
    }

    #[test]
    fn compresse_au_taux_avec_dimensions_maximales() {
        let dossier = dossier_test("pdf-compression-taux");
        let (source, taille) = pdf_lourd(&dossier);
        let resultat = compresser(&source, &dossier, 55, None, Some(1400)).unwrap();
        assert_eq!(resultat.images, 1);
        assert!(resultat.taille * 2 < taille, "{} devrait être bien < {taille}", resultat.taille);
        assert_eq!(largeur_image(&std::fs::read(resultat.chemin.unwrap()).unwrap()), 1400);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn atteint_une_taille_cible() {
        let dossier = dossier_test("pdf-compression-cible");
        let (source, taille) = pdf_lourd(&dossier);
        let cible = taille / 20;
        let resultat = compresser(&source, &dossier, 0, Some(cible), None).unwrap();
        assert!(resultat.chemin.is_some());
        assert!(resultat.taille <= cible, "{} devrait être ≤ {cible}", resultat.taille);
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn signale_une_taille_cible_impossible() {
        let dossier = dossier_test("pdf-compression-impossible");
        let (source, _) = pdf_lourd(&dossier);
        let erreur = compresser(&source, &dossier, 0, Some(500), None).unwrap_err();
        assert!(erreur.starts_with("Impossible d'atteindre"), "{erreur}");
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn decode_une_image_a_palette() {
        // 4 × 1 pixels, 1 bit par pixel : noir, blanc, blanc, noir ; palette RVB rouge / bleu
        let espace = Espace::Palette { base: Box::new(Espace::Rvb), couleurs: vec![255, 0, 0, 0, 0, 255] };
        let image = brut_vers_image(&[0b0110_0000], 4, 1, 1, &espace).unwrap().to_rgb8();
        let pixels: Vec<[u8; 3]> = image.pixels().map(|p| p.0).collect();
        assert_eq!(pixels, vec![[255, 0, 0], [0, 0, 255], [0, 0, 255], [255, 0, 0]]);
    }

    #[test]
    fn liste_et_extrait_les_images() {
        let dossier = dossier_test("pdf-extraction");
        let jpeg = dossier.join("photo.jpg");
        let png = dossier.join("logo.png");
        let photo = DynamicImage::ImageRgb8(RgbImage::from_pixel(300, 200, Rgb([200, 30, 30])));
        std::fs::write(&jpeg, encoder(&photo, FormatImage::Jpeg, 90).unwrap()).unwrap();
        RgbaImage::from_pixel(50, 80, Rgba([0, 0, 255, 128])).save(&png).unwrap();
        let mut document = creer_depuis_images(&[jpeg.clone(), png], FormatPage::A4, 0.0).unwrap();
        let pdf = sauvegarder(&mut document, &dossier, "album").unwrap().chemin;

        // Deux images (le masque de transparence du PNG n'est pas compté), dans l'ordre des pages
        let document = charger(&pdf).unwrap();
        let images = images_du_document(&document);
        assert_eq!(images.iter().map(|(_, p)| *p).collect::<Vec<_>>(), vec![Some(1), Some(2)]);
        let ids: Vec<String> = images.iter().map(|(id, _)| format!("{}-{}", id.0, id.1)).collect();

        // Archive ZIP : le JPEG est extrait tel quel, le PNG garde sa transparence
        let zip = extraire(&pdf, &ids, &dossier, true).unwrap();
        assert_eq!(zip.nombre, 2);
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip.chemin).unwrap()).unwrap();
        let mut noms: Vec<String> = archive.file_names().map(|n| n.unwrap().into_owned()).collect();
        noms.sort();
        assert_eq!(noms, vec!["page 001 - image 1.jpg", "page 002 - image 1.png"]);
        let mut extrait = Vec::new();
        std::io::Read::read_to_end(&mut archive.by_name("page 001 - image 1.jpg").unwrap(), &mut extrait).unwrap();
        assert_eq!(extrait, std::fs::read(&jpeg).unwrap());

        // Dossier : seulement l'image choisie
        let choix = extraire(&pdf, &ids[1..], &dossier, false).unwrap();
        assert_eq!(choix.nombre, 1);
        let relue = image::open(choix.chemin.join("page 002 - image 1.png")).unwrap();
        assert_eq!(relue.to_rgba8().get_pixel(0, 0)[3], 128);
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
