<script lang="ts">
  import CodeBlock from "./CodeBlock.svelte";

  export let content = "";

  // Estructura simple para el parseo
  type Part = { type: 'text', content: string } | { type: 'code', lang: string, content: string };
  
  let parts: Part[] = [];

  $: {
    parts = [];
    const regex = /```(\w+)?\n([\s\S]*?)```/g;
    let lastIndex = 0;
    let match;

    while ((match = regex.exec(content)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ type: 'text', content: content.slice(lastIndex, match.index) });
      }
      parts.push({ type: 'code', lang: match[1] || 'text', content: match[2] });
      lastIndex = regex.lastIndex;
    }

    if (lastIndex < content.length) {
      parts.push({ type: 'text', content: content.slice(lastIndex) });
    }
  }
</script>

<div>
  {#each parts as part}
    {#if part.type === 'text'}
      <p class="whitespace-pre-wrap mb-2">{part.content}</p>
    {:else}
      <CodeBlock language={part.lang} code={part.content} />
    {/if}
  {/each}
</div>
