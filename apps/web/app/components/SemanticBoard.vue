<script setup lang="ts">
import type {
	LegacyAttempt as Attempt,
	SemanticConnection,
} from "~/types/game";

const props = defineProps<{
	attempts: Attempt[];
	connections: SemanticConnection[];
	solved: boolean;
}>();
const visibleWords = computed(() => props.attempts.slice(-4));
function position(index: number): { x: number; y: number } {
	const positions = [
		{ x: 138, y: 255 },
		{ x: 310, y: 137 },
		{ x: 518, y: 226 },
		{ x: 700, y: 118 },
	];
	return positions[index] ?? { x: 138, y: 255 };
}
</script>

<template>
  <section class="semantic-field relative min-h-[310px] overflow-hidden rounded-md border border-black/12 sm:min-h-[410px]" aria-label="Camino semantico actual">
    <div class="paper-grid absolute inset-0 opacity-70" />
    <svg viewBox="0 0 820 410" class="absolute inset-0 h-full w-full" role="img" aria-label="Conexiones entre tus palabras">
      <path v-for="(connection, index) in connections" :key="`${connection.from}-${connection.to}`" :d="`M ${position(index).x} ${position(index).y} C ${position(index).x + 80} ${position(index).y - 75}, ${position(index + 1).x - 80} ${position(index + 1).y + 72}, ${position(index + 1).x} ${position(index + 1).y}`" fill="none" :stroke="connection.strength > .8 ? '#718a5c' : '#93958c'" :stroke-width="1.4 + connection.strength * 2" stroke-linecap="round" opacity=".68" />
      <g v-for="(attempt, index) in visibleWords" :key="attempt.createdAt" :transform="`translate(${position(index).x} ${position(index).y})`">
        <circle r="36" fill="#fffef9" stroke="#252620" stroke-opacity=".2" />
        <circle v-if="index === visibleWords.length - 1" class="pulse-ring" r="42" fill="none" stroke="#cb5a3d" stroke-width="1" />
        <text y="4" text-anchor="middle" fill="#252620" font-family="ui-sans-serif, system-ui" font-size="14" font-weight="600">{{ attempt.word }}</text>
        <text y="57" text-anchor="middle" fill="#5d625b" font-family="ui-monospace, monospace" font-size="10">{{ attempt.similarity }}% cerca</text>
      </g>
      <g transform="translate(700 286)"><circle r="36" :fill="solved ? '#cb5a3d' : '#252620'" /><text y="4" text-anchor="middle" fill="#fffef9" font-family="ui-sans-serif, system-ui" font-size="14" font-weight="600">{{ solved ? 'resuelto' : '?' }}</text><text y="57" text-anchor="middle" fill="#5d625b" font-family="ui-monospace, monospace" font-size="10">objetivo</text></g>
    </svg>
    <div class="absolute left-4 top-4 flex items-center gap-3 text-[10px] text-black/45"><span class="flex items-center gap-1.5"><i class="h-2 w-2 rounded-full bg-graphite" /> tus ideas</span><span class="flex items-center gap-1.5"><i class="h-2 w-2 rounded-full bg-coral" /> objetivo</span></div>
    <p class="absolute bottom-4 left-4 right-4 text-center text-xs text-black/48">Cada palabra abre una posible ruta. La proximidad no siempre muestra el camino completo.</p>
  </section>
</template>
