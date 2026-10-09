<!-- Bascule entre les deux outils : Convertir (bleu Cobalt) / Compresser (orange Braise). -->
<script lang="ts">
  import { ArrowLeftRight, Shrink } from "@lucide/svelte";
  import Segmented from "./Segmented.svelte";
  import type { Mode } from "./fichiers";

  interface Props {
    mode: Mode;
    onmode: (mode: Mode) => void;
    desactive?: boolean;
  }
  let { mode, onmode, desactive = false }: Props = $props();

  const options = [
    { id: "convertir" as const, libelle: "Conversion", icone: ArrowLeftRight },
    { id: "compresser" as const, libelle: "Compression", icone: Shrink },
  ];
</script>

<span
  class="bascule"
  style:--segment-actif={`var(--mode-${mode})`}
  style:--segment-actif-texte={`var(--mode-${mode}-texte)`}
>
  <Segmented {options} valeur={mode} onchange={onmode} etiquette="Outil" {desactive} />
</span>

<style>
  .bascule {
    display: contents;
  }
</style>
