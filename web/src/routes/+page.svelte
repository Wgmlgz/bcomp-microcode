<script lang="ts">
  import Mc from '$lib/mc.svelte';
  import 'carbon-components-svelte/css/white.css';
  import { Instruction, WTable } from 'bcomp-microcode-parser';
  import { onMount } from 'svelte';
  import { Table, TableBody, TableHeader, TableHead, TableRow } from 'carbon-components-svelte';
  import { TextInput } from 'carbon-components-svelte';
  import { CodeSnippet } from 'carbon-components-svelte';
  import { Grid, Row, Column } from 'carbon-components-svelte';
  import { get_hex } from '$lib/utils';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  let table: WTable | null = null;

  let diff: Record<number, { label: string; hex: string }> = {};
  onMount(() => {
    table = WTable.new();
  });

  // const sus = () => {
  //   if (window?.process?.browser) {
  //     $page.url.searchParams.set('diff', encodeURIComponent(JSON.stringify(diff)));
  //     goto(`?${$page.url.searchParams.toString()}`);
  //   }
  // };
  // $: {
  //   if (diff) sus();
  // }
</script>

<Grid>
  <Row>
    <Column>
      <h1>Microcode memory</h1>
      <p>i hate java</p>
      <div>
        <Table useStaticWidth>
          <TableRow>
            <TableHeader>Addr</TableHeader>
            <TableHeader>Hex</TableHeader>
            <TableHeader>Label</TableHeader>
            <TableHeader>Decoded</TableHeader>
          </TableRow>

          <TableBody>
            {#if table}
              {#each new Array(0x100) as item, idx}
                <Mc {idx} bind:diff bind:table />
              {/each}
            {/if}
          </TableBody>
        </Table>
      </div>
    </Column>
    <Column>
      <h1>Script to patch cli bcomp</h1>
      <div>
        <CodeSnippet
          type="multi"
          code={Object.values(diff)
            .map(({ hex }, idx) => `ma ${get_hex(idx, 2)}\nmw ${hex}`)
            .join('\n')}
        />
      </div>
    </Column>
  </Row>
</Grid>
