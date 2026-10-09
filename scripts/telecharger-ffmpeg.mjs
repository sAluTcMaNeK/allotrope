// Télécharge FFmpeg (build LGPL, sans x264 ni composant GPL) et l'installe comme « sidecar » Tauri :
// src-tauri/binaries/ffmpeg-<cible>.exe. Ne fait rien si le fichier est déjà là.
// Lancé automatiquement avant `npm run dev` et `npm run build` (scripts « predev » / « prebuild »).
//
// Source : https://github.com/BtbN/FFmpeg-Builds (version figée + empreinte SHA-256 vérifiée).
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const VERSION = "n9.0.2-23-g27b46f0fbc";
const BUILDS = {
  win32: {
    cible: "x86_64-pc-windows-msvc",
    url: `https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-08-13-05/ffmpeg-${VERSION}-win64-lgpl-9.0.zip`,
    sha256: "74823968af824fddced214fbfa3aa47c73c0cb15fd225db8d73ee3212e9757f7",
    executable: "ffmpeg.exe",
  },
};

const build = BUILDS[process.platform];
if (!build) {
  console.warn(`FFmpeg : pas encore de build prévue pour ${process.platform} (Windows uniquement pour l'instant).`);
  process.exit(0);
}

const dossier = "src-tauri/binaries";
const destination = join(dossier, `ffmpeg-${build.cible}${process.platform === "win32" ? ".exe" : ""}`);
if (existsSync(destination)) process.exit(0);

console.log(`FFmpeg ${VERSION} (LGPL) : téléchargement (~165 Mo, une seule fois)…`);
const reponse = await fetch(build.url);
if (!reponse.ok) throw new Error(`Téléchargement impossible : ${reponse.status} ${reponse.statusText}`);
const archive = Buffer.from(await reponse.arrayBuffer());

const empreinte = createHash("sha256").update(archive).digest("hex");
if (empreinte !== build.sha256) {
  throw new Error(`Empreinte SHA-256 inattendue (${empreinte}) : fichier corrompu ou modifié, installation annulée.`);
}

// Extraction dans un dossier temporaire neuf (tar sait lire les .zip sous Windows 10/11)
const temporaire = mkdtempSync(join(tmpdir(), "allotrope-ffmpeg-"));
try {
  const zip = join(temporaire, "ffmpeg.zip");
  writeFileSync(zip, archive);
  // tar de Windows (bsdtar) : celui de Git Bash, s'il passe avant dans le PATH, ne lit pas les chemins « C:… »
  const tar = process.platform === "win32" ? join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe") : "tar";
  execFileSync(tar, ["-xf", zip, "-C", temporaire]);
  const racine = join(temporaire, `ffmpeg-${VERSION}-win64-lgpl-9.0`);
  mkdirSync(dossier, { recursive: true });
  copyFileSync(join(racine, "bin", build.executable), destination);
  // Texte de la licence LGPL, à livrer avec l'application
  copyFileSync(join(racine, "LICENSE.txt"), join(dossier, "FFMPEG-LICENSE.txt"));
  console.log(`FFmpeg installé : ${destination}`);
} finally {
  rmSync(temporaire, { recursive: true, force: true });
}

// Vérification rapide : la build ne doit contenir aucun composant GPL
const configuration = execFileSync(destination, ["-hide_banner", "-buildconf"]).toString();
if (configuration.includes("--enable-gpl")) {
  rmSync(destination);
  throw new Error("Cette build de FFmpeg est sous GPL : installation annulée.");
}
