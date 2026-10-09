<!--
  Réglages de l'opération, à choisir avant (ou après) avoir ajouté des fichiers :
  format de sortie sur la page d'un type de fichiers, réglages de compression.
-->
<script lang="ts">
  import { Info } from "@lucide/svelte";
  import CompressionFields from "./CompressionFields.svelte";
  import Segmented from "./Segmented.svelte";
  import { FAMILLES, type Famille, type Mode, type OptionsCompression } from "./fichiers";

  interface Props {
    mode: Mode;
    /** Famille de la page, ou `null` sur l'accueil. */
    famille: Famille | null;
    /** Format de sortie choisi pour la famille (onglet Conversion). */
    cible: string;
    compression: OptionsCompression;
    desactive: boolean;
    oncible: (cible: string) => void;
    oncompression: (modification: Partial<OptionsCompression>) => void;
  }
  let { mode, famille, cible, compression, desactive, oncible, oncompression }: Props = $props();

  const infos = $derived(famille ? FAMILLES[famille] : null);
  /** Message si l'outil n'est pas encore disponible pour cette famille. */
  const indisponible = $derived(infos ? (mode === "convertir" ? infos.conversion : infos.compression) : null);
</script>

{#if mode === "convertir" && !infos}
  <p class="astuce">
    <Info size={16} strokeWidth={2.6} />
    Le format de sortie se choisit pour chaque fichier. Pour convertir tout un lot dans le même format,
    choisissez d'abord un type de fichiers dans le menu.
  </p>
{:else}
  <section class="panneau" aria-label="Réglages">
    {#if mode === "convertir" && infos}
      <div class="champ">
        <div class="etiquette">Convertir en</div>
        <span class="bascule">
          <Segmented
            options={infos.cibles}
            valeur={cible}
            onchange={oncible}
            etiquette="Format de sortie"
            desactive={desactive || indisponible !== null}
          />
        </span>
      </div>
    {:else if !indisponible}
      <CompressionFields id="file" options={compression} onchange={oncompression} {desactive} />
    {/if}

    {#if indisponible}
      <p class="note">
        <Info size={16} strokeWidth={2.6} />
        {indisponible}
      </p>
    {/if}
  </section>
{/if}

<style>
  .panneau {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px 20px;
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--surface);
  }

  .champ {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .etiquette {
    font-size: 14px;
    font-weight: 800;
  }

  .bascule {
    --segment-actif: var(--mode-convertir);
    --segment-actif-texte: var(--mode-convertir-texte);
  }

  .astuce,
  .note {
    margin: 0;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
    line-height: 1.4;
  }

  .astuce :global(svg),
  .note :global(svg) {
    flex: none;
    margin-top: 1px;
  }
</style>
