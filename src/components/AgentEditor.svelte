<script>
    import { createEventDispatcher } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    export let agent = null; // null = create new, Object = edit
    export let models = [];

    const dispatch = createEventDispatcher();

    let name = "";
    let description = "";
    let system_prompt = "";
    let default_model = "chatgpt";
    let errorMessage = "";

    $: if (agent) {
        name = agent[1];
        description = agent[2];
        system_prompt = agent[3];
        default_model = agent[4];
    } else {
        name = "";
        description = "";
        system_prompt = "";
        default_model = "chatgpt";
    }

    async function saveAgent() {
        if (!name || !system_prompt) {
            errorMessage = "Name and System Prompt are required.";
            return;
        }

        try {
            if (agent) {
                // Edit
                await invoke("update_agent", {
                    id: agent[0],
                    name,
                    description,
                    systemPrompt: system_prompt,
                    defaultModel: default_model,
                });
            } else {
                // Create
                await invoke("create_agent", {
                    name,
                    description,
                    systemPrompt: system_prompt,
                    defaultModel: default_model,
                });
            }
            dispatch("save");
        } catch (e) {
            errorMessage = e;
        }
    }

    function cancel() {
        dispatch("cancel");
    }

    async function deleteAgent() {
        if (!agent) return;
        if (!confirm("Are you sure you want to delete this agent?")) return;

        try {
            await invoke("delete_agent", { id: agent[0] });
            dispatch("save"); // re-fetch list
        } catch (e) {
            errorMessage = e;
        }
    }
</script>

<div
    class="p-6 bg-gray-900/95 glass-panel h-full flex flex-col overflow-y-auto"
>
    <h2 class="text-2xl font-bold mb-6 text-white">
        {agent ? "Edit Agent" : "Create New Agent"}
    </h2>

    {#if errorMessage}
        <div
            class="bg-red-900/50 border border-red-500 text-red-200 p-3 rounded mb-4"
        >
            {errorMessage}
        </div>
    {/if}

    <div class="space-y-4">
        <div>
            <label class="block text-sm font-medium text-gray-400 mb-1"
                >Name</label
            >
            <input
                type="text"
                bind:value={name}
                placeholder="e.g., Python Expert"
                class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-400 mb-1"
                >Description</label
            >
            <input
                type="text"
                bind:value={description}
                placeholder="Brief description of capabilities"
                class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-400 mb-1"
                >Default Model</label
            >
            <select
                bind:value={default_model}
                class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
            >
                {#each models as model}
                    <option value={model.id}>{model.name}</option>
                {/each}
            </select>
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-400 mb-1"
                >System Prompt / Persona</label
            >
            <textarea
                bind:value={system_prompt}
                rows="8"
                placeholder="You are an expert in..."
                class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:ring-1 focus:ring-blue-500 font-mono text-sm"
            ></textarea>
            <p class="text-xs text-gray-500 mt-1">
                Define the agent's personality, constraints, and expertise here.
            </p>
        </div>

        <div class="flex justify-end gap-3 pt-4 border-t border-gray-700">
            {#if agent && !agent[5]}
                <!-- Only delete custom agents (is_built_in=0, index 5) Assuming boolean is returned as index 5-->
                <button
                    on:click={deleteAgent}
                    class="px-4 py-2 bg-red-900/30 hover:bg-red-900/50 text-red-300 rounded transition-colors mr-auto"
                >
                    Delete
                </button>
            {/if}

            <button
                on:click={cancel}
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded transition-colors"
            >
                Cancel
            </button>
            <button
                on:click={saveAgent}
                class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded transition-colors shadow-lg shadow-blue-900/20"
            >
                Save Agent
            </button>
        </div>
    </div>
</div>
