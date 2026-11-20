<!--
  Aperçu avant / après d'un fichier traité : miniatures côte à côte (images), poids et gain obtenu.
  Fenêtre <dialog> native : la touche Échap la ferme et le focus reste à l'intérieur.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowRight, Folder, ImageOff, X } from "@lucide/svelte";
  import { FAMILLES, LIBELLES_FORMAT, formaterTaille, type ElementFile, type Mode } from "./fichiers";

  interface Props {
    element: ElementFile;
    mode: Mode;
    onfermer: () => void;
    onouvrir: (chemin: string) => void;
  }
  let { element, mode, onfermer, onouvrir }: Props = $props();

  let dialogue = $state<HTMLDialogElement>();
  let avant = $state<string | null>(null);
  let apres = $state<string | null>(null);
  /** Vrai si une miniature n'a pas pu être produite (format sans aperçu). */
  let sansAvant = $state(false);
  let sansApres = $state(false);

  const info = $derived(element.info);
  const resultat = $derived(element.resultat);
  const estImage = $derived(info.famille === "image");
  const famille = $derived(info.famille ? FAMILLES[info.famille] : null);
  const extensionApres = $derived(
    resultat?.chemin?.split(".").pop()?.toUpperCase() ?? (element.format ? LIBELLES_FORMAT[element.format] : ""),
  );
  /** Écart de poids en pourcentage : négatif quand le fichier a maigri. */
  const ecart = $derived(resultat ? Math.round((resultat.taille / info.taille - 1) * 100) : 0);
  const difference = $derived(resultat ? Math.abs(info.taille - resultat.taille) : 0);

  async function miniature(chemin: string): Promise<string> {
    return invoke<string>("apercu_image", { chemin, coteMax: 720 });
  }

  onMount(() => {
    dialogue?.showModal();
    if (estImage && resultat?.chemin) {
      miniature(info.chemin).then((url) => (avant = url), () => (sansAvant = true));
      miniature(resultat.chemin).then((url) => (apres = url), () => (sansApres = true));
    }
  });
</script>

<dialog
  bind:this={dialogue}
  class="apercu"
  aria-labelledby="titre-apercu"
  onclose={onfermer}
  onclick={(e) => {
    // Un clic sur le voile (hors du cadre) ferme l'aperçu
    if (e.target === dialogue) dialogue?.close();
  }}
>
  <div class="cadre">
    <header>
      <div>
        <h2 id="titre-apercu">Avant / après</h2>
        <div class="nom" title={info.chemin}>{info.nom}</div>
      </div>
      <button class="carre" aria-label="Fermer l'aperçu" onclick={() => dialogue?.close()}>
        <X size={18} strokeWidth={2.4} />
      </button>
    </header>

    <div class="comparaison">
      {#each [{ titre: "Avant", url: avant, sans: sansAvant, format: info.extension.toUpperCase(), taille: info.taille }, { titre: "Après", url: apres, sans: sansApres, format: extensionApres, taille: resultat?.taille ?? 0 }] as cote, i (cote.titre)}
        {#if i === 1}
          <span class="fleche" aria-hidden="true"><ArrowRight size={22} strokeWidth={2.6} /></span>
        {/if}
        <figure class="cote">
          <div class="image">
            {#if estImage && cote.url}
              <img src={cote.url} alt={`${cote.titre} : ${info.nom}`} />
            {:else if estImage && !cote.sans}
              <span class="chargement">Chargement…</span>
            {:else}
              <div class="badge" style:background={famille?.couleur} style:color={famille?.texte}>
                {#if estImage}<ImageOff size={22} strokeWidth={2.4} />{:else}{cote.format.slice(0, 4)}{/if}
              </div>
            {/if}
          </div>
          <figcaption>
            <span class="sur-titre">{cote.titre}</span>
            <span class="details">{cote.format} · {formaterTaille(cote.taille)}</span>
          </figcaption>
        </figure>
      {/each}
    </div>

    <footer>
      {#if ecart < 0}
        <div class="pastille succes">−{Math.abs(ecart)} % · {formaterTaille(difference)} gagnés</div>
      {:else if ecart > 0}
        <div class="pastille neutre">
          +{ecart} % · {mode === "convertir" ? "le nouveau format est plus lourd" : "plus lourd"}
        </div>
      {:else}
        <div class="pastille neutre">Poids identique</div>
      {/if}
      {#if resultat?.chemin}
        <button class="secondaire" onclick={() => resultat?.chemin && onouvrir(resultat.chemin)}>
          <Folder size={18} strokeWidth={2.4} />
          Afficher dans le dossier
        </button>
      {/if}
    </footer>
  </div>
</dialog>

<style>
  .apercu {
    width: min(920px, calc(100vw - 64px));
    max-height: calc(100vh - 64px);
    padding: 0;
    border: none;
    background: transparent;
    color: var(--givre);
    font-family: var(--police);
  }

  /* Voile opaque (pas de flou ni d'ombre : voir DESIGN.md) */
  .apercu::backdrop {
    background: var(--voile);
  }

  .cadre {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 28px;
    border-radius: 26px;
    border: 3px solid var(--givre);
    background: var(--surface);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
  }

  .nom {
    margin-top: 2px;
    font-size: 13px;
    font-weight: 600;
    color: var(--texte-secondaire);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .comparaison {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .fleche {
    display: flex;
    flex: none;
    color: var(--texte-secondaire);
  }

  .cote {
    flex: 1;
    min-width: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .image {
    height: min(46vh, 380px);
    border-radius: 22px;
    border: 2px solid var(--ligne);
    background: var(--abysse);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .chargement {
    font-size: 13px;
    font-weight: 700;
    color: var(--texte-secondaire);
  }

  .badge {
    width: 72px;
    height: 72px;
    border-radius: 16px;
    border: 2px solid var(--givre);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    font-weight: 900;
  }

  figcaption {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sur-titre {
    font-size: 12px;
    font-weight: 800;
    letter-spacing: 1.2px;
    text-transform: uppercase;
    color: var(--texte-secondaire);
  }

  .details {
    font-size: 15px;
    font-weight: 800;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .pastille {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 16px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 800;
  }

  .pastille.succes {
    background: var(--menthe-fond);
    color: var(--menthe);
  }

  .pastille.neutre {
    background: var(--abysse);
    color: var(--texte-secondaire);
  }

  /* Bouton secondaire de la charte */
  .secondaire {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 18px;
    border-radius: 999px;
    border: 2px solid var(--ligne-forte);
    background: transparent;
    color: var(--givre);
    font-family: inherit;
    font-size: 15px;
    font-weight: 800;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .secondaire:hover {
    border-color: var(--givre);
  }

  /* Bouton « Icône seule » de la charte */
  .carre {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 14px;
    border: 2px solid var(--ligne);
    background: transparent;
    color: var(--givre);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .carre:hover {
    border-color: var(--ligne-forte);
  }
</style>
