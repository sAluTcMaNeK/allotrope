<!-- Barre latérale : accueil, pages par type de fichiers, outils PDF, préréglages et rappel « 100 % local ». -->
<script lang="ts">
  import { tick } from "svelte";
  import { FileStack, House, Plus, ShieldCheck, X } from "@lucide/svelte";
  import { LISTE_FAMILLES } from "./fichiers";
  import type { Page } from "./navigation";
  import type { Prereglage } from "./prereglages";

  interface Props {
    page: Page;
    /** Nombre de fichiers en file d'attente pour chaque page. */
    compteurs: Record<string, number>;
    onnaviguer: (page: Page) => void;
    prereglages: Prereglage[];
    /** Préréglage appliqué en dernier (tant que ses réglages n'ont pas été modifiés). */
    prereglageActif: string | null;
    onprereglage: (prereglage: Prereglage) => void;
    /** Enregistre les réglages de la page affichée ; renvoie faux si ce n'est pas possible ici. */
    onenregistrer: (nom: string) => boolean;
    onsupprimer: (id: string) => void;
  }
  let { page, compteurs, onnaviguer, prereglages, prereglageActif, onprereglage, onenregistrer, onsupprimer }: Props =
    $props();

  /** Saisie du nom d'un nouveau préréglage. */
  let saisie = $state(false);
  let nom = $state("");
  let champ = $state<HTMLInputElement>();

  async function commencerSaisie() {
    saisie = true;
    nom = "";
    await tick();
    champ?.focus();
  }

  function valider() {
    const propre = nom.trim();
    if (propre && onenregistrer(propre)) saisie = false;
    else if (!propre) saisie = false;
  }
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

  <div class="groupe">
    <div class="sur-titre">Préréglages</div>
    {#each prereglages as prereglage (prereglage.id)}
      <div class="prereglage" class:actif={prereglageActif === prereglage.id}>
        <button
          class="nom-prereglage"
          aria-pressed={prereglageActif === prereglage.id}
          onclick={() => onprereglage(prereglage)}
        >
          {prereglage.nom}
        </button>
        <button
          class="supprimer"
          aria-label={`Supprimer le préréglage « ${prereglage.nom} »`}
          onclick={() => onsupprimer(prereglage.id)}
        >
          <X size={16} strokeWidth={2.6} />
        </button>
      </div>
    {/each}

    {#if saisie}
      <form
        class="saisie"
        onsubmit={(e) => {
          e.preventDefault();
          valider();
        }}
      >
        <input
          bind:this={champ}
          bind:value={nom}
          maxlength="40"
          placeholder="Nom du préréglage"
          aria-label="Nom du nouveau préréglage"
          onkeydown={(e) => {
            if (e.key === "Escape") saisie = false;
          }}
          onblur={() => {
            if (!nom.trim()) saisie = false;
          }}
        />
      </form>
    {:else}
      <button class="nouveau" onclick={commencerSaisie}>
        <Plus size={14} strokeWidth={3} />
        Nouveau préréglage
      </button>
    {/if}
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

  /* Préréglages : éléments plus discrets que la navigation (maquette accueil.html) */
  .prereglage {
    display: flex;
    align-items: center;
    border-radius: 14px;
    transition: background-color var(--transition);
  }

  .prereglage:hover:not(.actif) {
    background: var(--surface);
  }

  .prereglage.actif {
    background: var(--surface-active);
  }

  .nom-prereglage {
    flex: 1;
    min-width: 0;
    height: 44px;
    padding: 0 14px;
    border: none;
    background: transparent;
    color: var(--givre);
    font-family: inherit;
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }

  .prereglage.actif .nom-prereglage {
    font-weight: 800;
  }

  .supprimer {
    width: 44px;
    height: 44px;
    flex: none;
    border: none;
    border-radius: 12px;
    background: transparent;
    color: var(--texte-secondaire);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transition:
      opacity var(--transition),
      color var(--transition);
  }

  .prereglage:hover .supprimer,
  .supprimer:focus-visible {
    opacity: 1;
  }

  .supprimer:hover {
    color: var(--givre);
  }

  .nouveau {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 14px;
    border-radius: 14px;
    border: 2px dashed var(--ligne-forte);
    background: transparent;
    color: var(--lien);
    font-family: inherit;
    font-size: 14px;
    font-weight: 700;
    text-align: left;
    cursor: pointer;
    transition: border-color var(--transition);
  }

  .nouveau:hover {
    border-color: var(--lien);
  }

  .saisie input {
    width: 100%;
    height: 44px;
    padding: 0 14px;
    border-radius: 14px;
    border: 2px solid var(--givre);
    background: var(--abysse);
    color: var(--givre);
    font-family: inherit;
    font-size: 14px;
    font-weight: 700;
  }

  .saisie input::placeholder {
    color: var(--texte-secondaire);
  }

  .saisie input:focus {
    outline: none;
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
