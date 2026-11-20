//! Fichiers reçus au lancement : « Convertir avec Allotrope » (clic droit dans l'explorateur),
//! glisser des fichiers sur l'icône, ou ligne de commande `allotrope photo.png video.mov`.
//!
//! Avec une sélection de plusieurs fichiers, l'explorateur lance Allotrope une fois par fichier.
//! Le plugin `single-instance` fait suivre ces lancements à la fenêtre déjà ouverte (événement
//! « fichiers-ouverts »), si bien que tous les fichiers arrivent dans la même file d'attente.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

/// Fichiers passés au premier lancement, en attendant que l'interface soit prête à les recevoir.
#[derive(Default)]
pub struct FichiersAuDemarrage(pub Mutex<Vec<PathBuf>>);

/// Chemins de fichiers existants parmi les arguments (le premier est le programme lui-même).
/// Les chemins relatifs sont résolus depuis `dossier_courant` ; les options (« -… ») sont ignorées.
pub fn chemins_depuis_arguments(arguments: &[String], dossier_courant: &Path) -> Vec<PathBuf> {
    arguments
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .map(|a| {
            let chemin = PathBuf::from(a);
            if chemin.is_absolute() { chemin } else { dossier_courant.join(chemin) }
        })
        .filter(|c| c.is_file())
        .collect()
}

/// Commande appelée par l'interface au démarrage : renvoie (une seule fois) les fichiers reçus.
#[tauri::command]
pub fn fichiers_au_demarrage(etat: State<'_, FichiersAuDemarrage>) -> Vec<PathBuf> {
    etat.0.lock().map(|mut f| std::mem::take(&mut *f)).unwrap_or_default()
}

/// Appelé quand Allotrope est relancé alors qu'il est déjà ouvert : transmet les fichiers à la
/// fenêtre existante et la ramène au premier plan.
pub fn nouvelle_instance(app: &AppHandle, arguments: Vec<String>, dossier_courant: String) {
    let chemins = chemins_depuis_arguments(&arguments, Path::new(&dossier_courant));
    if !chemins.is_empty() {
        let _ = app.emit("fichiers-ouverts", chemins);
    }
    if let Some(fenetre) = app.get_webview_window("main") {
        let _ = fenetre.unminimize();
        let _ = fenetre.show();
        let _ = fenetre.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_contextuel_a_jour() {
        // Chaque extension traitée par Allotrope doit figurer dans le menu de l'installateur
        let script = include_str!("../windows/menu-contextuel.nsh");
        let attendues = crate::conversion::EXTENSIONS_LISIBLES
            .iter()
            .chain(crate::media::EXTENSIONS_VIDEO)
            .chain(crate::media::EXTENSIONS_AUDIO)
            .chain(crate::bureautique::EXTENSIONS_DOCUMENT)
            .chain(&["pdf"])
            // « .ts » : surtout des fichiers TypeScript ; « .txt » : bien trop courant
            .filter(|e| !matches!(**e, "ts" | "txt"));
        for extension in attendues {
            assert!(
                script.contains(&format!("AllotropeAjouterMenu \"{extension}\"")),
                "extension absente du menu contextuel : {extension}"
            );
            assert!(script.contains(&format!("AllotropeRetirerMenu \"{extension}\"")), "{extension}");
        }
    }

    #[test]
    fn garde_les_fichiers_existants() {
        let dossier = std::env::temp_dir().join("allotrope-test-arguments");
        std::fs::create_dir_all(&dossier).unwrap();
        std::fs::write(dossier.join("photo.png"), b"x").unwrap();

        let arguments: Vec<String> = vec![
            "allotrope.exe".into(),
            "photo.png".into(),           // relatif au dossier courant
            "--une-option".into(),        // ignoré
            "absent.mov".into(),          // n'existe pas
            dossier.to_string_lossy().into(), // dossier : ignoré
        ];
        let chemins = chemins_depuis_arguments(&arguments, &dossier);
        assert_eq!(chemins, vec![dossier.join("photo.png")]);
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
