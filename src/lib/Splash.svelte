<!--
  Écran de démarrage : le logo complet officiel (assets/allotrope-logo-complet.svg) s'assemble,
  puis l'écran s'efface. Le logo n'est ni redessiné ni modifié : on anime seulement ses calques existants
  (les trois formes tombent l'une après l'autre, puis le mot « allotrope » glisse en place).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import logo from "../../assets/allotrope-logo-complet.svg?raw";

  interface Props {
    onfin: () => void;
  }
  let { onfin }: Props = $props();

  const mouvementReduit = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  onMount(() => {
    const minuterie = setTimeout(onfin, mouvementReduit ? 600 : 1900);
    return () => clearTimeout(minuterie);
  });
</script>

<div class="splash" data-tauri-drag-region out:fade={{ duration: mouvementReduit ? 0 : 350 }}>
  <!-- Contenu sûr : fichier SVG du projet, importé tel quel -->
  <div class="logo" role="img" aria-label="Allotrope">{@html logo}</div>
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--abysse);
  }

  .logo {
    width: min(560px, 80vw);
    pointer-events: none;
  }

  .logo :global(svg) {
    display: block;
    width: 100%;
    height: auto;
  }

  /* Les trois formes (4 polygones chacune) tombent l'une après l'autre */
  .logo :global(svg > g > g:first-child polygon) {
    animation: tomber 560ms cubic-bezier(0.2, 0.9, 0.3, 1.25) both;
  }

  .logo :global(svg > g > g:first-child polygon:nth-child(n + 5)) {
    animation-delay: 150ms;
  }

  .logo :global(svg > g > g:first-child polygon:nth-child(n + 9)) {
    animation-delay: 300ms;
  }

  /* Puis le mot « allotrope » glisse en place */
  .logo :global(svg > g > g:last-child > g) {
    animation: glisser 650ms ease-out 560ms both;
  }

  @keyframes tomber {
    from {
      opacity: 0;
      transform: translateY(-28px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  @keyframes glisser {
    from {
      opacity: 0;
      transform: translateX(-40px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
</style>
