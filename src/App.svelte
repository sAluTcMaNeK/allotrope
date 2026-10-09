<script lang="ts">
  import TitleBar from "./lib/TitleBar.svelte";
</script>

<div class="application">
  <TitleBar />

  <!-- Contenu provisoire : l'écran d'accueil (zone de dépôt) arrive à l'étape suivante. -->
  <main class="contenu">
    <h1>Que voulez-vous convertir ?</h1>
    <p>Les fondations visuelles sont en place.</p>
  </main>
</div>

<style>
  .application {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .contenu {
    flex: 1;
    min-height: 0;
    padding: 32px 40px 40px;
  }

  h1 {
    margin: 0 0 8px;
    font-size: 30px;
    font-weight: 900;
    letter-spacing: -0.3px;
  }

  p {
    margin: 0;
    color: var(--texte-secondaire);
  }
</style>
