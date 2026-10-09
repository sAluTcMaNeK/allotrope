<!-- Miniature d'une page PDF, dessinée par pdf.js quand elle devient visible à l'écran. -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { PDFDocumentProxy } from "pdfjs-dist";
  import { dessinerPage } from "./pdf";

  interface Props {
    document: PDFDocumentProxy;
    numero: number;
    /** Rotation affichée, en degrés (aperçu de la rotation choisie). */
    rotation: number;
    largeur: number;
  }
  let { document, numero, rotation, largeur }: Props = $props();

  let canvas: HTMLCanvasElement;
  let prete = $state(false);
  // Tournée d'un quart de tour, la page échange largeur et hauteur : on inverse les limites avant rotation
  const quartDeTour = $derived(Math.abs(rotation / 90) % 2 === 1);
  const maxLargeur = $derived(quartDeTour ? largeur * 1.3 : largeur);
  const maxHauteur = $derived(quartDeTour ? largeur : largeur * 1.3);

  onMount(() => {
    // Ne dessine que les pages visibles : un PDF de 300 pages reste fluide
    const observateur = new IntersectionObserver(
      async (entrees) => {
        if (!entrees.some((e) => e.isIntersecting)) return;
        observateur.disconnect();
        try {
          await dessinerPage(document, numero, canvas, largeur);
          prete = true;
        } catch {
          // Page illisible : la case reste vide
        }
      },
      { rootMargin: "200px" },
    );
    observateur.observe(canvas);
    return () => observateur.disconnect();
  });
</script>

<div class="cadre" style:width="{largeur}px" style:height="{largeur * 1.3}px">
  <canvas
    bind:this={canvas}
    class:prete
    style:max-width="{maxLargeur}px"
    style:max-height="{maxHauteur}px"
    style:transform="rotate({rotation}deg)"
  ></canvas>
</div>

<style>
  .cadre {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  canvas {
    width: auto;
    height: auto;
    border-radius: 6px;
    opacity: 0;
    transition:
      opacity var(--transition),
      transform var(--transition);
  }

  /* Le fond blanc de la page n'apparaît qu'une fois la page dessinée */
  canvas.prete {
    opacity: 1;
    background: var(--blanc);
  }
</style>
