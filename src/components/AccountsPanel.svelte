<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let accounts: [number, string, string][] = []; // id, service, username
  let newService = "chatgpt";
  let newUsername = "";
  let newPassword = "";
  let message = "";

  async function checkUpdates() {
    try {
        message = "Buscando actualizaciones...";
        const res = await invoke("check_updates");
        message = res as string;
    } catch (e) {
        message = "Error actualizando: " + e;
    }
  }

  async function loadAccounts() {
    try {
      accounts = await invoke("get_accounts");
    } catch (e) {
      console.error("Failed to load accounts", e);
    }
  }

  async function addAccount() {
    if (!newUsername || !newPassword) {
        message = "Usuario y contraseña requeridos";
        return;
    }
    try {
      await invoke("add_account", { service: newService, username: newUsername, password: newPassword });
      message = "Cuenta añadida exitosamente";
      newUsername = "";
      newPassword = "";
      loadAccounts();
    } catch (e) {
      message = "Error: " + e;
    }
  }

  async function deleteAccount(service: string, username: string) {
    if (!confirm(`¿Borrar cuenta ${username} de ${service}?`)) return;
    try {
      await invoke("delete_account", { service, username });
      loadAccounts();
    } catch (e) {
        message = "Error borrando: " + e;
    }
  }

  onMount(() => {
    loadAccounts();
  });
</script>

<div class="bg-gray-800 p-4 rounded-lg shadow-md mt-4">
  <h2 class="text-xl font-bold mb-4 text-gray-200">Cuentas Vinculadas</h2>
  
  <div class="mb-4 space-y-2">
    <div class="flex gap-2">
        <select class="bg-gray-700 p-2 rounded text-white" bind:value={newService}>
            <option value="chatgpt">ChatGPT</option>
            <option value="deepseek">DeepSeek</option>
            <option value="openai_api">OpenAI API</option>
        </select>
        <input class="bg-gray-700 p-2 rounded text-white flex-1" placeholder="Usuario/Email" bind:value={newUsername} />
    </div>
    
    <div class="mt-4 border-t border-gray-600 pt-2">
        <button class="w-full bg-indigo-700 hover:bg-indigo-600 p-2 rounded text-sm text-white font-bold" on:click={checkUpdates}>
            Buscar Actualizaciones (OTA)
        </button>
    </div>
    <div class="flex gap-2">
        <input class="bg-gray-700 p-2 rounded text-white flex-1" type="password" placeholder="Contraseña" bind:value={newPassword} />
        <button class="bg-green-600 hover:bg-green-500 px-4 py-2 rounded font-bold text-white" on:click={addAccount}>
            + Añadir
        </button>
    </div>
    {#if message}
        <p class="text-sm text-yellow-400">{message}</p>
    {/if}
  </div>

  <ul class="space-y-2">
    {#each accounts as [id, service, username]}
      <li class="flex justify-between items-center bg-gray-700 p-2 rounded">
        <div class="text-sm">
            <span class="font-bold capitalize text-blue-300">{service}</span>
            <span class="text-gray-300 ml-2">{username}</span>
        </div>
        <button class="text-red-400 hover:text-red-300" on:click={() => deleteAccount(service, username)}>
            X
        </button>
      </li>
    {/each}
  </ul>
</div>
