mod compression;
mod conversion;
mod fichiers;
mod pdf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Sélecteur de fichiers et de dossiers natif
        .plugin(tauri_plugin_dialog::init())
        // Afficher un fichier converti dans l'explorateur
        .plugin(tauri_plugin_opener::init())
        // Commandes Rust accessibles depuis l'interface avec `invoke`
        .invoke_handler(tauri::generate_handler![
            fichiers::analyser_fichiers,
            conversion::convertir_image,
            compression::compresser_image,
            pdf::infos_pdf,
            pdf::lire_pdf,
            pdf::assembler_pdf,
            pdf::images_en_pdf,
            pdf::compresser_pdf
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
