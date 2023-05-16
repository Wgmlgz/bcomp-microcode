<script lang="ts">
  import Mc from '$lib/mc.svelte';
  import 'carbon-components-svelte/css/white.css';
  import { WTable } from 'bcomp-microcode-parser';
  import { onMount } from 'svelte';
  import { Table, TableBody, TableHeader, TableHead, TableRow } from 'carbon-components-svelte';
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
    Content,
    Grid,
    Row,
    Column
  } from 'carbon-components-svelte';

  let isSideNavOpen = false;

</script>

<Header company="WGMLGZ" platformName="bcomp microcode" bind:isSideNavOpen>
  <svelte:fragment slot="skip-to-content">
    <SkipToContent />
  </svelte:fragment>
  <HeaderNav>
    <HeaderNavItem href="/" text="Link 1" />
    <HeaderNavItem href="/" text="Link 2" />
    <HeaderNavItem href="/" text="Link 3" />
    <HeaderNavMenu text="Menu">
      <HeaderNavItem href="/" text="Link 1" />
      <HeaderNavItem href="/" text="Link 2" />
      <HeaderNavItem href="/" text="Link 3" />
    </HeaderNavMenu>
    <HeaderNavItem href="/" text="Link 4" />
  </HeaderNav>
</Header>

<Content>
  <slot></slot>
</Content>
