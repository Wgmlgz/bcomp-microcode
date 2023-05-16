<script lang="ts">
  import Mc from '$lib/mc.svelte';

  import { WTable } from 'bcomp-microcode-parser';
  import { onMount } from 'svelte';

  let table: WTable | null = null;
  onMount(() => {
    table = WTable.new();
  });
</script>

{#if table}
  {#each new Array(0xff) as item, idx}
    <!-- content here -->
    <Mc {idx} bind:table />
  {/each}
{/if}
