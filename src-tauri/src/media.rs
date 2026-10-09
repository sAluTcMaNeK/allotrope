//! Module audio / vidéo : conversion et compression avec FFmpeg.
//!
//! FFmpeg est un programme séparé (« sidecar » Tauri, voir `bundle.externalBin` dans tauri.conf.json),
//! installé par `scripts/telecharger-ffmpeg.mjs`. C'est une build **LGPL** : pas de x264 ;
//! le H.264 est encodé avec OpenH264 (licence BSD de Cisco).
//!
//! Organisation :
//! - `lire_infos` lit les informations (durée, définition, codecs) qu'affiche `ffmpeg -i` ;
//! - `arguments` construit la ligne de commande FFmpeg pour une opération (fonction pure, testable) ;
//! - `traiter_media` lance FFmpeg et envoie la progression à l'interface en temps réel.

use crate::conversion::{creer_fichier_libre, extension_de, nom_sans_extension};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tauri::ipc::Channel;
use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

// --- Formats ---

/// Formats audio / vidéo que l'on sait produire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatMedia {
    Mp4,
    Webm,
    Mkv,
    Mov,
    Gif,
    Mp3,
    M4a,
    Ogg,
    Opus,
    Flac,
    Wav,
}

impl FormatMedia {
    pub fn extension(self) -> &'static str {
        match self {
            FormatMedia::Mp4 => "mp4",
            FormatMedia::Webm => "webm",
            FormatMedia::Mkv => "mkv",
            FormatMedia::Mov => "mov",
            FormatMedia::Gif => "gif",
            FormatMedia::Mp3 => "mp3",
            FormatMedia::M4a => "m4a",
            FormatMedia::Ogg => "ogg",
            FormatMedia::Opus => "opus",
            FormatMedia::Flac => "flac",
            FormatMedia::Wav => "wav",
        }
    }

    fn depuis_extension(extension: &str) -> Option<Self> {
        match extension {
            "mp4" | "m4v" => Some(FormatMedia::Mp4),
            "webm" => Some(FormatMedia::Webm),
            "mkv" => Some(FormatMedia::Mkv),
            "mov" => Some(FormatMedia::Mov),
            "gif" => Some(FormatMedia::Gif),
            "mp3" => Some(FormatMedia::Mp3),
            "m4a" | "aac" => Some(FormatMedia::M4a),
            "ogg" | "oga" => Some(FormatMedia::Ogg),
            "opus" => Some(FormatMedia::Opus),
            "flac" => Some(FormatMedia::Flac),
            "wav" => Some(FormatMedia::Wav),
            _ => None,
        }
    }

    /// Vrai pour les formats qui ne contiennent que du son.
    fn est_audio(self) -> bool {
        matches!(
            self,
            FormatMedia::Mp3 | FormatMedia::M4a | FormatMedia::Ogg | FormatMedia::Opus | FormatMedia::Flac | FormatMedia::Wav
        )
    }
}

/// Extensions reconnues comme vidéo (le GIF, souvent animé, est classé ici).
pub const EXTENSIONS_VIDEO: &[&str] = &[
    "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "3gp", "ogv", "gif",
];
/// Extensions reconnues comme audio.
pub const EXTENSIONS_AUDIO: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "m4a", "aac", "opus", "wma", "aiff", "aif", "ac3", "amr", "mka",
];

const FORMATS_VIDEO: [FormatMedia; 8] = [
    FormatMedia::Mp4,
    FormatMedia::Webm,
    FormatMedia::Mkv,
    FormatMedia::Mov,
    FormatMedia::Gif,
    FormatMedia::Mp3,
    FormatMedia::M4a,
    FormatMedia::Wav,
];
const FORMATS_AUDIO: [FormatMedia; 6] = [
    FormatMedia::Mp3,
    FormatMedia::M4a,
    FormatMedia::Ogg,
    FormatMedia::Opus,
    FormatMedia::Flac,
    FormatMedia::Wav,
];

/// Formats de sortie proposés pour une extension audio ou vidéo (vide si ce n'en est pas une).
pub fn formats_sortie_media(extension: &str) -> Vec<FormatMedia> {
    let extension = extension.to_lowercase();
    let source = FormatMedia::depuis_extension(&extension);
    let formats: &[FormatMedia] = if EXTENSIONS_VIDEO.contains(&extension.as_str()) {
        &FORMATS_VIDEO
    } else if EXTENSIONS_AUDIO.contains(&extension.as_str()) {
        &FORMATS_AUDIO
    } else {
        &[]
    };
    formats.iter().copied().filter(|f| Some(*f) != source).collect()
}

/// Format produit par la compression d'un fichier : son propre format quand c'est possible
/// (les vidéos AVI, WMV… deviennent des MP4). `None` pour l'audio sans perte (WAV, FLAC…),
/// qu'il vaut mieux convertir en MP3 / M4A / OPUS.
pub fn format_compression(extension: &str) -> Option<FormatMedia> {
    let extension = extension.to_lowercase();
    match FormatMedia::depuis_extension(&extension) {
        Some(FormatMedia::Flac | FormatMedia::Wav) => None,
        Some(format) => Some(format),
        None if EXTENSIONS_VIDEO.contains(&extension.as_str()) => Some(FormatMedia::Mp4),
        None => None,
    }
}

// --- Informations sur un fichier ---

/// Informations lues dans la sortie de `ffmpeg -i`.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoMedia {
    /// Durée en secondes.
    pub duree: Option<f64>,
    pub largeur: Option<u32>,
    pub hauteur: Option<u32>,
    pub images_par_seconde: Option<f64>,
    pub codec_video: Option<String>,
    pub codec_audio: Option<String>,
}

static DUREE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Duration: (\d+):(\d{2}):(\d{2}(?:\.\d+)?)").unwrap());
static FLUX_VIDEO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Stream #\S+.*?: Video: (\w+)").unwrap());
static FLUX_AUDIO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Stream #\S+.*?: Audio: (\w+)").unwrap());
static DIMENSIONS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(\d{2,5})x(\d{2,5})\b").unwrap());
static IMAGES_SECONDE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\d.]+) (?:fps|tbr)").unwrap());

/// Analyse la sortie d'erreur de `ffmpeg -i fichier` (FFmpeg y décrit le fichier d'entrée).
pub fn lire_infos(texte: &str) -> InfoMedia {
    let mut info = InfoMedia::default();
    for ligne in texte.lines() {
        if let Some(c) = DUREE.captures(ligne) {
            let h: f64 = c[1].parse().unwrap_or(0.0);
            let m: f64 = c[2].parse().unwrap_or(0.0);
            let s: f64 = c[3].parse().unwrap_or(0.0);
            info.duree = Some(h * 3600.0 + m * 60.0 + s);
        }
        // Les pochettes d'album sont des « vidéos » d'une image : on les ignore
        if let (Some(c), None, false) = (FLUX_VIDEO.captures(ligne), &info.codec_video, ligne.contains("attached pic")) {
            info.codec_video = Some(c[1].to_string());
            if let Some(d) = DIMENSIONS.captures(ligne) {
                info.largeur = d[1].parse().ok();
                info.hauteur = d[2].parse().ok();
            }
            info.images_par_seconde = IMAGES_SECONDE.captures(ligne).and_then(|i| i[1].parse().ok());
        }
        if let (Some(c), None) = (FLUX_AUDIO.captures(ligne), &info.codec_audio) {
            info.codec_audio = Some(c[1].to_string());
        }
    }
    info
}

/// Lance `ffmpeg -i` sur le fichier pour lire ses informations.
pub async fn infos(app: &AppHandle, chemin: &Path) -> Result<InfoMedia, String> {
    let sortie = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("FFmpeg introuvable : {e}"))?
        .args([OsString::from("-hide_banner"), "-nostdin".into(), "-i".into(), chemin.into()])
        .output()
        .await
        .map_err(|e| format!("Impossible de lancer FFmpeg : {e}"))?;
    let texte = String::from_utf8_lossy(&sortie.stderr);
    let info = lire_infos(&texte);
    if info.codec_video.is_none() && info.codec_audio.is_none() {
        return Err("Fichier audio ou vidéo illisible".into());
    }
    Ok(info)
}

/// Commande appelée depuis l'interface : `invoke("infos_media", { chemin })`.
#[tauri::command]
pub async fn infos_media(app: AppHandle, chemin: PathBuf) -> Result<InfoMedia, String> {
    infos(&app, &chemin).await
}

// --- Construction de la commande FFmpeg ---

/// Opération demandée par l'interface.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", rename_all_fields = "camelCase")]
pub enum OperationMedia {
    Convertir {
        format: FormatMedia,
    },
    /// `taux` de 5 (qualité maximale) à 95 (fichier le plus léger) ; `taille_cible` en octets ;
    /// `hauteur_max` en pixels (720 pour du 720p…).
    Compresser {
        taux: u8,
        taille_cible: Option<u64>,
        hauteur_max: Option<u32>,
    },
}

/// Format du fichier produit par l'opération.
pub fn format_produit(operation: OperationMedia, extension_source: &str) -> Option<FormatMedia> {
    match operation {
        OperationMedia::Convertir { format } => Some(format),
        OperationMedia::Compresser { .. } => format_compression(extension_source),
    }
}

/// Débit audio (bits/s) des vidéos converties ou compressées.
const DEBIT_AUDIO_VIDEO: u64 = 128_000;
const DEBIT_AUDIO_VIDEO_LEGER: u64 = 96_000;

/// Interpolation linéaire selon le taux de compression (5 → `debut`, 95 → `fin`).
fn selon_taux(taux: u8, debut: f64, fin: f64) -> f64 {
    let t = (taux.clamp(5, 95) as f64 - 5.0) / 90.0;
    debut + (fin - debut) * t
}

/// Hauteur de la vidéo produite : réduite à `hauteur_max` si besoin, et toujours paire (exigé par H.264).
fn hauteur_sortie(info: &InfoMedia, hauteur_max: Option<u32>) -> Option<u32> {
    let hauteur = info.hauteur?;
    Some(hauteur.min(hauteur_max.unwrap_or(u32::MAX)) / 2 * 2)
}

/// Filtre de mise à l'échelle : dimensions paires, hauteur limitée si demandé.
fn filtre_echelle(hauteur_max: Option<u32>) -> String {
    match hauteur_max {
        Some(h) => format!("scale=-2:'min(ih,{})':flags=lanczos", h / 2 * 2),
        None => "scale=trunc(iw/2)*2:trunc(ih/2)*2".into(),
    }
}

/// Débit vidéo estimé pour une qualité donnée, en « bits par pixel » (plus c'est bas, plus c'est léger).
fn debit_video(info: &InfoMedia, hauteur_max: Option<u32>, bits_par_pixel: f64) -> u64 {
    let (largeur, hauteur) = (info.largeur.unwrap_or(1280) as f64, info.hauteur.unwrap_or(720) as f64);
    let h = hauteur_sortie(info, hauteur_max).map(f64::from).unwrap_or(hauteur);
    let l = largeur * h / hauteur;
    let ips = info.images_par_seconde.unwrap_or(30.0).clamp(1.0, 60.0);
    ((l * h * ips * bits_par_pixel) as u64).clamp(150_000, 40_000_000)
}

/// Codec vidéo H.264 (OpenH264) à un débit donné.
fn h264(debit: u64) -> Vec<String> {
    let debit = debit.to_string();
    ["-c:v", "libopenh264", "-rc_mode", "bitrate", "-b:v", &debit, "-pix_fmt", "yuv420p"]
        .map(String::from)
        .to_vec()
}

/// Codec vidéo VP9 (WebM), soit à qualité constante (`crf`), soit à débit fixe.
fn vp9(crf: Option<u32>, debit: Option<u64>) -> Vec<String> {
    let mut a: Vec<String> = ["-c:v", "libvpx-vp9", "-row-mt", "1", "-deadline", "good", "-cpu-used", "4"]
        .map(String::from)
        .to_vec();
    match (crf, debit) {
        (Some(crf), _) => a.extend(["-crf".into(), crf.to_string(), "-b:v".into(), "0".into()]),
        (None, Some(debit)) => a.extend(["-b:v".into(), debit.to_string()]),
        _ => {}
    }
    a
}

/// Codec audio pour un format, à un débit donné en bits/s (`None` : réglage de qualité par défaut).
fn codec_audio(format: FormatMedia, debit: Option<u64>) -> Vec<String> {
    let avec_debit = |codec: &str, defaut: u64| {
        vec!["-c:a".into(), codec.into(), "-b:a".into(), debit.unwrap_or(defaut).to_string()]
    };
    match (format, debit) {
        // Sans débit imposé, MP3 et Vorbis utilisent leur mode « qualité variable », plus efficace
        (FormatMedia::Mp3, None) => ["-c:a", "libmp3lame", "-q:a", "2"].map(String::from).to_vec(),
        (FormatMedia::Mp3, _) => avec_debit("libmp3lame", 192_000),
        (FormatMedia::Ogg, None) => ["-c:a", "libvorbis", "-q:a", "6"].map(String::from).to_vec(),
        (FormatMedia::Ogg, _) => avec_debit("libvorbis", 160_000),
        (FormatMedia::Opus | FormatMedia::Webm, _) => avec_debit("libopus", 128_000),
        (FormatMedia::Flac, _) => ["-c:a", "flac", "-compression_level", "8"].map(String::from).to_vec(),
        (FormatMedia::Wav, _) => ["-c:a", "pcm_s16le"].map(String::from).to_vec(),
        // MP4, MOV, MKV, M4A : AAC
        _ => avec_debit("aac", 192_000),
    }
}

/// Filtre d'un GIF animé de bonne qualité : palette de couleurs calculée pour la vidéo.
fn filtre_gif(ips: u32, largeur_max: u32, couleurs: u32) -> String {
    format!(
        "fps={ips},scale='min({largeur_max},iw)':-2:flags=lanczos,split[a][b];\
         [a]palettegen=max_colors={couleurs}:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4"
    )
}

/// Construit les arguments de FFmpeg pour produire `sortie` à partir de `source`.
pub fn arguments(
    source: &Path,
    sortie: &Path,
    operation: OperationMedia,
    info: &InfoMedia,
    extension_source: &str,
) -> Result<Vec<OsString>, String> {
    let format = format_produit(operation, extension_source).ok_or(
        "Format sans perte (WAV, FLAC…) : convertissez-le plutôt en MP3, M4A ou OPUS pour l'alléger",
    )?;
    let a_video = info.codec_video.is_some();
    let a_audio = info.codec_audio.is_some();
    if format.est_audio() && !a_audio {
        return Err("Ce fichier ne contient pas de son".into());
    }
    if !format.est_audio() && !a_video {
        return Err("Ce fichier ne contient pas d'image vidéo".into());
    }

    let mut a: Vec<String> = Vec::new();
    // Flux gardés : la première vidéo et le premier son (le « ? » rend le son facultatif)
    let flux = |a: &mut Vec<String>| a.extend(["-map", "0:v:0", "-map", "0:a:0?"].map(String::from));

    match (operation, format) {
        // --- Son seul (conversion audio, ou extraction du son d'une vidéo) ---
        (OperationMedia::Convertir { .. }, f) if f.est_audio() => {
            a.extend(["-map", "0:a:0", "-vn"].map(String::from));
            a.extend(codec_audio(f, None));
        }
        (OperationMedia::Compresser { taux, taille_cible, .. }, f) if f.est_audio() => {
            let debit = match taille_cible {
                Some(cible) => {
                    let duree = info.duree.filter(|d| *d > 0.0).ok_or("Durée inconnue : taille cible impossible")?;
                    (cible as f64 * 8.0 / duree * 0.97) as u64
                }
                None => selon_taux(taux, 224_000.0, 48_000.0) as u64,
            };
            let minimum = if f == FormatMedia::Opus { 12_000 } else { 32_000 };
            if debit < minimum {
                return Err(format!("Taille cible trop petite pour {} de son", duree_lisible(info.duree)));
            }
            a.extend(["-map", "0:a:0", "-vn"].map(String::from));
            a.extend(codec_audio(f, Some(debit.min(320_000))));
        }

        // --- GIF animé ---
        (OperationMedia::Convertir { .. }, FormatMedia::Gif) => {
            a.extend(["-an".into(), "-vf".into(), filtre_gif(12, 640, 256), "-loop".into(), "0".into()]);
        }
        (OperationMedia::Compresser { taux, taille_cible, hauteur_max }, FormatMedia::Gif) => {
            if taille_cible.is_some() {
                return Err("La taille cible n'est pas disponible pour les GIF : utilisez le taux".into());
            }
            let largeur = hauteur_max.map(|h| h * 16 / 9).unwrap_or(selon_taux(taux, 640.0, 320.0) as u32);
            let filtre = filtre_gif(selon_taux(taux, 15.0, 8.0) as u32, largeur, selon_taux(taux, 256.0, 48.0) as u32);
            a.extend(["-an".into(), "-vf".into(), filtre, "-loop".into(), "0".into()]);
        }

        // --- Vidéo : conversion ---
        (OperationMedia::Convertir { .. }, FormatMedia::Mkv) if info.codec_video.as_deref() != Some("gif") => {
            // Le MKV accepte presque tous les codecs : simple changement de conteneur, sans perte et rapide
            a.extend(["-map", "0:v", "-map", "0:a?", "-map", "0:s?", "-c", "copy"].map(String::from));
        }
        (OperationMedia::Convertir { .. }, f @ (FormatMedia::Mp4 | FormatMedia::Mov))
            if info.codec_video.as_deref() == Some("h264")
                && matches!(info.codec_audio.as_deref(), None | Some("aac") | Some("mp3")) =>
        {
            // Déjà en H.264 / AAC (MOV d'appareil photo…) : copie directe, sans perte et quasi instantanée
            flux(&mut a);
            a.extend(["-c", "copy"].map(String::from));
            if f == FormatMedia::Mp4 {
                a.extend(["-movflags", "+faststart"].map(String::from));
            }
        }
        (OperationMedia::Convertir { .. }, FormatMedia::Webm) => {
            flux(&mut a);
            a.extend(["-vf".into(), filtre_echelle(None)]);
            a.extend(vp9(Some(32), None));
            a.extend(codec_audio(FormatMedia::Webm, None));
        }
        (OperationMedia::Convertir { .. }, f) => {
            // MP4, MOV, MKV : H.264 de bonne qualité + AAC
            flux(&mut a);
            a.extend(["-vf".into(), filtre_echelle(None)]);
            a.extend(h264(debit_video(info, None, 0.10)));
            a.extend(codec_audio(f, Some(DEBIT_AUDIO_VIDEO + 32_000)));
            if f != FormatMedia::Mkv {
                a.extend(["-movflags", "+faststart"].map(String::from));
            }
        }

        // --- Vidéo : compression ---
        (OperationMedia::Compresser { taux, taille_cible, hauteur_max }, f) => {
            flux(&mut a);
            a.extend(["-vf".into(), filtre_echelle(hauteur_max)]);
            let debit_audio = if a_audio {
                if taille_cible.is_some() || taux >= 60 { DEBIT_AUDIO_VIDEO_LEGER } else { DEBIT_AUDIO_VIDEO }
            } else {
                0
            };
            let debit_cible = match taille_cible {
                Some(cible) => {
                    let duree = info.duree.filter(|d| *d > 0.0).ok_or("Durée inconnue : taille cible impossible")?;
                    // 6 % de marge pour l'enveloppe du fichier (conteneur, index)
                    let total = (cible as f64 * 8.0 / duree * 0.94) as u64;
                    let video = total.saturating_sub(debit_audio);
                    if video < 80_000 {
                        return Err(format!(
                            "Taille cible trop petite pour une vidéo de {} : essayez au moins {}",
                            duree_lisible(info.duree),
                            taille_minimale(duree, debit_audio)
                        ));
                    }
                    Some(video)
                }
                None => None,
            };
            if f == FormatMedia::Webm {
                let crf = debit_cible.is_none().then(|| selon_taux(taux, 24.0, 50.0) as u32);
                a.extend(vp9(crf, debit_cible));
            } else {
                let debit = debit_cible.unwrap_or_else(|| debit_video(info, hauteur_max, selon_taux(taux, 0.12, 0.015)));
                a.extend(h264(debit));
            }
            if a_audio {
                a.extend(codec_audio(f, Some(debit_audio)));
            }
            if matches!(f, FormatMedia::Mp4 | FormatMedia::Mov) {
                a.extend(["-movflags", "+faststart"].map(String::from));
            }
        }
    }

    let mut arguments: Vec<OsString> = ["-hide_banner", "-nostdin", "-y", "-i"].map(OsString::from).to_vec();
    arguments.push(source.into());
    arguments.extend(a.into_iter().map(OsString::from));
    // Progression lisible par Allotrope sur la sortie standard
    arguments.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
    arguments.push(sortie.into());
    Ok(arguments)
}

/// « 3 min 12 s », pour les messages.
fn duree_lisible(duree: Option<f64>) -> String {
    let s = duree.unwrap_or(0.0).round() as u64;
    match (s / 60, s % 60) {
        (0, s) => format!("{s} s"),
        (m, 0) => format!("{m} min"),
        (m, s) => format!("{m} min {s} s"),
    }
}

/// Plus petite taille raisonnable d'une vidéo de `duree` secondes (environ 100 kbit/s d'image).
fn taille_minimale(duree: f64, debit_audio: u64) -> String {
    let octets = (100_000 + debit_audio) as f64 * duree / 8.0 / 0.94;
    if octets >= 1024.0 * 1024.0 {
        format!("{:.0} Mo", (octets / (1024.0 * 1024.0)).ceil())
    } else {
        format!("{:.0} Ko", (octets / 1024.0).ceil())
    }
}

// --- Exécution ---

/// Progression envoyée à l'interface pendant le traitement.
#[derive(Debug, Clone, Serialize)]
pub struct Progression {
    /// De 0 à 100.
    pub pourcentage: f64,
}

/// Résultat renvoyé à l'interface. `chemin` est `None` si la compression n'a rien fait gagner.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatMedia {
    pub chemin: Option<PathBuf>,
    pub taille: u64,
}

/// FFmpeg en cours d'exécution, par fichier source (pour pouvoir l'arrêter).
static PROCESSUS: LazyLock<Mutex<HashMap<PathBuf, CommandChild>>> = LazyLock::new(Default::default);
/// Traitements annulés par l'utilisateur.
static ANNULES: LazyLock<Mutex<HashSet<PathBuf>>> = LazyLock::new(Default::default);

/// Commande appelée depuis l'interface :
/// `invoke("traiter_media", { source, dossier, operation, progression })`.
/// `progression` est un canal Tauri : chaque message fait avancer la barre de progression.
#[tauri::command]
pub async fn traiter_media(
    app: AppHandle,
    source: PathBuf,
    dossier: PathBuf,
    operation: OperationMedia,
    progression: Channel<Progression>,
) -> Result<ResultatMedia, String> {
    let info = infos(&app, &source).await?;
    let extension = extension_de(&source);
    let format = format_produit(operation, &extension)
        .ok_or("Format sans perte (WAV, FLAC…) : convertissez-le plutôt en MP3, M4A ou OPUS pour l'alléger")?;

    // Nom du fichier produit, réservé tout de suite (jamais d'écrasement), puis rempli par FFmpeg (« -y »)
    let suffixe = if matches!(operation, OperationMedia::Compresser { .. }) { " (compressé)" } else { "" };
    std::fs::create_dir_all(&dossier).map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
    let (sortie, fichier) =
        creer_fichier_libre(&dossier, &format!("{}{suffixe}", nom_sans_extension(&source)), format.extension())
            .map_err(|e| format!("Impossible de créer le fichier de sortie : {e}"))?;
    drop(fichier);

    let resultat = match arguments(&source, &sortie, operation, &info, &extension) {
        Ok(args) => executer(&app, &source, args, info.duree, &progression).await,
        Err(e) => Err(e),
    };
    if let Err(e) = resultat {
        let _ = std::fs::remove_file(&sortie);
        return Err(e);
    }

    let taille = std::fs::metadata(&sortie).map(|m| m.len()).unwrap_or(0);
    // Compression au taux sans réduction de définition : si le fichier n'a pas maigri, on ne le garde pas
    if let OperationMedia::Compresser { taille_cible: None, hauteur_max, .. } = operation {
        let reduite = matches!((hauteur_max, info.hauteur), (Some(max), Some(h)) if h > max);
        let taille_source = std::fs::metadata(&source).map(|m| m.len()).unwrap_or(u64::MAX);
        if !reduite && taille >= taille_source {
            let _ = std::fs::remove_file(&sortie);
            return Ok(ResultatMedia { chemin: None, taille: taille_source });
        }
    }
    let _ = progression.send(Progression { pourcentage: 100.0 });
    Ok(ResultatMedia { chemin: Some(sortie), taille })
}

/// Arrête le traitement en cours de `source` (bouton « Annuler »).
#[tauri::command]
pub fn annuler_media(source: PathBuf) {
    ANNULES.lock().unwrap().insert(source.clone());
    if let Some(processus) = PROCESSUS.lock().unwrap().remove(&source) {
        let _ = processus.kill();
    }
}

/// Lance FFmpeg et suit sa progression jusqu'à la fin.
async fn executer(
    app: &AppHandle,
    cle: &Path,
    arguments: Vec<OsString>,
    duree: Option<f64>,
    progression: &Channel<Progression>,
) -> Result<(), String> {
    ANNULES.lock().unwrap().remove(cle);
    let (mut evenements, processus) = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("FFmpeg introuvable : {e}"))?
        .args(arguments)
        .spawn()
        .map_err(|e| format!("Impossible de lancer FFmpeg : {e}"))?;
    PROCESSUS.lock().unwrap().insert(cle.to_path_buf(), processus);

    // Les dernières lignes d'erreur de FFmpeg, pour expliquer un échec
    let mut journal: VecDeque<String> = VecDeque::with_capacity(12);
    let mut code = None;
    while let Some(evenement) = evenements.recv().await {
        match evenement {
            CommandEvent::Stdout(ligne) => {
                if let Some(pourcentage) = lire_progression(&String::from_utf8_lossy(&ligne), duree) {
                    let _ = progression.send(Progression { pourcentage });
                }
            }
            CommandEvent::Stderr(ligne) => {
                let ligne = String::from_utf8_lossy(&ligne).trim().to_string();
                if !ligne.is_empty() {
                    if journal.len() == 12 {
                        journal.pop_front();
                    }
                    journal.push_back(ligne);
                }
            }
            CommandEvent::Error(e) => journal.push_back(e),
            CommandEvent::Terminated(fin) => code = fin.code,
            _ => {}
        }
    }

    PROCESSUS.lock().unwrap().remove(cle);
    if ANNULES.lock().unwrap().remove(cle) {
        return Err("Annulé".into());
    }
    match code {
        Some(0) => Ok(()),
        _ => Err(format!(
            "FFmpeg a échoué : {}",
            journal.back().cloned().unwrap_or_else(|| "erreur inconnue".into())
        )),
    }
}

/// Lit une ligne de progression de FFmpeg (`out_time_us=…`) et la convertit en pourcentage.
pub fn lire_progression(ligne: &str, duree: Option<f64>) -> Option<f64> {
    let microsecondes: f64 = ligne.trim().strip_prefix("out_time_us=")?.parse().ok()?;
    let duree = duree.filter(|d| *d > 0.0)?;
    Some((microsecondes / 1_000_000.0 / duree * 100.0).clamp(0.0, 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::tests::dossier_test;
    use std::process::Command;

    /// FFmpeg installé par `scripts/telecharger-ffmpeg.mjs`. Les tests qui en ont besoin sont
    /// ignorés (avec un message) s'il est absent.
    fn ffmpeg() -> Option<PathBuf> {
        let nom = if cfg!(windows) { "ffmpeg-x86_64-pc-windows-msvc.exe" } else { "ffmpeg" };
        let chemin = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries").join(nom);
        if chemin.exists() {
            Some(chemin)
        } else {
            eprintln!("FFmpeg absent : lancez `node scripts/telecharger-ffmpeg.mjs` (test ignoré)");
            None
        }
    }

    /// Lance FFmpeg directement (sans Tauri) et renvoie sa sortie d'erreur.
    fn lancer(ffmpeg: &Path, arguments: &[OsString]) -> (bool, String) {
        let sortie = Command::new(ffmpeg).args(arguments).output().unwrap();
        (sortie.status.success(), String::from_utf8_lossy(&sortie.stderr).into_owned())
    }

    fn infos_directes(ffmpeg: &Path, chemin: &Path) -> InfoMedia {
        let sortie = Command::new(ffmpeg).args(["-hide_banner", "-i"]).arg(chemin).output().unwrap();
        lire_infos(&String::from_utf8_lossy(&sortie.stderr))
    }

    /// Crée une vidéo de test de 2 s (mire + son à 440 Hz), générée par FFmpeg lui-même.
    fn video_test(ffmpeg: &Path, dossier: &Path) -> PathBuf {
        let chemin = dossier.join("mire.mov");
        let (ok, journal) = lancer(
            ffmpeg,
            &[
                "-hide_banner", "-y", "-f", "lavfi", "-i", "testsrc2=duration=2:size=640x360:rate=25", "-f", "lavfi",
                "-i", "sine=frequency=440:duration=2", "-c:v", "libopenh264", "-b:v", "2M", "-c:a", "aac",
            ]
            .map(OsString::from)
            .into_iter()
            .chain([chemin.clone().into()])
            .collect::<Vec<_>>(),
        );
        assert!(ok, "{journal}");
        chemin
    }

    #[test]
    fn lit_les_informations_affichees_par_ffmpeg() {
        let texte = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'video.mov':
  Duration: 00:01:23.45, start: 0.000000, bitrate: 5000 kb/s
  Stream #0:0[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(tv, bt709), 1920x1080 [SAR 1:1 DAR 16:9], 4900 kb/s, 29.97 fps, 29.97 tbr
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 128 kb/s";
        let info = lire_infos(texte);
        assert!((info.duree.unwrap() - 83.45).abs() < 1e-6);
        assert_eq!((info.largeur, info.hauteur), (Some(1920), Some(1080)));
        assert_eq!(info.images_par_seconde, Some(29.97));
        assert_eq!(info.codec_video.as_deref(), Some("h264"));
        assert_eq!(info.codec_audio.as_deref(), Some("aac"));

        // Une pochette d'album n'est pas une vidéo
        let mp3 = "  Duration: 00:03:00.00, start: 0.025057, bitrate: 320 kb/s
  Stream #0:0: Audio: mp3 (mp3float), 44100 Hz, stereo, fltp, 320 kb/s
  Stream #0:1: Video: mjpeg (Baseline), yuvj420p(pc), 500x500, 90k tbr, 90k tbn (attached pic)";
        let info = lire_infos(mp3);
        assert!(info.codec_video.is_none());
        assert_eq!(info.codec_audio.as_deref(), Some("mp3"));
    }

    #[test]
    fn lit_la_progression() {
        assert_eq!(lire_progression("out_time_us=5000000", Some(10.0)), Some(50.0));
        assert_eq!(lire_progression("out_time_us=N/A", Some(10.0)), None);
        assert_eq!(lire_progression("progress=continue", Some(10.0)), None);
    }

    #[test]
    fn propose_les_bons_formats() {
        assert!(formats_sortie_media("MOV").contains(&FormatMedia::Mp4));
        assert!(!formats_sortie_media("mp4").contains(&FormatMedia::Mp4));
        assert!(formats_sortie_media("mp4").contains(&FormatMedia::Mp3));
        assert_eq!(formats_sortie_media("wav").len(), FORMATS_AUDIO.len() - 1);
        assert!(formats_sortie_media("png").is_empty());
        assert_eq!(format_compression("avi"), Some(FormatMedia::Mp4));
        assert_eq!(format_compression("flac"), None);
    }

    #[test]
    fn refuse_une_taille_cible_impossible() {
        let info = InfoMedia {
            duree: Some(600.0),
            largeur: Some(1920),
            hauteur: Some(1080),
            images_par_seconde: Some(30.0),
            codec_video: Some("h264".into()),
            codec_audio: Some("aac".into()),
        };
        let operation = OperationMedia::Compresser { taux: 50, taille_cible: Some(1024 * 1024), hauteur_max: None };
        let erreur = arguments(Path::new("a.mp4"), Path::new("b.mp4"), operation, &info, "mp4").unwrap_err();
        assert!(erreur.starts_with("Taille cible trop petite"), "{erreur}");
    }

    #[test]
    fn convertit_vers_chaque_format() {
        let Some(ffmpeg) = ffmpeg() else { return };
        let dossier = dossier_test("media-formats");
        let source = video_test(&ffmpeg, &dossier);
        let info = infos_directes(&ffmpeg, &source);
        assert_eq!(info.codec_audio.as_deref(), Some("aac"));

        for format in FORMATS_VIDEO.into_iter().filter(|f| *f != FormatMedia::Mov) {
            let sortie = dossier.join(format!("sortie.{}", format.extension()));
            let args = arguments(&source, &sortie, OperationMedia::Convertir { format }, &info, "mov").unwrap();
            let (ok, journal) = lancer(&ffmpeg, &args);
            assert!(ok, "{format:?} : {journal}");
            let produit = infos_directes(&ffmpeg, &sortie);
            assert!(produit.duree.unwrap_or(0.0) > 1.5, "{format:?} : durée {:?}", produit.duree);
            assert_eq!(produit.codec_video.is_some(), !format.est_audio(), "{format:?}");
        }
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn compresse_une_video_sous_une_taille_cible() {
        let Some(ffmpeg) = ffmpeg() else { return };
        let dossier = dossier_test("media-compression");
        let source = video_test(&ffmpeg, &dossier);
        let info = infos_directes(&ffmpeg, &source);

        let cible = 150 * 1024;
        let sortie = dossier.join("compressee.mov");
        let operation = OperationMedia::Compresser { taux: 50, taille_cible: Some(cible), hauteur_max: Some(240) };
        let args = arguments(&source, &sortie, operation, &info, "mov").unwrap();
        let (ok, journal) = lancer(&ffmpeg, &args);
        assert!(ok, "{journal}");

        // Le réglage par débit vise la cible ; on tolère 15 % d'écart sur une vidéo aussi courte
        let taille = std::fs::metadata(&sortie).unwrap().len();
        assert!(taille <= cible * 115 / 100, "{taille} octets pour une cible de {cible}");
        assert_eq!(infos_directes(&ffmpeg, &sortie).hauteur, Some(240));
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn compresse_un_fichier_audio() {
        let Some(ffmpeg) = ffmpeg() else { return };
        let dossier = dossier_test("media-audio");
        let source = video_test(&ffmpeg, &dossier);

        // D'abord extraire le son en MP3 à 320 kbit/s (comme un fichier de bonne qualité), puis le compresser fortement
        let mp3 = dossier.join("son.mp3");
        let extraction: Vec<OsString> = ["-hide_banner", "-y", "-i"]
            .map(OsString::from)
            .into_iter()
            .chain([source.clone().into()])
            .chain(["-vn", "-c:a", "libmp3lame", "-b:a", "320k"].map(OsString::from))
            .chain([mp3.clone().into()])
            .collect();
        assert!(lancer(&ffmpeg, &extraction).0);
        let info_mp3 = infos_directes(&ffmpeg, &mp3);
        let leger = dossier.join("leger.mp3");
        let operation = OperationMedia::Compresser { taux: 95, taille_cible: None, hauteur_max: None };
        let args = arguments(&mp3, &leger, operation, &info_mp3, "mp3").unwrap();
        let (ok, journal) = lancer(&ffmpeg, &args);
        assert!(ok, "{journal}");
        assert!(std::fs::metadata(&leger).unwrap().len() < std::fs::metadata(&mp3).unwrap().len());
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
