//! Conversion PDF → DOCX / ODT avec un traitement de texte installé sur l'ordinateur.
//!
//! Reconstituer un document modifiable (paragraphes, tableaux, images) à partir d'un PDF demande
//! un vrai moteur de traitement de texte. Deux moteurs possibles, tous deux locaux :
//! - **Microsoft Word** (Windows), piloté par automatisation COM via PowerShell : meilleur résultat ;
//! - **LibreOffice** (Windows, macOS, Linux), en mode « headless » (sans fenêtre).
//!
//! Le code propre à Windows (Word, masquage des fenêtres) est isolé dans le sous-module `systeme`.

use crate::conversion::{creer_fichier_libre, nom_sans_extension};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Délai maximal d'une conversion : au-delà, le moteur est sans doute bloqué (fenêtre invisible).
const DELAI: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatTexte {
    Docx,
    Odt,
}

impl FormatTexte {
    fn extension(self) -> &'static str {
        match self {
            FormatTexte::Docx => "docx",
            FormatTexte::Odt => "odt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Moteur {
    Word,
    Libreoffice,
}

/// Moteurs disponibles sur cet ordinateur.
#[derive(Debug, Serialize)]
pub struct Moteurs {
    pub word: bool,
    pub libreoffice: bool,
}

#[tauri::command]
pub async fn moteurs_bureautique() -> Moteurs {
    Moteurs { word: systeme::word_installe(), libreoffice: chemin_libreoffice().is_some() }
}

/// Résultat renvoyé à l'interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultatBureautique {
    pub chemin: PathBuf,
    pub taille: u64,
}

/// Convertit le PDF `source` en `dossier/nom.docx` (ou `.odt`) avec le moteur choisi.
#[tauri::command]
pub async fn pdf_vers_texte(
    source: PathBuf,
    dossier: PathBuf,
    format: FormatTexte,
    moteur: Moteur,
) -> Result<ResultatBureautique, String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&dossier).map_err(|e| format!("Impossible de créer le dossier de destination : {e}"))?;
        // Nom réservé tout de suite (jamais d'écrasement), puis rempli par le moteur
        let (sortie, fichier) = creer_fichier_libre(&dossier, &nom_sans_extension(&source), format.extension())
            .map_err(|e| format!("Impossible de créer le fichier de sortie : {e}"))?;
        drop(fichier);

        let resultat = match moteur {
            Moteur::Word => systeme::convertir_avec_word(&source, &sortie, format),
            Moteur::Libreoffice => match chemin_libreoffice() {
                Some(soffice) => convertir_avec_libreoffice(&soffice, &source, &sortie, format),
                None => Err("LibreOffice n'est pas installé".into()),
            },
        };
        let taille = std::fs::metadata(&sortie).map(|m| m.len()).unwrap_or(0);
        match resultat {
            Ok(()) if taille > 0 => Ok(ResultatBureautique { chemin: sortie, taille }),
            Ok(()) => {
                let _ = std::fs::remove_file(&sortie);
                Err("Le moteur n'a produit aucun document".into())
            }
            Err(e) => {
                let _ = std::fs::remove_file(&sortie);
                Err(e)
            }
        }
    })
    .await
    .map_err(|e| format!("Conversion interrompue : {e}"))?
}

// --- Exécution d'un programme externe avec délai ---

/// Résultat d'un programme : code de sortie (si terminé normalement) et messages d'erreur.
struct Sortie {
    succes: bool,
    erreurs: String,
}

/// Lance `commande` et attend sa fin, au plus `delai`. En cas de dépassement, le programme est
/// arrêté (puis `nettoyer` est appelé, par exemple pour fermer un Word resté bloqué).
fn executer(mut commande: Command, delai: Duration, nettoyer: impl FnOnce()) -> Result<Sortie, String> {
    systeme::sans_fenetre(&mut commande);
    let mut processus = commande
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Impossible de lancer le moteur : {e}"))?;
    // Lecture des erreurs dans un fil à part, pour que le programme ne se bloque jamais en écrivant
    let mut flux_erreurs = processus.stderr.take();
    let lecteur = std::thread::spawn(move || {
        let mut texte = String::new();
        if let Some(flux) = flux_erreurs.as_mut() {
            let _ = flux.read_to_string(&mut texte);
        }
        texte
    });

    let debut = Instant::now();
    let statut = loop {
        match processus.try_wait() {
            Ok(Some(statut)) => break statut,
            Ok(None) if debut.elapsed() > delai => {
                let _ = processus.kill();
                let _ = processus.wait();
                nettoyer();
                return Err(format!(
                    "Le moteur ne répond plus après {} minutes (une fenêtre invisible attend peut-être une réponse)",
                    delai.as_secs() / 60
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(200)),
            Err(e) => return Err(format!("Erreur pendant la conversion : {e}")),
        }
    };
    let erreurs = lecteur.join().unwrap_or_default();
    Ok(Sortie { succes: statut.success(), erreurs })
}

// --- LibreOffice ---

/// Emplacement de LibreOffice (`soffice`), s'il est installé à un endroit habituel.
pub fn chemin_libreoffice() -> Option<PathBuf> {
    let mut candidats: Vec<PathBuf> = Vec::new();
    for variable in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Some(dossier) = std::env::var_os(variable) {
            candidats.push(PathBuf::from(dossier).join("LibreOffice").join("program").join("soffice.exe"));
        }
    }
    candidats.extend(
        [
            "/Applications/LibreOffice.app/Contents/MacOS/soffice",
            "/usr/bin/soffice",
            "/usr/bin/libreoffice",
            "/usr/local/bin/soffice",
            "/snap/bin/libreoffice",
            "/opt/libreoffice/program/soffice",
        ]
        .map(PathBuf::from),
    );
    candidats.into_iter().find(|c| c.is_file())
}

/// Adresse « file:/// » d'un dossier, pour l'option `-env:UserInstallation` de LibreOffice.
pub fn url_fichier(chemin: &Path) -> String {
    let texte = chemin.to_string_lossy().replace('\\', "/");
    if texte.starts_with('/') { format!("file://{texte}") } else { format!("file:///{texte}") }
}

/// Arguments de LibreOffice pour convertir un PDF en document Writer modifiable.
pub fn arguments_libreoffice(source: &Path, dossier_sortie: &Path, profil: &Path, format: FormatTexte) -> Vec<String> {
    let filtre = match format {
        FormatTexte::Docx => "docx:MS Word 2007 XML",
        FormatTexte::Odt => "odt:writer8",
    };
    vec![
        "--headless".into(),
        "--norestore".into(),
        "--nolockcheck".into(),
        // Profil séparé : la conversion fonctionne même si LibreOffice est déjà ouvert
        format!("-env:UserInstallation={}", url_fichier(profil)),
        // Importe le PDF dans Writer (traitement de texte) plutôt que dans Draw (dessin)
        "--infilter=writer_pdf_import".into(),
        "--convert-to".into(),
        filtre.into(),
        "--outdir".into(),
        dossier_sortie.to_string_lossy().into_owned(),
        source.to_string_lossy().into_owned(),
    ]
}

fn convertir_avec_libreoffice(soffice: &Path, source: &Path, sortie: &Path, format: FormatTexte) -> Result<(), String> {
    // Dossier de travail temporaire : profil LibreOffice et fichier produit
    let travail = std::env::temp_dir().join(format!(
        "allotrope-libreoffice-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
    ));
    let dossier_sortie = travail.join("sortie");
    std::fs::create_dir_all(&dossier_sortie).map_err(|e| format!("Dossier temporaire impossible : {e}"))?;

    let mut commande = Command::new(soffice);
    commande.args(arguments_libreoffice(source, &dossier_sortie, &travail.join("profil"), format));
    let resultat = executer(commande, DELAI, || {}).and_then(|sortie_lo| {
        let produit = dossier_sortie.join(format!("{}.{}", nom_sans_extension(source), format.extension()));
        if !produit.is_file() {
            return Err(format!(
                "LibreOffice n'a pas pu convertir ce PDF{}",
                if sortie_lo.erreurs.trim().is_empty() { String::new() } else { format!(" : {}", sortie_lo.erreurs.trim()) }
            ));
        }
        std::fs::copy(&produit, sortie).map_err(|e| format!("Échec de l'écriture du document : {e}"))?;
        Ok(())
    });
    let _ = std::fs::remove_dir_all(&travail);
    resultat
}

// --- Microsoft Word (Windows uniquement) ---

#[cfg(windows)]
mod systeme {
    use super::{executer, FormatTexte, Sortie, DELAI};
    use std::os::windows::process::CommandExt;
    use std::path::Path;
    use std::process::Command;

    /// Empêche l'ouverture d'une console noire pour les programmes lancés.
    pub fn sans_fenetre(commande: &mut Command) {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        commande.creation_flags(CREATE_NO_WINDOW);
    }

    /// Vrai si Microsoft Word est installé (son automatisation COM est enregistrée).
    pub fn word_installe() -> bool {
        let mut commande = Command::new("reg");
        commande.args(["query", r"HKCR\Word.Application\CurVer"]);
        sans_fenetre(&mut commande);
        commande.output().map(|s| s.status.success()).unwrap_or(false)
    }

    /// Numéros de processus de Word en cours (pour fermer celui qu'Allotrope aurait laissé bloqué).
    fn processus_word() -> Vec<u32> {
        let mut commande = Command::new("tasklist");
        commande.args(["/FI", "IMAGENAME eq WINWORD.EXE", "/FO", "CSV", "/NH"]);
        sans_fenetre(&mut commande);
        let Ok(sortie) = commande.output() else { return Vec::new() };
        String::from_utf8_lossy(&sortie.stdout)
            .lines()
            .filter_map(|ligne| ligne.split(',').nth(1)?.trim_matches('"').parse().ok())
            .collect()
    }

    /// Script PowerShell : ouvre le PDF dans Word (conversion « PDF Reflow ») et l'enregistre.
    /// Les chemins passent par des variables d'environnement : aucun risque avec les caractères spéciaux.
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$word = New-Object -ComObject Word.Application
try {
  $word.Visible = $false
  $word.DisplayAlerts = 0
  # Ouvrir(fichier, confirmer la conversion : non, lecture seule : oui, fichiers récents : non)
  $document = $word.Documents.Open($env:ALLOTROPE_SOURCE, $false, $true, $false)
  $document.SaveAs2($env:ALLOTROPE_SORTIE, [int]$env:ALLOTROPE_FORMAT)
  $document.Close(0)
} catch {
  [Console]::Error.WriteLine($_.Exception.Message)
  exit 1
} finally {
  $word.Quit(0)
  [void][Runtime.InteropServices.Marshal]::ReleaseComObject($word)
}
"#;

    pub fn convertir_avec_word(source: &Path, sortie: &Path, format: FormatTexte) -> Result<(), String> {
        // Formats d'enregistrement de Word : 16 = .docx, 23 = .odt (OpenDocument)
        let code = match format {
            FormatTexte::Docx => "16",
            FormatTexte::Odt => "23",
        };
        let avant = processus_word();
        let mut commande = Command::new("powershell");
        commande
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", SCRIPT])
            .env("ALLOTROPE_SOURCE", source)
            .env("ALLOTROPE_SORTIE", sortie)
            .env("ALLOTROPE_FORMAT", code);
        let Sortie { succes, erreurs } = executer(commande, DELAI, || {
            // Délai dépassé : on ferme uniquement le Word lancé par Allotrope
            for pid in processus_word().into_iter().filter(|p| !avant.contains(p)) {
                let mut arret = Command::new("taskkill");
                arret.args(["/PID", &pid.to_string(), "/F"]);
                sans_fenetre(&mut arret);
                let _ = arret.output();
            }
        })?;
        if succes {
            Ok(())
        } else {
            Err(format!("Word n'a pas pu convertir ce PDF : {}", erreurs.trim()))
        }
    }
}

/// Hors Windows : pas de Word, et rien à masquer.
#[cfg(not(windows))]
mod systeme {
    use super::FormatTexte;
    use std::path::Path;
    use std::process::Command;

    pub fn sans_fenetre(_commande: &mut Command) {}

    pub fn word_installe() -> bool {
        false
    }

    pub fn convertir_avec_word(_source: &Path, _sortie: &Path, _format: FormatTexte) -> Result<(), String> {
        Err("Microsoft Word n'est utilisable que sous Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construit_les_arguments_de_libreoffice() {
        let args = arguments_libreoffice(
            Path::new("C:/docs/rapport.pdf"),
            Path::new("C:/tmp/sortie"),
            Path::new(r"C:\tmp\profil"),
            FormatTexte::Odt,
        );
        assert!(args.contains(&"--infilter=writer_pdf_import".to_string()));
        assert!(args.contains(&"odt:writer8".to_string()));
        assert!(args.contains(&"-env:UserInstallation=file:///C:/tmp/profil".to_string()));
        assert_eq!(args.last().unwrap(), "C:/docs/rapport.pdf");
    }

    #[test]
    fn adresse_d_un_dossier_unix() {
        assert_eq!(url_fichier(Path::new("/tmp/profil")), "file:///tmp/profil");
    }

    #[test]
    fn arrete_un_programme_trop_long() {
        let mut commande = if cfg!(windows) { Command::new("powershell") } else { Command::new("sleep") };
        if cfg!(windows) {
            commande.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"]);
        } else {
            commande.arg("30");
        }
        let mut nettoye = false;
        let debut = Instant::now();
        let resultat = executer(commande, Duration::from_secs(1), || nettoye = true);
        assert!(resultat.is_err());
        assert!(nettoye);
        assert!(debut.elapsed() < Duration::from_secs(10));
    }
}
