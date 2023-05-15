<script lang="ts">
  import 'virtual:uno.css'
	import { greet, parse_microcode } from 'bcomp-microcode-parser';
	// console.log(greet())
	let s: string = '';
	let microcode: bigint = 0n;
	let err: string | null = null;

	$: try {
		microcode = parse_microcode(s);
    err = null
	} catch (e) {
    console.log(e)
    err = e as string
  }
</script>

<input bind:value={s} />



{#if err === null}
	<pre>
		{microcode.toString(16).padStart(10, '0')}
	</pre>
{:else}
	<p class="text-red">
		{err}
	</p>
{/if}
