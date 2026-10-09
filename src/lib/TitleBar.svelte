<!--
  Barre de titre personnalisée (la fenêtre n'a pas de décoration système, voir tauri.conf.json).
  Les zones marquées `data-tauri-drag-region` permettent de déplacer la fenêtre ;
  un double-clic dessus agrandit / restaure la fenêtre.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Copy, Minus, Settings, Square, X } from "@lucide/svelte";
  import logo from "../../assets/allotrope-logo-sans-lisere.svg";

  interface Props {
    /** Vrai quand l'écran Paramètres est affiché. */
    parametresActif: boolean;
    onparametres: () => void;
  }
  let { parametresActif, onparametres }: Props = $props();

  const fenetre = getCurrentWindow();
  let agrandie = $state(false);

  onMount(() => {
    // Suit l'état agrandi / restauré pour afficher la bonne icône
    const majEtat = async () => {
      agrandie = await fenetre.isMaximized();
    };
    majEtat();
    const arret = fenetre.onResized(majEtat);
    return () => {
      arret.then((desabonner) => desabonner());
    };
  });
</script>

<header class="barre" data-tauri-drag-region>
  <div class="marque" data-tauri-drag-region>
    <img src={logo} alt="allotrope" class="logo" draggable="false" />
  </div>

  <div class="actions">
    <button
      class="parametres"
      class:actif={parametresActif}
      aria-label="Paramètres"
      aria-pressed={parametresActif}
      onclick={onparametres}
    >
      <Settings size={20} strokeWidth={2.2} />
    </button>
    <button class="controle" aria-label="Réduire" onclick={() => fenetre.minimize()}>
      <Minus size={16} strokeWidth={2.4} />
    </button>
    <button
      class="controle"
      aria-label={agrandie ? "Restaurer" : "Agrandir"}
      onclick={() => fenetre.toggleMaximize()}
    >
      {#if agrandie}
        <Copy size={16} strokeWidth={2.4} />
      {:else}
        <Square size={16} strokeWidth={2.4} />
      {/if}
    </button>
    <button class="controle fermer" aria-label="Fermer" onclick={() => fenetre.close()}>
      <X size={16} strokeWidth={2.4} />
    </button>
  </div>
</header>

<style>
  .barre {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 16px 12px 24px;
    border-bottom: 2px solid var(--ligne);
    flex: none;
  }

  .marque {
    display: flex;
    align-items: center;
    align-self: stretch;
  }

  .logo {
    height: 30px;
    display: block;
    /* Laisse le glisser-déposer de la fenêtre passer à travers le logo */
    pointer-events: none;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  button {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background-color var(--transition),
      color var(--transition),
      border-color var(--transition);
  }

  .parametres {
    margin-right: 12px;
    border-radius: 14px;
    border: 2px solid var(--ligne);
    background: var(--surface);
    color: var(--givre);
  }

  .parametres:hover:not(.actif) {
    border-color: var(--ligne-forte);
    background: var(--surface-active);
  }

  /* Même traitement que l'élément actif de la barre latérale */
  .parametres.actif {
    border-color: var(--givre);
    background: var(--cobalt);
    color: var(--blanc);
  }

  .controle {
    border-radius: 12px;
    border: none;
    background: transparent;
    color: var(--texte-secondaire);
  }

  .controle:hover {
    background: var(--surface);
    color: var(--givre);
  }

  /* Jamais de texte blanc sur Braise : l'icône passe en Abysse */
  .fermer:hover {
    background: var(--braise);
    color: var(--abysse);
  }
</style>
