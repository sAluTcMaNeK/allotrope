<!-- Encart affiché quand un PDF a été créé. -->
<script lang="ts">
  import { Check, Folder } from "@lucide/svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { formaterTaille } from "../fichiers";
  import type { ResultatPdf } from "./pdf";

  interface Props {
    resultat: ResultatPdf;
    onrecommencer: () => void;
  }
  let { resultat, onrecommencer }: Props = $props();

  const nom = $derived(resultat.chemin.split(/[\\/]/).pop());
</script>

<div class="resultat" role="status">
  <span class="coche"><Check size={22} strokeWidth={3} /></span>
  <div class="infos">
    <div class="titre">PDF créé</div>
    <div class="meta">
      {nom} · {resultat.pages} page{resultat.pages > 1 ? "s" : ""} · {formaterTaille(resultat.taille)}
    </div>
  </div>
  <button class="secondaire" onclick={() => revealItemInDir(resultat.chemin)}>
    <Folder size={16} strokeWidth={2.6} />
    Afficher dans le dossier
  </button>
  <button class="secondaire" onclick={onrecommencer}>Recommencer</button>
</div>

<style>
  .resultat {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 14px;
    padding: 16px 18px;
    border-radius: 22px;
    border: 2px solid var(--menthe);
    background: var(--menthe-fond);
  }

  .coche {
    display: flex;
    color: var(--menthe);
  }

  .infos {
    flex: 1;
    min-width: 200px;
  }

  .titre {
    font-size: 15px;
    font-weight: 900;
    color: var(--menthe);
  }

  .meta {
    font-size: 13px;
    font-weight: 600;
    color: var(--givre);
    overflow-wrap: anywhere;
  }

  /* Bouton « Secondaire » de la charte */
  .secondaire {
    height: 44px;
    padding: 0 18px;
    border-radius: 999px;
    border: 2px solid var(--ligne-forte);
    background: var(--abysse);
    color: var(--givre);
    font-size: 14px;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .secondaire:hover {
    border-color: var(--givre);
  }
</style>
