<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    const dispatch = createEventDispatcher();

    let documents: any[] = [];
    let collection = "default";
    let inputText = "";
    let filename = "";
    let isIngesting = false;
    let statusMessage = "";
    let searchTestQuery = "";
    let searchResults: any[] = [];

    onMount(() => {
        loadDocuments();
    });

    async function loadDocuments() {
        try {
            documents = await invoke("get_documents", { collection });
        } catch (e) {
            console.error(e);
        }
    }

    async function ingest() {
        if (!inputText || !filename) {
            statusMessage = "Please provide both filename and content.";
            return;
        }
        isIngesting = true;
        try {
            await invoke("ingest_document", {
                collection,
                filename,
                content: inputText,
            });
            statusMessage = "Ingested successfully!";
            inputText = "";
            filename = "";
            loadDocuments();
        } catch (e) {
            statusMessage = "Error: " + e;
        } finally {
            isIngesting = false;
        }
    }

    async function testSearch() {
        if (!searchTestQuery) return;
        try {
            searchResults = await invoke("rag_search", {
                collection,
                query: searchTestQuery,
            });
        } catch (e) {
            console.error(e);
        }
    }
</script>

<div
    class="p-6 bg-gray-900/95 glass-panel h-full flex flex-col overflow-y-auto"
>
    <div class="flex justify-between items-center mb-6">
        <h2 class="text-2xl font-bold text-white flex items-center gap-2">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6 text-purple-400"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25"
                />
            </svg>
            Knowledge Base (RAG)
        </h2>
        <button
            class="text-gray-400 hover:text-white"
            on:click={() => dispatch("close")}>✖</button
        >
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Ingestion Column -->
        <div class="space-y-4">
            <div class="bg-gray-800/50 p-4 rounded-lg border border-gray-700">
                <h3 class="text-sm font-semibold text-gray-400 uppercase mb-2">
                    Ingest Document
                </h3>
                <input
                    type="text"
                    bind:value={filename}
                    placeholder="Filename (e.g., policy.txt)"
                    class="w-full bg-gray-900 text-white rounded p-2 mb-2 border border-gray-700 focus:border-purple-500 outline-none"
                />
                <textarea
                    bind:value={inputText}
                    rows="6"
                    placeholder="Paste document content here..."
                    class="w-full bg-gray-900 text-white rounded p-2 border border-gray-700 focus:border-purple-500 outline-none text-sm font-mono"
                ></textarea>
                <div class="flex justify-between items-center mt-2">
                    <span class="text-xs text-orange-400">{statusMessage}</span>
                    <button
                        on:click={ingest}
                        disabled={isIngesting}
                        class="bg-purple-600 hover:bg-purple-500 text-white px-4 py-2 rounded font-medium disabled:opacity-50 transition-colors"
                    >
                        {isIngesting ? "Ingesting..." : "Add to Knowledge Base"}
                    </button>
                </div>
            </div>

            <div class="bg-gray-800/50 p-4 rounded-lg border border-gray-700">
                <h3 class="text-sm font-semibold text-gray-400 uppercase mb-2">
                    Search Sandbox
                </h3>
                <div class="flex gap-2">
                    <input
                        type="text"
                        bind:value={searchTestQuery}
                        placeholder="Test query..."
                        class="flex-1 bg-gray-900 text-white rounded p-1 border border-gray-700"
                        on:keydown={(e) => e.key === "Enter" && testSearch()}
                    />
                    <button
                        on:click={testSearch}
                        class="bg-gray-700 hover:bg-gray-600 text-white px-3 rounded"
                        >Test</button
                    >
                </div>
                {#if searchResults.length > 0}
                    <div class="mt-2 space-y-2 max-h-40 overflow-y-auto">
                        {#each searchResults as result}
                            <div class="text-xs bg-black/30 p-2 rounded">
                                <div class="font-bold text-purple-300">
                                    {result.filename}
                                </div>
                                <div class="text-gray-400 truncate">
                                    {result.content}
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>

        <!-- List Column -->
        <div
            class="bg-gray-800/50 p-4 rounded-lg border border-gray-700 h-full flex flex-col"
        >
            <h3 class="text-sm font-semibold text-gray-400 uppercase mb-2">
                Stored Documents ({collection})
            </h3>
            <div class="flex-1 overflow-y-auto space-y-2">
                {#if documents.length === 0}
                    <div class="text-gray-500 text-center italic mt-10">
                        No documents found.
                    </div>
                {/if}
                {#each documents as doc}
                    <div
                        class="p-2 bg-gray-900 rounded border border-gray-700 flex justify-between items-center"
                    >
                        <div>
                            <div class="text-sm font-medium text-gray-200">
                                {doc[1]}
                            </div>
                            <!-- filename -->
                            <div class="text-xs text-gray-500">
                                {new Date(doc[2]).toLocaleString()}
                            </div>
                        </div>
                        <div class="text-xs text-gray-600">ID: {doc[0]}</div>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</div>
