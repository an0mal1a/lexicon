<script setup lang="ts">
import { Flame, Target, Trophy } from "lucide-vue-next";
import type { PlayerStats } from "~/types/game";

defineProps<{ stats: PlayerStats }>();
</script>

<template>
  <section class="grid gap-4 lg:grid-cols-[1.1fr_.9fr]"><div class="rounded-md border border-black/12 bg-white/72 p-5"><p class="text-[10px] font-bold uppercase tracking-[.12em] text-black/45">Tu recorrido</p><div class="mt-5 grid grid-cols-3 divide-x divide-black/10"><div class="pr-4"><Flame :size="17" class="text-coral" /><strong class="mt-3 block font-mono text-3xl font-medium">{{ stats.currentStreak }}</strong><span class="text-[11px] text-black/50">racha actual</span></div><div class="px-4"><Trophy :size="17" class="text-moss" /><strong class="mt-3 block font-mono text-3xl font-medium">{{ stats.bestStreak }}</strong><span class="text-[11px] text-black/50">mejor racha</span></div><div class="pl-4"><Target :size="17" class="text-graphite" /><strong class="mt-3 block font-mono text-3xl font-medium">{{ Math.round(stats.gamesWon / stats.gamesPlayed * 100) }}%</strong><span class="text-[11px] text-black/50">retos ganados</span></div></div></div><div class="rounded-md border border-black/12 bg-[#eeeee8] p-5"><p class="text-[10px] font-bold uppercase tracking-[.12em] text-black/45">Esta semana</p><div class="mt-7 flex items-end justify-between gap-2"><div v-for="(won, index) in stats.recentDays" :key="index" class="flex flex-1 flex-col items-center gap-2"><i class="block h-9 w-full rounded-sm border border-black/10" :class="won ? 'bg-moss' : 'bg-white/55'" /><span class="font-mono text-[9px] text-black/42">{{ ['L','M','X','J','V','S','D'][index] }}</span></div></div><p class="mt-4 text-xs text-black/54">{{ stats.gamesWon }} retos resueltos · media de {{ stats.averageAttempts }} intentos</p></div></section>
</template>
