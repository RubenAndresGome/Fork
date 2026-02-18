<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import StatsPanel from "../components/StatsPanel.svelte";
  import AccountsPanel from "../components/AccountsPanel.svelte";
  import MessageDisplay from "../components/MessageDisplay.svelte";
  import AgentsPanel from "../components/AgentsPanel.svelte";
  import AgentCard from "../components/AgentCard.svelte";
  import AgentEditor from "../components/AgentEditor.svelte";
  import RagPanel from "../components/RagPanel.svelte";

  // UI State
  let messages: { role: string; content: string }[] = [];
  let prompt = "";
  let selectedModel = "auto";
  let activeTab = "chat"; // 'chat' | 'agents' | 'settings' | 'rag'

  // Agentes
  let agents: any[] = [];
  let selectedAgentId: number | null = null;

  // Agent Editor State
  let isEditingAgent = false;
  let editingAgentData = null; // null = create

  // Voice State
  let isVoiceEnabled = false;
  let isRagEnabled = false;

  // Model Options
  const models = [
    // ...
  ];

  // ...

  async function sendPrompt() {
    if (!prompt) return;

    // Añadir mensaje del usuario
    messages = [...messages, { role: "user", content: prompt }];
    const currentPrompt = prompt;
    prompt = ""; // Clear immediately

    try {
      const response = await invoke("send_prompt", {
        prompt: currentPrompt,
        model: selectedModel,
        agentId: selectedAgentId,
        useSearch: isRagEnabled,
        collection: "default",
      });
      messages = [...messages, { role: "system", content: response as string }];
      speak(response as string);
    } catch (e) {
      console.error(e);
      messages = [
        ...messages,
        { role: "error", content: "Error sending prompt: " + e },
      ];
    }
  }

  async function loadLocalModel() {
    try {
      messages = [
        ...messages,
        {
          role: "system",
          content: "Cargando modelo local... (esto puede tardar)",
        },
      ];
      const res = await invoke("load_local_model");
      messages = [...messages, { role: "system", content: res as string }];
    } catch (e) {
      messages = [
        ...messages,
        { role: "error", content: "Error cargando modelo: " + e },
      ];
    }
  }

  async function unloadLocalModel() {
    try {
      const res = await invoke("unload_local_model");
      messages = [...messages, { role: "system", content: res as string }];
    } catch (e) {
      messages = [...messages, { role: "error", content: "Error: " + e }];
    }
  }

  function selectAgent(agent: any) {
    selectedAgentId = agent[0];
    activeTab = "chat";
  }

  function openAgentEditor(agent: any = null) {
    editingAgentData = agent;
    isEditingAgent = true;
  }

  function closeAgentEditor() {
    isEditingAgent = false;
    editingAgentData = null;
    // Trigger refresh of agents via AgentsPanel if possible, or reload page?
    // Ideally AgentsPanel should refetch. For now we rely on Svelte reactivity if we can trigger it.
    // Or just invoke get_agents again.
    // A hack: toggle activeTab to force re-render or similar?
    // Actually AgentsPanel fetches on mount.
    // Let's reload the window for now or add a refresh function.
    window.location.reload();
  }

  async function showTelemetry() {
    try {
      const logs = await invoke("get_telemetry_log");
      console.log(logs);
      // For MVP, just dumping to a system message
      messages = [
        ...messages,
        { role: "system", content: "Telemetry Logs:\n" + logs },
      ];
    } catch (e) {
      console.error(e);
    }
  }
</script>

<main
  class="h-screen w-screen flex bg-gray-900 text-gray-100 overflow-hidden font-sans"
>
  <!-- LEFT PANE: Command Center (Navigation & Assets) -->
  <aside
    class="w-64 flex flex-col border-r border-gray-800 bg-gray-900/95 glass-panel z-10"
  >
    <!-- Header -->
    <div class="p-4 border-b border-gray-800">
      <h1
        class="text-xl font-bold tracking-tight bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent"
      >
        ANTIGRAVITY
      </h1>
      <p class="text-[10px] text-gray-500 uppercase tracking-widest mt-1">
        Mission Control
      </p>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 overflow-y-auto p-2 space-y-6">
      <!-- Stats Module -->
      <div class="space-y-2">
        <h2 class="text-xs font-semibold text-gray-500 px-2 uppercase">
          System Status
        </h2>
        <StatsPanel />
      </div>

      <!-- Knowledge Module -->
      <div class="space-y-2">
        <h2 class="text-xs font-semibold text-gray-500 px-2 uppercase">
          Knowledge
        </h2>
        <button
          class={`w-full justify-start text-left p-2 rounded text-sm transition-colors mx-2 ${activeTab === "rag" ? "bg-purple-600/20 text-purple-200 border border-purple-500/30" : "hover:bg-gray-800 text-gray-400"}`}
          on:click={openRag}
        >
          🧠 Knowledge Base
        </button>
      </div>

      <!-- Agents Module -->
      <div class="space-y-2">
        <div class="flex items-center justify-between px-2">
          <h2 class="text-xs font-semibold text-gray-500 uppercase">
            Active Agents
          </h2>
          <button
            class="text-xs text-blue-400 hover:text-blue-300 transition-colors"
            on:click={() => openAgentEditor(null)}>+ NEW</button
          >
        </div>

        <div class="flex flex-col gap-2 px-2">
          <button
            class={`justify-start text-left p-2 rounded text-sm transition-colors ${selectedAgentId === null ? "bg-blue-600/20 text-blue-200 border border-blue-500/30" : "hover:bg-gray-800 text-gray-400"}`}
            on:click={() => (selectedAgentId = null)}
          >
            ✦ Manual Override (Chat)
          </button>
          {#each agents as agent}
            <div class="group relative">
              <button
                class={`w-full justify-start text-left p-2 rounded text-sm transition-colors ${selectedAgentId === agent[0] ? "bg-blue-600/20 text-blue-200 border border-blue-500/30" : "hover:bg-gray-800 text-gray-400"}`}
                on:click={() => selectAgent(agent)}
              >
                ⚡ {agent[1]}
              </button>
              {#if !agent[5]}
                <!-- Only edit custom agents (is_built_in = false) -->
                <button
                  class="absolute right-2 top-2 p-0.5 text-gray-500 hover:text-white opacity-0 group-hover:opacity-100 transition-all bg-gray-900/50 rounded"
                  on:click|stopPropagation={() => openAgentEditor(agent)}
                  title="Edit Agent"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    class="w-3 h-3"
                  >
                    <path
                      d="m2.695 14.762-1.262 3.155a.5.5 0 0 0 .65.65l3.155-1.262a4 4 0 0 0 1.343-.886L17.5 5.501a2.121 2.121 0 0 0-3-3L3.58 13.419a4 4 0 0 0-.885 1.343Z"
                    />
                  </svg>
                </button>
              {/if}
            </div>
          {/each}
        </div>
        <!-- Hidden real logic for agents fetching, visually handled above -->
        <div class="hidden"><AgentsPanel on:update={handleAgentsUpdate} /></div>
      </div>
    </nav>

    <!-- Footer / Accounts -->
    <div class="p-2 border-t border-gray-800 bg-gray-900/50">
      <AccountsPanel />
    </div>
  </aside>

  <!-- CENTER PANE: Stage (Chat / Editor) -->
  <section
    class="flex-1 flex flex-col relative z-0 bg-gradient-to-br from-gray-900 to-gray-800"
  >
    <!-- Toolbar -->
    <div
      class="h-14 border-b border-gray-700/50 flex items-center px-6 justify-between glass"
    >
      <div class="flex items-center gap-4">
        <span class="text-gray-400 text-sm">Target Model:</span>
        <select
          class="bg-gray-800 border border-gray-600 text-gray-200 text-sm rounded px-3 py-1 focus:ring-2 focus:ring-blue-500 outline-none"
          bind:value={selectedModel}
        >
          {#each models as m}
            <option value={m.id}>{m.name}</option>
          {/each}
        </select>
      </div>

      <div class="flex items-center gap-2">
        {#if selectedModel === "local_phi2"}
          <button
            class="text-xs bg-gray-700 hover:bg-gray-600 px-3 py-1 rounded transition-colors"
            on:click={loadLocalModel}>Load</button
          >
          <button
            class="text-xs bg-red-900/50 hover:bg-red-800/50 text-red-200 px-3 py-1 rounded transition-colors"
            on:click={unloadLocalModel}>Unload</button
          >
        {/if}
        <div
          class="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]"
        ></div>
        <span class="text-xs text-green-400 font-mono">ONLINE</span>
      </div>
    </div>

    {#if isEditingAgent}
      <AgentEditor
        agent={editingAgentData}
        {models}
        on:cancel={closeAgentEditor}
        on:save={closeAgentEditor}
      />
    {:else if activeTab === "rag"}
      <RagPanel on:close={() => (activeTab = "chat")} />
    {:else}
      <!-- Chat Area -->
      <div
        class="flex-1 overflow-y-auto p-4 space-y-4 scroll-smooth"
        id="chat-container"
      >
        {#if messages.length === 0}
          <div
            class="h-full flex flex-col items-center justify-center text-gray-600 space-y-4"
          >
            <div
              class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center"
            >
              <span class="text-2xl">🌌</span>
            </div>
            <p>Ready to defy limits.</p>
          </div>
        {/if}

        {#each messages as msg}
          <!-- Message Item -->
          <div
            class={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
          >
            <div
              class={`max-w-[80%] rounded-2xl p-4 ${msg.role === "user" ? "bg-blue-600 text-white" : "bg-gray-800 text-gray-200 shadow-sm"}`}
            >
              {#if msg.role === "system" || msg.role === "error"}
                <div class="text-xs font-mono opacity-70 mb-1 uppercase">
                  {msg.role}
                </div>
              {/if}
              <div
                class="markdown-body text-sm leading-relaxed whitespace-pre-wrap"
              >
                {msg.content}
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Input Area -->
      <div
        class="p-4 bg-gray-900/50 backdrop-blur-md border-t border-gray-800 z-20"
      >
        <div class="relative max-w-4xl mx-auto">
          <textarea
            bind:value={prompt}
            on:keydown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                sendPrompt();
              }
            }}
            placeholder="Construct reality..."
            class="w-full bg-gray-800/80 border border-gray-700 text-gray-100 rounded-xl px-4 py-3 pr-12 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all resize-none shadow-lg"
            rows="1"
          ></textarea>
          <button
            on:click={() => (isRagEnabled = !isRagEnabled)}
            class={`absolute right-20 top-2 p-1.5 rounded-lg transition-colors ${isRagEnabled ? "bg-purple-600/20 text-purple-400" : "text-gray-500 hover:text-gray-300"}`}
            title="Toggle RAG (Context)"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              class="w-5 h-5"
            >
              <path
                d="M10 2a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0v-1.5A.75.75 0 0 1 10 2ZM10 15a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0v-1.5A.75.75 0 0 1 10 15ZM10 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6ZM15.657 5.757a.75.75 0 0 0-1.06-1.06l-1.061 1.06a.75.75 0 0 0 1.06 1.06l1.06-1.06ZM6.464 14.95a.75.75 0 0 0-1.06-1.06l-1.06 1.06a.75.75 0 0 0 1.06 1.06l1.06-1.06ZM16.25 10a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75ZM2.25 10a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5H3a.75.75 0 0 1-.75-.75ZM6.464 5.05a.75.75 0 0 0-1.06 1.06l1.06 1.06a.75.75 0 0 0 1.06-1.06l-1.06-1.06ZM15.657 14.243a.75.75 0 0 0-1.06 1.06l1.06 1.06a.75.75 0 0 0 1.06-1.06l-1.06-1.06Z"
              />
            </svg>
          </button>
          <button
            on:click={() => (isVoiceEnabled = !isVoiceEnabled)}
            class={`absolute right-12 top-2 p-1.5 rounded-lg transition-colors ${isVoiceEnabled ? "bg-green-600/20 text-green-400" : "text-gray-500 hover:text-gray-300"}`}
            title="Toggle Text-to-Speech"
          >
            {#if isVoiceEnabled}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                class="w-5 h-5"
              >
                <path
                  d="M10 3a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3ZM10 8.5a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3ZM11.5 15.5a1.5 1.5 0 1 0-3 0 1.5 1.5 0 0 0 3 0Z"
                />
                <path
                  fill-rule="evenodd"
                  d="M2.25 5A2.75 2.75 0 0 1 5 2.25h10A2.75 2.75 0 0 1 17.75 5v10A2.75 2.75 0 0 1 15 17.75H5A2.75 2.75 0 0 1 2.25 15V5Zm2.75-1.25a1.25 1.25 0 0 0-1.25 1.25v10c0 .69.56 1.25 1.25 1.25h10c.69 0 1.25-.56 1.25-1.25V5a1.25 1.25 0 0 0-1.25-1.25H5Z"
                  clip-rule="evenodd"
                />
                <path
                  d="M9.547 3.064A1.5 1.5 0 0 1 10.5 4.5v11a1.5 1.5 0 0 1-2.923.475l-2.09-5.186a1.5 1.5 0 0 0-2.434-1.378l-.5.35a1.5 1.5 0 0 1-2.296-1.636l1.205-3.86a2.5 2.5 0 0 1 2.05-1.72l2.355-.37a2.5 2.5 0 0 1 2.458 1.488l1.222-1.6z"
                />
                <!-- Using a simpler speaker icon -->
                <path
                  fill-rule="evenodd"
                  d="M9.53 2.29a.75.75 0 0 1 .58.12l.13.09a3.2 3.2 0 0 1 1.23 4.22l.62.2a2.3 2.3 0 0 1-1.33 4.31l-.22.02a.75.75 0 0 1-.76-.87.75.75 0 0 1 .63-.64l.11-.01a.8.8 0 0 0 .46-1.5l-.6-.2a.75.75 0 0 1-.49-.93.75.75 0 0 1 .83-.5c.34.07.69.04 1-.09a1.7 1.7 0 0 0 .66-2.27l-.09-.16a1.7 1.7 0 0 0-2.34-.63.75.75 0 0 1-1.03-.4.75.75 0 0 1 .4-.93l.21-.08ZM5.5 13.5H4a2.5 2.5 0 0 1-2.5-2.5v-2a2.5 2.5 0 0 1 2.5-2.5h1.5v7Z"
                  clip-rule="evenodd"
                />
                <path
                  fill-rule="evenodd"
                  d="M8.28 1.47a.75.75 0 0 1 .22 1.03A9 9 0 1 0 16 11a9 9 0 0 0-7.5-8.87.75.75 0 0 1 .24-1.48A10.5 10.5 0 1 1 7.25 10.2a.75.75 0 1 1-1.5.09A9 9 0 0 0 8.28 1.47Z"
                  clip-rule="evenodd"
                />
              </svg>
              <!-- Just a simple speaker icon -->
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                class="w-5 h-5"
              >
                <path
                  d="M13.5 4.06c0-1.336-1.616-2.005-2.56-1.06l-4.5 4.5H4.508c-1.141 0-2.318.664-2.66 1.905A9.76 9.76 0 0 0 1.5 12c0 2.485.62 4.196 1.75 4.618a1 1 0 0 0 .25.032h1.498l4.5 4.5c.945.945 2.56.276 2.56-1.06V4.06ZM18.584 5.106a.75.75 0 0 1 1.06 0c3.808 3.807 3.808 9.98 0 13.788a.75.75 0 1 1-1.06-1.06 8.25 8.25 0 0 0 0-11.668.75.75 0 0 1 0-1.06Z"
                />
                <path
                  d="M15.932 7.757a.75.75 0 0 1 1.061 0 6 6 0 0 1 0 8.486.75.75 0 0 1-1.06-1.061 4.5 4.5 0 0 0 0-6.364.75.75 0 0 1 0-1.06Z"
                />
              </svg>
            {:else}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-5"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M19.114 5.636a9 9 0 0 1 0 12.728M16.463 8.288a5.25 5.25 0 0 1 0 7.424M6.75 8.25l4.72-4.72a.75.75 0 0 1 1.28.53v15.88a.75.75 0 0 1-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.009 9.009 0 0 1 2.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75Z"
                />
              </svg>
            {/if}
          </button>
          <button
            on:click={sendPrompt}
            class="absolute right-2 top-2 p-1.5 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-colors shadow-lg shadow-blue-500/20"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="currentColor"
              class="w-5 h-5"
            >
              <path
                d="M3.478 2.404a.75.75 0 0 0-.926.941l2.432 7.905H13.5a.75.75 0 0 1 0 1.5H4.984l-2.432 7.905a.75.75 0 0 0 .926.94 60.519 60.519 0 0 0 18.445-8.986.75.75 0 0 0 0-1.218A60.517 60.517 0 0 0 3.478 2.404Z"
              />
            </svg>
          </button>
        </div>
        <div class="flex justify-center mt-2 gap-4 text-xs text-gray-500">
          <span>{selectedModel}</span>
          <span>•</span>
          <span>{messages.length} messages</span>
        </div>
      </div>
    {/if}
  </section>

  <!-- RIGHT PANE: Inspector (Details & History) -->
  <aside
    class="w-72 border-l border-gray-800 bg-gray-900/95 glass-panel hidden lg:flex flex-col"
  >
    <div class="p-4 border-b border-gray-800">
      <h2 class="text-sm font-bold text-gray-400 uppercase">Inspector</h2>
    </div>
    <div class="p-4 space-y-4 overflow-y-auto flex-1">
      <!-- Active Model Info -->
      <div class="bg-gray-800/50 rounded p-3 border border-gray-700">
        <h3 class="text-xs text-gray-500 uppercase mb-2">Active Protocol</h3>
        <div class="flex items-center gap-2">
          <div
            class={`w-3 h-3 rounded-full bg-${models.find((m) => m.id === selectedModel)?.color || "gray"}-500`}
          ></div>
          <span class="font-medium text-sm"
            >{models.find((m) => m.id === selectedModel)?.name}</span
          >
        </div>
        <p class="text-xs text-gray-500 mt-1">
          Provider: {models.find((m) => m.id === selectedModel)?.provider}
        </p>
      </div>

      <!-- Quick Actions -->
      <div>
        <h3 class="text-xs text-gray-500 uppercase mb-2">Quick Actions</h3>
        <div class="grid grid-cols-2 gap-2">
          <button
            class="bg-gray-800 hover:bg-gray-700 border border-gray-700 p-2 rounded text-xs text-gray-300 transition-colors"
            >Clean Context</button
          >
          <button
            class="bg-gray-800 hover:bg-gray-700 border border-gray-700 p-2 rounded text-xs text-gray-300 transition-colors"
            >Export Log</button
          >
          <button
            class="bg-gray-800 hover:bg-gray-700 border border-gray-700 p-2 rounded text-xs text-gray-300 transition-colors col-span-2"
            on:click={showTelemetry}>View Telemetry Logs</button
          >
        </div>
      </div>
    </div>
  </aside>
</main>
