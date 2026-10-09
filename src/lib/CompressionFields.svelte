<!--
  Réglages de compression : par taux (curseur) ou par taille cible, plus la taille maximale.
  Utilisé au-dessus de la file d'attente et dans les Paramètres (valeurs par défaut).
-->
<script lang="ts">
  import { ChevronDown } from "@lucide/svelte";
  import Segmented from "./Segmented.svelte";
  import { TAILLES_MAX, type OptionsCompression, type Unite } from "./fichiers";

  interface Props {
    options: OptionsCompression;
    onchange: (modification: Partial<OptionsCompression>) => void;
    desactive?: boolean;
    /** Préfixe des identifiants HTML (deux instances peuvent coexister). */
    id: string;
  }
  let { options, onchange, desactive = false, id }: Props = $props();

  const types = [
    { id: "taux" as const, libelle: "Par taux" },
    { id: "taille" as const, libelle: "Par taille cible" },
  ];
</script>

<div class="champs">
  <Segmented
    options={types}
    valeur={options.type}
    onchange={(type) => onchange({ type })}
    etiquette="Mode de compression"
    {desactive}
  />

  {#if options.type === "taux"}
    <div class="champ large">
      <div class="ligne">
        <label class="etiquette" for="{id}-taux">Taux de compression</label>
        <span class="valeur">{options.taux} %</span>
      </div>
      <input
        id="{id}-taux"
        type="range"
        min="5"
        max="95"
        step="5"
        value={options.taux}
        disabled={desactive}
        oninput={(e) => onchange({ taux: Number(e.currentTarget.value) })}
      />
      <div class="ligne legende">
        <span>Meilleure qualité</span>
        <span>Fichier plus léger</span>
      </div>
    </div>
  {:else}
    <div class="champ">
      <label class="etiquette" for="{id}-cible">Taille maximale par fichier</label>
      <div class="cible">
        <input
          id="{id}-cible"
          class="nombre"
          type="number"
          min="1"
          step="any"
          value={options.tailleCible}
          disabled={desactive}
          onchange={(e) => {
            const valeur = Number(e.currentTarget.value);
            if (valeur > 0) onchange({ tailleCible: valeur });
            else e.currentTarget.value = String(options.tailleCible);
          }}
        />
        <span class="selecteur">
          <select
            aria-label="Unité"
            value={options.unite}
            disabled={desactive}
            onchange={(e) => onchange({ unite: e.currentTarget.value as Unite })}
          >
            <option value="Ko">Ko</option>
            <option value="Mo">Mo</option>
          </select>
          <span class="chevron"><ChevronDown size={14} strokeWidth={3} /></span>
        </span>
      </div>
    </div>
  {/if}

  <div class="champ">
    <label class="etiquette" for="{id}-dimensions">Dimensions maximales</label>
    <span class="selecteur">
      <select
        id="{id}-dimensions"
        value={String(options.tailleMax)}
        disabled={desactive}
        onchange={(e) => {
          const v = e.currentTarget.value;
          onchange({ tailleMax: v === "null" ? null : Number(v) });
        }}
      >
        {#each TAILLES_MAX as t (t.libelle)}
          <option value={String(t.valeur)}>{t.libelle}</option>
        {/each}
      </select>
      <span class="chevron"><ChevronDown size={14} strokeWidth={3} /></span>
    </span>
  </div>
</div>

<style>
  .champs {
    --segment-actif: var(--mode-compresser);
    --segment-actif-texte: var(--mode-compresser-texte);
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 20px 28px;
  }

  .champ {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .champ.large {
    flex: 1 1 260px;
  }

  .ligne {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .etiquette {
    font-size: 14px;
    font-weight: 800;
  }

  .valeur {
    font-size: 14px;
    font-weight: 900;
    color: var(--mode-compresser);
  }

  .legende {
    font-size: 12px;
    font-weight: 700;
    color: var(--texte-secondaire);
  }

  input[type="range"] {
    width: 100%;
    accent-color: var(--mode-compresser);
  }

  .cible {
    display: flex;
    gap: 8px;
  }

  .nombre,
  select {
    height: 44px;
    border-radius: 14px;
    border: 2px solid var(--ligne-forte);
    background: var(--abysse);
    color: var(--givre);
    font-family: inherit;
    font-size: 14px;
    font-weight: 800;
  }

  .nombre {
    width: 110px;
    padding: 0 14px;
  }

  .nombre:focus {
    border-color: var(--mode-compresser);
    outline: none;
  }

  .selecteur {
    position: relative;
    display: flex;
    align-items: center;
  }

  select {
    appearance: none;
    padding: 0 36px 0 16px;
    cursor: pointer;
  }

  select:disabled,
  .nombre:disabled {
    opacity: 0.6;
    cursor: default;
  }

  option {
    background: var(--surface);
  }

  .chevron {
    position: absolute;
    right: 14px;
    display: flex;
    pointer-events: none;
  }
</style>
