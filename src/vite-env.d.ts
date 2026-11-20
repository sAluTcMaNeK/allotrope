/// <reference types="svelte" />
/// <reference types="vite/client" />

interface Window {
  /** Déclaré par le cœur Rust : vrai pour la version portable (voir src/lib/edition.ts). */
  __ALLOTROPE_PORTABLE__?: boolean;
}
