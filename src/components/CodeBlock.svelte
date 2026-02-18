<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  export let language = "";
  export let code = "";

  let output = "";
  let error = "";
  let executing = false;

  async function run() {
    executing = true;
    output = "";
    error = "";
    try {
        // Normalizar lenguaje para el backend
        let langCmd = language.toLowerCase();
        if (langCmd === "py" || langCmd === "python3") langCmd = "python";
        if (langCmd === "js" || langCmd === "javascript") langCmd = "node";

        output = await invoke("run_code", { language: langCmd, code });
    } catch (e) {
        error = e as string;
    } finally {
        executing = false;
    }
  }
</script>

<div class="my-2 bg-black rounded p-2 border border-gray-600">
  <div class="flex justify-between items-center mb-1 bg-gray-800 p-1 rounded">
    <span class="text-xs text-gray-400 font-mono">{language}</span>
    <button 
        class="bg-green-700 hover:bg-green-600 text-white text-xs px-2 py-1 rounded flex items-center gap-1 disabled:opacity-50"
        on:click={run}
        disabled={executing}
    >
        {#if executing}
            <span>Corriendo...</span>
        {:else}
            <span>▶ Ejecutar</span>
        {/if}
    </button>
  </div>
  <pre class="overflow-x-auto text-sm text-gray-300 p-2 font-mono"><code>{code}</code></pre>
  
  {#if output}
    <div class="mt-2 text-xs">
        <span class="block text-green-500 font-bold">Salida:</span>
        <pre class="bg-gray-900 p-2 rounded text-green-300 mt-1 whitespace-pre-wrap">{output}</pre>
    </div>
  {/if}
  {#if error}
    <div class="mt-2 text-xs">
        <span class="block text-red-500 font-bold">Error:</span>
        <pre class="bg-gray-900 p-2 rounded text-red-300 mt-1 whitespace-pre-wrap">{error}</pre>
    </div>
  {/if}
</div>
