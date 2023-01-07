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
