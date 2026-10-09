<!-- Barre latérale : familles de fichiers et rappel « 100 % local ». -->
<script lang="ts">
  import { LayoutGrid, ShieldCheck } from "@lucide/svelte";
  import { FAMILLES, type Famille } from "./fichiers";

  const familles = Object.entries(FAMILLES) as [Famille, (typeof FAMILLES)[Famille]][];
</script>

<nav class="laterale" aria-label="Navigation principale">
  <div class="groupe">
    <div class="sur-titre">Types de fichiers</div>
    <button class="element actif" aria-current="page">
      <LayoutGrid size={18} strokeWidth={2.4} />
      Tout convertir
    </button>
    {#each familles as [id, famille] (id)}
      <button class="element">
        <span class="pastille" style:background={famille.couleur}></span>
        {famille.libelle}
      </button>
    {/each}
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

  button {
    display: flex;
    align-items: center;
    text-align: left;
    cursor: pointer;
    color: var(--givre);
    transition:
      background-color var(--transition),
      border-color var(--transition);
  }

  .element {
    gap: 12px;
    height: 46px;
    padding: 0 14px;
    border-radius: 16px;
    border: 2px solid transparent;
    background: transparent;
    font-size: 15px;
    font-weight: 700;
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
