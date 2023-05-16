<script lang="ts">
  import 'virtual:uno.css';

  import { TextInput, TableRow, TableCell } from 'carbon-components-svelte';
  import type { Instruction, WTable } from 'bcomp-microcode-parser';
  import { get_hex } from './utils';

  export let idx: number;
  export let table: WTable;
  export let diff: Record<number, {label: string, hex: string}>;

  $: {
    if (orig.encoded !== BigInt(parseInt(hex || '0', 16))) {
      diff[idx] = {
        label: instr.label,
        hex
      };
      console.log(instr)
    } else {
      delete diff[idx];
      diff = diff;
    }
  }

  let hex = '';
  const update_hex = () => (hex = get_hex(instr.encoded));

  const orig = table.get(idx);
  const instr = table.get(idx);
  update_hex();

  let err: string | null = null;

  const on_string = (e: CustomEvent<string | number | null>) => {
    console.log('on_string');

    try {
      instr.encoded = table.from_string(String(e.detail));
      update_hex();
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };

  const on_microcode = (e: CustomEvent<string | number | null>) => {
    console.log('on_microcode');
    try {
      instr.decoded = table.to_string(BigInt(parseInt(String(e.detail || '0'), 16)));
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };
  const on_label = (e: CustomEvent<string | number | null>) => {
    try {
      instr.label = String(e.detail);
      table.set(idx, instr);
      // err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };
</script>

<TableRow id={String(idx)}>
  <TableCell>
    <div class="grid items-center justify-items-center">
      {idx.toString(16).padStart(2, '0')}
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-120px -mx-15px">
      <TextInput light pattern="[0-9a-fA-F]" bind:value={hex} on:input={on_microcode} />
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-100px -mx-15px">
      <TextInput light bind:value={instr.label} on:input={on_label} />
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-500px -mx-15px">
      <TextInput
        invalid={err !== null}
        invalidText={err || undefined}
        light={true}
        bind:value={instr.decoded}
        on:input={on_string}
      />
    </div>
  </TableCell>
</TableRow>
