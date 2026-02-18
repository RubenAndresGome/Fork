<script lang="ts">
    export let name: string;
    export let description: string;
    export let model: string;
    export let active: boolean = false;
</script>

<div
    class={`p-3 rounded-lg border transition-all cursor-pointer ${active ? "bg-blue-600/20 border-blue-500 shadow-lg shadow-blue-500/10" : "bg-gray-800/50 border-gray-700 hover:border-gray-600"}`}
>
    <div class="flex justify-between items-start mb-1">
        <h3 class="font-bold text-sm text-gray-100">{name}</h3>
        <span
            class="text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-gray-900 text-gray-400 border border-gray-700"
            >{model}</span
        >
    </div>
    <p class="text-xs text-gray-400 line-clamp-2">{description}</p>
</div>
