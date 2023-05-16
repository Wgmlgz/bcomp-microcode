<script lang="ts">
  import 'virtual:uno.css';

  import { TextInput, TableRow, TableCell } from 'carbon-components-svelte';
  import { Instruction, type WTable } from 'bcomp-microcode-parser';
  import { get_hex } from './utils';

  export let idx: number;
  export let table: WTable;
  export let diff: Record<number, { label: string; hex: string }>;
  export let flag: unknown;

  const on_ok = () => {
    diff[idx] = {
      label: instr.label,
      hex
    };
    console.log(diff[idx]);
  };
  const on_none = () => {
    delete diff[idx];
    diff[idx] = diff[idx];
    // diff = diff;
  };
  $: {
    if (orig.encoded !== BigInt(parseInt(hex || '0', 16)) || orig.label !== instr.label) {
      console.log(orig.encoded, BigInt(parseInt(hex || '0', 16)));
      on_ok();
    } else {
      on_none();
    }
  }

  let hex = '';
  const update_hex = () => (hex = get_hex(instr.encoded));

  const orig = table.get(idx);
  const instr = table.get(idx);

  update_hex();

  let err: string | null = null;

  const on_string = (e: string | number | null) => {
    try {
      instr.encoded = table.from_string(String(e));
      update_hex();
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };

  const on_microcode = (e: string | number | null) => {
    try {
      instr.decoded = table.to_string(BigInt(parseInt(String(e || '0'), 16)));
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };
  const on_label = (e: string | number | null) => {
    try {
      let t = String(e);
      instr.label = t;
      table.set(idx, Instruction.new(orig.address, orig.encoded, instr.label, orig.decoded));
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };

  const t = () => {
    if (diff[idx]) {
      hex = diff[idx].hex;
      instr.label = diff[idx].label;
    }
    on_label(instr.label);
    on_microcode(hex);
  };
  $: if (flag) t();
  t();
</script>

<TableRow id={String(idx)}>
  <TableCell>
    <div class="grid items-center justify-items-center">
      {idx.toString(16).padStart(2, '0')}
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-120px -mx-15px">
      <TextInput
        light
        pattern="[0-9a-fA-F]"
        bind:value={hex}
        on:input={(e) => on_microcode(e.detail)}
      />
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-100px -mx-15px">
      <TextInput light bind:value={instr.label} on:input={(e) => on_label(e.detail)} />
    </div>
  </TableCell>
  <TableCell class="align-top! pt-1">
    <div class="w-500px -mx-15px">
      <TextInput
        invalid={err !== null}
        invalidText={err || undefined}
        light={true}
        bind:value={instr.decoded}
        on:input={(e) => on_string(e.detail)}
      />
    </div>
  </TableCell>
</TableRow>
