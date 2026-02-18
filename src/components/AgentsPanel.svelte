<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  // id, name, description, system_prompt, default_model, is_built_in
  let agents: [number, string, string, string, string, boolean][] = []; 
  
  let newName = "";
  let newDesc = "";
  let newPrompt = "";
  let newModel = "chatgpt";
  let message = "";

  async function loadAgents() {
    try {
      agents = await invoke("get_agents");
      dispatch("update", agents); // Notificar al padre para actualizar selector
    } catch (e) {
      console.error("Failed to load agents", e);
    }
  }

  async function createAgent() {
    if (!newName || !newPrompt) {
        message = "Nombre y Prompt requeridos";
        return;
    }
    try {
      await invoke("create_agent", { 
        name: newName, 
        description: newDesc, 
        systemPrompt: newPrompt, 
        defaultModel: newModel 
      });
      message = "Agente creado";
      newName = "";
      newDesc = "";
      newPrompt = "";
      loadAgents();
    } catch (e) {
      message = "Error: " + e;
    }
  }

  async function deleteAgent(id: number, name: string) {
    if (!confirm(`¿Borrar agente ${name}?`)) return;
    try {
      await invoke("delete_agent", { id });
      loadAgents();
    } catch (e) {
        message = "Error borrando: " + e;
    }
  }

  onMount(() => {
    loadAgents();
  });
</script>

<div class="bg-gray-800 p-4 rounded-lg shadow-md mt-4">
  <h2 class="text-xl font-bold mb-4 text-gray-200">Gestor de Agentes</h2>
  
  <div class="mb-4 space-y-2 bg-gray-700 p-2 rounded">
    <input class="bg-gray-600 p-2 rounded text-white w-full" placeholder="Nombre (ej. Experto Rust)" bind:value={newName} />
    <input class="bg-gray-600 p-2 rounded text-white w-full" placeholder="Descripción corta" bind:value={newDesc} />
    <textarea class="bg-gray-600 p-2 rounded text-white w-full h-24" placeholder="System Prompt (ej. Eres un experto en...)" bind:value={newPrompt}></textarea>
    
    <div class="flex gap-2">
        <select class="bg-gray-600 p-2 rounded text-white" bind:value={newModel}>
            <option value="chatgpt">ChatGPT</option>
            <option value="deepseek">DeepSeek</option>
            <option value="openai_api">OpenAI API</option>
        </select>
        <button class="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded font-bold text-white flex-1" on:click={createAgent}>
            + Crear Agente
        </button>
    </div>
    {#if message}
        <p class="text-sm text-yellow-400">{message}</p>
    {/if}
  </div>

  <ul class="space-y-2 max-h-48 overflow-y-auto">
    {#each agents as [id, name, desc, prompt, model, isBuiltIn]}
      <li class="flex justify-between items-center bg-gray-700 p-2 rounded">
        <div class="text-sm overflow-hidden">
            <div class="font-bold text-green-300">{name}</div>
            <div class="text-xs text-gray-400 truncate">{desc}</div>
        </div>
        {#if !isBuiltIn}
            <button class="text-red-400 hover:text-red-300 ml-2" on:click={() => deleteAgent(id, name)}>
                X
            </button>
        {:else}
            <span class="text-xs text-gray-500 italic">Built-in</span>
        {/if}
      </li>
    {/each}
  </ul>
</div>
