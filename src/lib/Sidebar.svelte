<!-- Barre latérale : accueil, pages par type de fichiers, outils PDF et rappel « 100 % local ». -->
<script lang="ts">
  import { FileStack, House, ShieldCheck } from "@lucide/svelte";
  import { LISTE_FAMILLES } from "./fichiers";
  import type { Page } from "./navigation";

  interface Props {
    page: Page;
    /** Nombre de fichiers en file d'attente pour chaque page. */
    compteurs: Record<string, number>;
    onnaviguer: (page: Page) => void;
  }
  let { page, compteurs, onnaviguer }: Props = $props();
</script>

<nav class="laterale" aria-label="Navigation principale">
  <div class="groupe">
    <button
      class="element"
      class:actif={page === "accueil"}
      aria-current={page === "accueil" ? "page" : undefined}
      onclick={() => onnaviguer("accueil")}
    >
      <House size={18} strokeWidth={2.4} />
      <span class="texte">Accueil</span>
      {#if compteurs.accueil}<span class="compteur">{compteurs.accueil}</span>{/if}
    </button>
  </div>

  <div class="groupe">
    <div class="sur-titre">Types de fichiers</div>
    {#each LISTE_FAMILLES as [id, famille] (id)}
      <button
        class="element"
        class:actif={page === id}
        aria-current={page === id ? "page" : undefined}
        onclick={() => onnaviguer(id)}
      >
        <span class="pastille" style:background={famille.couleur}></span>
        <span class="texte">{famille.libelle}</span>
        {#if compteurs[id]}<span class="compteur">{compteurs[id]}</span>{/if}
      </button>
    {/each}
  </div>

  <div class="groupe">
    <div class="sur-titre">Outils</div>
    <button
      class="element"
      class:actif={page === "pdf"}
      aria-current={page === "pdf" ? "page" : undefined}
      onclick={() => onnaviguer("pdf")}
    >
      <FileStack size={18} strokeWidth={2.4} />
      <span class="texte">Outils PDF</span>
    </button>
  </div>

  <div class="local">
    <span class="bouclier"><ShieldCheck size={22} strokeWidth={2.4} /></span>
    <div>
      <div class="local-titre">100 % local</div>
      <div class="local-texte">Aucun fichier ne quitte votre ordinateur.</div>
    </div>
  </div>
</nav>

<style>
  .laterale {
    width: 240px;
    flex: none;
    padding: 24px 16px;
    border-right: 2px solid var(--ligne);
    display: flex;
    flex-direction: column;
    gap: 28px;
    overflow-y: auto;
  }

  .groupe {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .sur-titre {
    font-size: 12px;
    font-weight: 800;
    letter-spacing: 1.2px;
    text-transform: uppercase;
    color: var(--texte-secondaire);
    padding: 0 12px 6px;
  }

  .element {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 46px;
    padding: 0 14px;
    border-radius: 16px;
    border: 2px solid transparent;
    background: transparent;
    color: var(--givre);
    font-size: 15px;
    font-weight: 700;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--transition),
      border-color var(--transition);
  }

  .element:hover:not(.actif) {
    background: var(--surface);
  }

  .element.actif {
    border-color: var(--givre);
    background: var(--cobalt);
    color: var(--blanc);
    font-weight: 800;
  }

  .texte {
    flex: 1;
  }

  .compteur {
    min-width: 24px;
    height: 24px;
    padding: 0 7px;
    border-radius: 999px;
    background: var(--abysse);
    color: var(--givre);
    font-size: 12px;
    font-weight: 900;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pastille {
    width: 12px;
    height: 12px;
    border-radius: 4px;
    border: 2px solid var(--givre);
  }

  .local {
    margin-top: auto;
    padding: 16px;
    border-radius: 20px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .bouclier {
    flex: none;
    display: flex;
    margin-top: 1px;
    color: var(--menthe);
  }

  .local-titre {
    font-size: 14px;
    font-weight: 800;
  }

  .local-texte {
    font-size: 13px;
    font-weight: 500;
    color: var(--texte-secondaire);
    line-height: 1.4;
  }
</style>
