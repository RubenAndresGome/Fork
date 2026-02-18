<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import StatsPanel from "../components/StatsPanel.svelte";
  import AccountsPanel from "../components/AccountsPanel.svelte";
  import MessageDisplay from "../components/MessageDisplay.svelte";
  import AgentsPanel from "../components/AgentsPanel.svelte";

  let prompt = "";
  let messages: { role: string; content: string }[] = [];
  let selectedModel = "chatgpt";
  
  // Agentes
  let agents: any[] = [];
  let selectedAgentId: number | null = null;

  function handleAgentsUpdate(event: CustomEvent) {
    agents = event.detail;
  }

  async function send() {
    if (!prompt) return;
    
    // Añadir mensaje del usuario
    messages = [...messages, { role: "user", content: prompt }];
    
    try {
        const response = await invoke("send_prompt", { 
            prompt, 
            model: selectedModel,
            agentId: selectedAgentId 
        });
        messages = [...messages, { role: "system", content: response as string }];
    } catch (e) {
        console.error(e);
        messages = [...messages, { role: "error", content: "Error sending prompt: " + e }];
    }
    
    prompt = "";
  }

  async function loadLocalModel() {
    try {
        messages = [...messages, { role: "system", content: "Cargando modelo local... (esto puede tardar)" }];
        const res = await invoke("load_local_model");
        messages = [...messages, { role: "system", content: res as string }];
    } catch (e) {
        messages = [...messages, { role: "error", content: "Error cargando modelo: " + e + ". Asegúrate de descargar 'phi-2-int4.onnx' y 'tokenizer.json' en la carpeta 'models' de AppData." }];
    }
  }
</script>

<main class="container mx-auto p-4 h-screen flex flex-row bg-gray-900 text-white gap-4">
  <!-- Sidebar -->
  <aside class="w-64 flex flex-col gap-4">
    <div class="bg-gray-800 p-4 rounded-lg">
      <h1 class="text-xl font-bold mb-4">CodeChat</h1>
      
      <label class="block text-xs text-gray-400 mb-1">
          Agente
          <select class="bg-gray-700 p-2 rounded w-full mb-2 mt-1" bind:value={selectedAgentId}>
              <option value={null}>Chat General</option>
              {#each agents as agent}
                <option value={agent[0]}>{agent[1]}</option>
              {/each}
          </select>
      </label>

      <label class="block text-xs text-gray-400 mb-1">
          Modelo
          <select class="bg-gray-700 p-2 rounded w-full mb-2 mt-1" bind:value={selectedModel}>
              <option value="deepseek">DeepSeek</option>
              <option value="chatgpt">ChatGPT</option>
              <option value="openai_api">OpenAI API ($)</option>
              <option value="local_phi2">Local Phi-2 (Offline)</option>
          </select>
      </label>
      
      {#if selectedModel === 'local_phi2'}
        <button 
            class="w-full bg-green-700 hover:bg-green-600 p-2 rounded text-sm font-bold mb-2"
            on:click={loadLocalModel}
        >
            Cargar/Verificar Modelo
        </button>
      {/if}
    </div>
    <StatsPanel />
    <AgentsPanel on:update={handleAgentsUpdate} />
    <AccountsPanel />
  </aside>

  <!-- Chat Area -->
  <div class="flex-1 flex flex-col">
    <div class="flex-1 overflow-y-auto mb-4 bg-gray-800 rounded p-4 space-y-4">
    {#each messages as msg}
      <div class={`p-3 rounded-lg max-w-[80%] ${msg.role === 'user' ? 'bg-blue-600 ml-auto' : 'bg-gray-700'}`}>
        <p class="text-sm font-semibold opacity-75 mb-1">{msg.role}</p>
        <MessageDisplay content={msg.content} />
      </div>
    {/each}
  </div>

  <form on:submit|preventDefault={send} class="flex gap-2">
    <input
      class="flex-1 p-2 rounded bg-gray-700 border border-gray-600 focus:outline-none focus:border-blue-500"
      bind:value={prompt}
      placeholder="Escribe tu mensaje..."
    />
    <button type="submit" class="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded font-bold">
      Enviar
    </button>
  </form>
  </div>
</main>
