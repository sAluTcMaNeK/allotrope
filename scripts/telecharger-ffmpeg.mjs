// Télécharge FFmpeg (build LGPL, sans x264 ni composant GPL) et l'installe comme « sidecar » Tauri :
// src-tauri/binaries/ffmpeg-<cible>[.exe]. Ne fait rien si le fichier est déjà là.
// Lancé automatiquement avant `npm run dev` et `npm run build` (scripts « predev » / « prebuild »).
//
// Les archives sont cherchées d'abord dans la publication « ffmpeg-9.0.2 » du dépôt Allotrope
// (copie permanente, voir .github/workflows/ffmpeg.yml), puis à leur source d'origine
// (https://github.com/BtbN/FFmpeg-Builds, dont les publications sont supprimées après quelques
// semaines). L'empreinte SHA-256 de chaque archive est figée ici et toujours vérifiée.
import { createHash } from "node:crypto";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const MIROIR = "https://github.com/sAluTcMaNeK/allotrope/releases/download/ffmpeg-9.0.2";
const BTBN = "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-11-20-12-40";
const NOM_BTBN = "ffmpeg-n9.0.2-61-gd59078794d";

/** Archive à télécharger pour chaque système (clé : `${process.platform}-${process.arch}`). */
const BUILDS = {
  "win32-x64": {
    cible: "x86_64-pc-windows-msvc",
    archive: `${NOM_BTBN}-win64-lgpl-9.0.zip`,
    sources: [MIROIR, BTBN],
    sha256: "814c62c35ca7e2b15d85b0a2a5b79bbf21783176520154f26a6df1e9df72761e",
    dossier: `${NOM_BTBN}-win64-lgpl-9.0`,
    executable: "bin/ffmpeg.exe",
    licence: "LICENSE.txt",
  },
  "linux-x64": {
    cible: "x86_64-unknown-linux-gnu",
    archive: `${NOM_BTBN}-linux64-lgpl-9.0.tar.xz`,
    sources: [MIROIR, BTBN],
    sha256: "56e74f65818998c92a15f743e0e65d9090312a5b2ff3ec8758bf911e770b99a3",
    dossier: `${NOM_BTBN}-linux64-lgpl-9.0`,
    executable: "bin/ffmpeg",
    licence: "LICENSE.txt",
  },
  // BtbN ne publie pas de build macOS : elle est compilée par .github/workflows/ffmpeg.yml.
  // Après la première exécution de ce workflow, recopier ici l'empreinte qu'il affiche.
  "darwin-arm64": {
    cible: "aarch64-apple-darwin",
    archive: "ffmpeg-9.0.2-macos-arm64-lgpl.tar.xz",
    sources: [MIROIR],
    sha256: null,
    dossier: "ffmpeg-9.0.2-macos-arm64-lgpl",
    executable: "bin/ffmpeg",
    licence: "LICENSE.txt",
  },
};

const plateforme = `${process.platform}-${process.arch}`;
const build = BUILDS[plateforme];
if (!build) {
  console.warn(`FFmpeg : pas de build prévue pour ${plateforme} ; l'audio et la vidéo ne fonctionneront pas.`);
  process.exit(0);
}

const dossier = "src-tauri/binaries";
const windows = process.platform === "win32";
const destination = join(dossier, `ffmpeg-${build.cible}${windows ? ".exe" : ""}`);
if (existsSync(destination)) process.exit(0);

if (!build.sha256) {
  throw new Error(
    "FFmpeg pour macOS : empreinte SHA-256 non renseignée dans scripts/telecharger-ffmpeg.mjs. " +
      "Lancez d'abord le workflow GitHub « FFmpeg » (.github/workflows/ffmpeg.yml), puis recopiez l'empreinte affichée.",
  );
}

/** Télécharge l'archive depuis la première source qui répond, et vérifie son empreinte. */
async function telecharger() {
  for (const source of build.sources) {
    const url = `${source}/${build.archive}`;
    try {
      const reponse = await fetch(url);
      if (!reponse.ok) {
        console.warn(`FFmpeg : ${url} indisponible (${reponse.status}), source suivante…`);
        continue;
      }
      const archive = Buffer.from(await reponse.arrayBuffer());
      const empreinte = createHash("sha256").update(archive).digest("hex");
      if (empreinte !== build.sha256) {
        throw new Error(`Empreinte SHA-256 inattendue (${empreinte}) pour ${url} : fichier corrompu ou modifié.`);
      }
      return archive;
    } catch (erreur) {
      if (String(erreur).includes("Empreinte")) throw erreur;
      console.warn(`FFmpeg : ${url} injoignable (${erreur}), source suivante…`);
    }
  }
  throw new Error("FFmpeg : aucune source n'a pu fournir l'archive. Vérifiez la connexion internet.");
}

console.log(`FFmpeg 9.0.2 (LGPL) : téléchargement (~150 Mo, une seule fois)…`);
const archive = await telecharger();

const temporaire = mkdtempSync(join(tmpdir(), "allotrope-ffmpeg-"));
try {
  const chemin = join(temporaire, build.archive);
  writeFileSync(chemin, archive);
  // tar sait lire les .zip sous Windows 10/11 (bsdtar) et les .tar.xz ailleurs.
  // Sous Windows, on prend celui du système : celui de Git Bash ne comprend pas les chemins « C:… ».
  const tar = windows ? join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe") : "tar";
  execFileSync(tar, ["-xf", chemin, "-C", temporaire]);
  const racine = join(temporaire, build.dossier);
  mkdirSync(dossier, { recursive: true });
  copyFileSync(join(racine, build.executable), destination);
  if (!windows) chmodSync(destination, 0o755);
  // Texte de la licence LGPL, à livrer avec l'application
  copyFileSync(join(racine, build.licence), join(dossier, "FFMPEG-LICENSE.txt"));
  console.log(`FFmpeg installé : ${destination}`);
} finally {
  rmSync(temporaire, { recursive: true, force: true });
}

// Vérification : la build ne doit contenir aucun composant GPL
const configuration = execFileSync(destination, ["-hide_banner", "-buildconf"]).toString();
if (configuration.includes("--enable-gpl")) {
  rmSync(destination);
  throw new Error("Cette build de FFmpeg est sous GPL : installation annulée.");
}
