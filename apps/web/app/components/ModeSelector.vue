<script setup lang="ts">
import { ArrowUpRight, LockKeyhole } from "lucide-vue-next";
import type { GameModeCard } from "~/types/game";

defineProps<{ modes: GameModeCard[] }>();
const emit = defineEmits<{ select: [id: string] }>();
const accentClasses: Record<GameModeCard["accent"], string> = {
	ink: "bg-graphite text-white",
	coral: "bg-coral text-white",
	sage: "bg-sage text-ink",
	moss: "bg-moss text-white",
};
</script>

<template>
  <section><div class="mb-4 flex items-end justify-between"><div><p class="text-[10px] font-bold uppercase tracking-[.12em] text-black/45">Elige como jugar</p><h2 class="mt-1 text-lg font-semibold tracking-tight">Otros modos</h2></div><span class="text-[11px] text-black/42">La racha solo cuenta en el diario</span></div>
    <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      <button v-for="mode in modes" :key="mode.id" class="group min-h-40 rounded-md border border-black/12 bg-white/72 p-4 text-left transition hover:-translate-y-0.5 hover:border-black/28 hover:bg-white disabled:cursor-not-allowed disabled:opacity-55" :disabled="!mode.available" type="button" @click="emit('select', mode.id)">
        <span class="mb-7 grid h-8 w-8 place-items-center rounded-sm" :class="accentClasses[mode.accent]"><LockKeyhole v-if="!mode.available" :size="15" /><ArrowUpRight v-else :size="16" /></span>
        <strong class="block text-sm">{{ mode.title }}</strong><span class="mt-1.5 block text-xs leading-5 text-black/52">{{ mode.description }}</span>
      </button>
    </div>
  </section>
</template>
