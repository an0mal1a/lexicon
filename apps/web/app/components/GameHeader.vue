<script setup lang="ts">
import {
	BarChart3,
	Compass,
	Flame,
	Gamepad2,
	HelpCircle,
} from "lucide-vue-next";

defineProps<{ active: "play" | "discover" | "stats"; streak: number }>();
const emit = defineEmits<{
	navigate: [view: "play" | "discover" | "stats"];
	help: [];
}>();
</script>

<template>
  <header class="mx-auto flex h-16 max-w-7xl items-center justify-between border-b border-black/12 px-4 sm:px-6">
    <button class="flex items-center gap-2.5" type="button" aria-label="Ir al reto diario" @click="emit('navigate', 'play')">
      <span class="grid h-7 w-7 place-items-center rounded-sm bg-graphite font-mono text-[10px] font-bold text-white">LX</span>
      <span class="text-sm font-bold tracking-tight">Lexicon</span>
    </button>
    <nav class="hidden items-center gap-1 sm:flex" aria-label="Navegacion del juego">
      <button class="rounded-sm px-3 py-2 text-xs transition" :class="active === 'play' ? 'bg-black/6 font-semibold text-ink' : 'text-black/48 hover:bg-black/4 hover:text-ink'" type="button" @click="emit('navigate', 'play')"><Gamepad2 class="mr-1.5 inline" :size="14" />Jugar</button>
      <button class="rounded-sm px-3 py-2 text-xs transition" :class="active === 'discover' ? 'bg-black/6 font-semibold text-ink' : 'text-black/48 hover:bg-black/4 hover:text-ink'" type="button" @click="emit('navigate', 'discover')"><Compass class="mr-1.5 inline" :size="14" />Modos</button>
      <button class="rounded-sm px-3 py-2 text-xs transition" :class="active === 'stats' ? 'bg-black/6 font-semibold text-ink' : 'text-black/48 hover:bg-black/4 hover:text-ink'" type="button" @click="emit('navigate', 'stats')"><BarChart3 class="mr-1.5 inline" :size="14" />Estadisticas</button>
    </nav>
    <div class="flex items-center gap-2">
      <span class="flex items-center gap-1.5 rounded-sm border border-black/10 bg-white/55 px-2.5 py-1.5 font-mono text-[10px] text-black/62"><Flame :size="14" class="text-coral" />{{ streak }} dias</span>
      <button class="grid h-8 w-8 place-items-center rounded-sm text-black/48 transition hover:bg-black/5 hover:text-ink" type="button" aria-label="Como jugar" @click="emit('help')"><HelpCircle :size="17" /></button>
    </div>
  </header>
</template>
