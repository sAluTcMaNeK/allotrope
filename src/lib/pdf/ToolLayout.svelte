<!-- Gabarit commun des outils PDF : en-tête avec retour, contenu défilant, pied de page facultatif. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { ArrowLeft } from "@lucide/svelte";

  interface Props {
    titre: string;
    description: string;
    onretour: () => void;
    children: Snippet;
    /** Pied de page (destination + bouton principal), affiché sous la zone défilante. */
    pied?: Snippet;
  }
  let { titre, description, onretour, children, pied }: Props = $props();
</script>

<div class="outil">
  <div class="defilement">
    <header class="entete">
      <button class="retour" onclick={onretour}>
        <ArrowLeft size={16} strokeWidth={2.6} />
        Outils PDF
      </button>
      <h1>{titre}</h1>
      <p class="description">{description}</p>
    </header>
    {@render children()}
  </div>
  {#if pied}
    {@render pied()}
  {/if}
</div>

<style>
  .outil {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .defilement {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 24px 40px 32px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .entete {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  /* Bouton « Secondaire » de la charte, en version compacte */
  .retour {
    height: 44px;
    padding: 0 16px;
    margin-bottom: 8px;
    border-radius: 999px;
    border: 2px solid var(--ligne-forte);
    background: transparent;
    color: var(--givre);
    font-size: 14px;
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .retour:hover {
    border-color: var(--givre);
  }

  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 900;
    letter-spacing: -0.3px;
  }

  .description {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--texte-secondaire);
  }
</style>
