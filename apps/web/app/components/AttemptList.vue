<script setup lang="ts">
import type { LegacyAttempt as Attempt } from "~/types/game";

defineProps<{ attempts: Attempt[] }>();
</script>

<template>
  <section class="rounded-md border border-black/12 bg-white/68 p-4">
    <div class="flex items-center justify-between"><p class="text-[10px] font-bold uppercase tracking-[.12em] text-black/45">Tus intentos</p><span class="font-mono text-[10px] text-black/40">{{ attempts.length }} palabras</span></div>
    <ol class="mt-3 divide-y divide-black/8">
      <li v-for="attempt in [...attempts].reverse()" :key="attempt.createdAt" class="flex items-center gap-3 py-2.5">
        <span class="grid h-6 w-6 place-items-center rounded-full border border-black/12 font-mono text-[9px] text-black/48">{{ attempt.rank }}</span>
        <strong class="min-w-0 flex-1 truncate text-sm font-semibold">{{ attempt.word }}</strong>
        <span class="h-1.5 w-16 overflow-hidden rounded-full bg-black/8"><i class="block h-full bg-moss" :style="{ width: `${attempt.similarity}%` }" /></span>
        <span class="w-9 text-right font-mono text-xs text-black/58">{{ attempt.similarity }}%</span>
      </li>
    </ol>
  </section>
</template>
