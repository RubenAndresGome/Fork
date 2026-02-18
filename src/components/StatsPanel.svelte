<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let stats: [string, number, number][] = [];
  let loading = true;
  let totalCost = 0.0;

  async function loadStats() {
    try {
      // El comando devuelve Vec<(String, i64, f64)>
      stats = await invoke("get_stats");
      totalCost = stats.reduce((acc, [_, __, cost]) => acc + cost, 0);
    } catch (e) {
      console.error("Failed to load stats", e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadStats();
    // Actualizar cada 5 segundos
    const interval = setInterval(loadStats, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="bg-gray-800 p-4 rounded-lg shadow-md">
  <h2 class="text-xl font-bold mb-4 text-gray-200">Uso Diario</h2>
  {#if loading}
    <p class="text-gray-400">Cargando...</p>
  {:else if stats.length === 0}
    <p class="text-gray-400">Sin actividad hoy.</p>
  {:else}
    <div class="mb-4 text-green-400 font-bold text-lg">
        Gasto Est.: ${totalCost.toFixed(4)}
    </div>
    <ul class="space-y-2">
      {#each stats as [model, count, cost]}
        <li class="flex justify-between items-center text-gray-300">
          <span class="capitalize">{model}</span>
          <div class="flex gap-2">
            <span class="text-xs text-gray-500 mt-1">${cost.toFixed(4)}</span>
            <span class="bg-blue-600 px-2 py-1 rounded text-xs font-bold">{count}</span>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
