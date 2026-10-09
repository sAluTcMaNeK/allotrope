<!--
  Contrôle segmenté de la charte (DESIGN.md, « Contrôle segmenté ») :
  conteneur Abysse, segment actif Cobalt bordé de Givre.
-->
<script lang="ts" generics="T extends string">
  import type { Component } from "svelte";

  interface Option {
    id: T;
    libelle: string;
    icone?: Component<{ size?: number; strokeWidth?: number }>;
  }

  interface Props {
    options: Option[];
    valeur: T;
    onchange: (valeur: T) => void;
    /** Nom accessible du groupe (lu par les lecteurs d'écran). */
    etiquette: string;
    desactive?: boolean;
  }
  let { options, valeur, onchange, etiquette, desactive = false }: Props = $props();
</script>

<div class="segments" role="group" aria-label={etiquette}>
  {#each options as option (option.id)}
    <button
      class="segment"
      class:actif={option.id === valeur}
      aria-pressed={option.id === valeur}
      disabled={desactive}
      onclick={() => onchange(option.id)}
    >
      {#if option.icone}
        <option.icone size={16} strokeWidth={2.6} />
      {/if}
      {option.libelle}
    </button>
  {/each}
</div>

<style>
  /* Les segments passent à la ligne s'ils sont nombreux (9 formats d'image) */
  .segments {
    display: inline-flex;
    flex-wrap: wrap;
    max-width: 100%;
    gap: 6px;
    padding: 5px;
    border-radius: 16px;
    border: 2px solid var(--ligne);
    background: var(--abysse);
  }

  .segment {
    flex: 1 0 auto;
    min-width: 72px;
    height: 40px;
    padding: 0 18px;
    border-radius: 12px;
    border: 2px solid transparent;
    background: transparent;
    color: var(--givre);
    font-size: 14px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background-color var(--transition),
      border-color var(--transition);
  }

  .segment:hover:not(.actif):not(:disabled) {
    background: var(--surface);
  }

  /* Couleur du segment actif réglable par le parent (bascule Convertir / Compresser) */
  .segment.actif {
    border-color: var(--givre);
    background: var(--segment-actif, var(--cobalt));
    color: var(--segment-actif-texte, var(--blanc));
    font-weight: 900;
  }

  .segment:disabled {
    cursor: default;
  }

  .segment:disabled:not(.actif) {
    opacity: 0.5;
  }
</style>
