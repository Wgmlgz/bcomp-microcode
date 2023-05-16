<script lang="ts">
  import 'carbon-components-svelte/css/white.css';
  import { WTable } from 'bcomp-microcode-parser';
  import { onMount } from 'svelte';
  import {
    Table,
    TableBody,
    TableHeader,
    TableHead,
    TableRow,
    CodeSnippet
  } from 'carbon-components-svelte';
  import { TextInput } from 'carbon-components-svelte';

  import {
    Header,
    HeaderNav,
    HeaderNavItem,
    HeaderNavMenu,
    SideNav,
    SideNavItems,
    SideNavMenu,
    SideNavMenuItem,
    SideNavLink,
    SideNavDivider,
    SkipToContent,
    Content
  } from 'carbon-components-svelte';

  let isSideNavOpen = false;

  import Mc from '$lib/mc.svelte';
  import { Grid, Row, Column, Button } from 'carbon-components-svelte';
  import { get_hex } from '$lib/utils';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  let table: WTable | null = null;
  type Diff = Record<number, { label: string; hex: string }>;

  let diff: Diff | null = null;
  onMount(() => {
    const raw = $page.url.searchParams.get('diff');

    diff = raw ? JSON.parse(atob(raw)) : {};

    table = WTable.new();
    setTimeout(patch_labels);
  });

  const sus = () => {
    $page.url.searchParams.set('diff', btoa(JSON.stringify(diff)));
    if (table !== null)
      goto(`?${$page.url.searchParams.toString()}`, {
        keepFocus: true
      });
  };

  $: {
    if (diff) sus();
  }

  let flag = 'huh';
  const patch_labels = () => {
    flag = String(Math.random());
  };
</script>

<Header company="WGMLGZ" platformName="bcomp microcode" bind:isSideNavOpen>
  <svelte:fragment slot="skip-to-content">
    <SkipToContent />
  </svelte:fragment>
  <HeaderNav>
    <HeaderNavItem text="Reset Memory" on:click={() => window.open('/', '_self')} />
    <HeaderNavItem text="Patch Labels" on:click={patch_labels} />
    <HeaderNavItem text="Link 3" />
    <HeaderNavItem href="/" text="Link 4" />
  </HeaderNav>
</Header>

<Content>
  <Grid>
    <Row>
      <Column>
        <h1>Microcode memory</h1>
        <br />
        <br />
        <div class="overflow-y-scroll" style="height: calc(100vh - 200px);">
          <Table useStaticWidth>
            <TableRow>
              <TableHeader>Addr</TableHeader>
              <TableHeader>Hex</TableHeader>
              <TableHeader>Label</TableHeader>
              <TableHeader>Decoded</TableHeader>
            </TableRow>

            <TableBody>
              {#if table && diff}
                {#each new Array(0x100) as item, idx}
                  <Mc bind:flag {idx} bind:diff bind:table />
                {/each}
              {/if}
            </TableBody>
          </Table>
        </div>
      </Column>
      <Column>
        {#if diff}
          <h1>Script to patch cli bcomp</h1>
          <br />
          <br />
          <div>
            <!-- content here -->
            <CodeSnippet
              type="multi"
              code={Object.entries(diff)
                .map(([idx, { hex }]) => `ma ${get_hex(idx, 2)}\nmw ${hex}`)
                .join('\n')}
            />
          </div>
        {/if}
      </Column>
    </Row>
  </Grid>
</Content>
