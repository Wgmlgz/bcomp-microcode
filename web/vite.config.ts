import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import UnoCSS from 'unocss/vite';

export default defineConfig({
	plugins: [UnoCSS(), wasm(), topLevelAwait(), sveltekit()]
});
