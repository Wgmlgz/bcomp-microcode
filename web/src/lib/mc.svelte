<script lang="ts">
  import 'virtual:uno.css';
  import type { WTable } from 'bcomp-microcode-parser';

  export let idx: number;
  export let table: WTable;

  const mc = table.get(idx);
  let s: string = mc.decoded;
  let label: string = mc.label;
  let microcode: bigint = mc.encoded;
  let err: string | null = null;

  const on_string = (e: { currentTarget: HTMLInputElement }) => {
    try {
      console.log(table, e.currentTarget.value)
      microcode = table.from_string(e.currentTarget.value);
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };

  const on_microcode = (e: { currentTarget: HTMLInputElement }) => {
    try {
      s = table.to_string(BigInt(parseInt(e.currentTarget.value, 16)));
      err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };
  const on_label = (e: { currentTarget: HTMLInputElement }) => {
    try {
      table.set_label(idx, e.currentTarget.value);
      // err = null;
    } catch (e) {
      console.log(e);
      err = e as string;
    }
  };
</script>

<div>
  <div class="flex gap-10">
    <input
      type="text"
      pattern="#[0-9a-fA-F]{0 - 10}"
      value={(() => microcode.toString(16))()}
      on:input={on_microcode}
    />
    <input value={label} on:input={on_label} />
    <input value={s} on:input={on_string} />
  </div>

  {#if err === null}
    <pre>
		{microcode.toString(16).padStart(10, '0')}
	</pre>
  {:else}
    <p class="text-red">
      {err}
    </p>
  {/if}
</div>
