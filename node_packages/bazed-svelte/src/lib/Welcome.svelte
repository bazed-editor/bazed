<!--
  # Welcome
  ---
  Placeholder, currently displayed during session establishment. May be replaced with a "local"
  terminal or something allowing the user to attempt opening a document in pretense.

  TODO: implement window lacking session

    @param {SvelteComponent} slot - component to be displayed at the bottom of the window
-->
<div class="container">
  <div class="logo">BAZED</div>
  <slot />
</div>

<style lang="sass">
  .container 
    width: 100vw
    height: 100vh
    display: flex
    flex-direction: column
    font-family: monospace

  .logo
    flex: 1
    display: flex
    justify-content: center
    align-items: center
    font-size: 400px
</style>
