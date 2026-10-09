mod fichiers;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Sélecteur de fichiers natif (bouton « Parcourir les fichiers »)
        .plugin(tauri_plugin_dialog::init())
        // Commandes Rust accessibles depuis l'interface avec `invoke`
        .invoke_handler(tauri::generate_handler![fichiers::analyser_fichiers])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
