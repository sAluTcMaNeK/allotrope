// Empêche l'ouverture d'une console noire en plus de la fenêtre, en version finale sous Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    allotrope_lib::run()
}
