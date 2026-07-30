<script setup lang="ts">
import {
	Lightbulb,
	LoaderCircle,
	Send,
	Share2,
	Trophy,
	WifiOff,
	X,
} from "lucide-vue-next";
import { useGame } from "~/stores/useGame";

const game = useGame();
const word = ref("");
const showResult = ref(true);
const copied = ref(false);

onMounted(() => game.initialize());

async function submit(): Promise<void> {
	const value = word.value.trim();
	if (!value) return;
	await game.submit(value);
	word.value = "";
}

async function share(): Promise<void> {
	if (!game.game.value) return;
	const attempts = game.game.value.attempts;
	const tiles = attempts
		.map((attempt) => {
			if (attempt.is_exact) return "🟩";
			if (attempt.score >= 0.7) return "🟨";
			return "🟧";
		})
		.join("");
	const text = `Lexicon #${game.game.value.challenge_id}\n${tiles}\n${attempts.length} intentos · 🔥 ${game.stats.value?.current_streak ?? 0}`;
	try {
		if (navigator.share) await navigator.share({ text });
		else await navigator.clipboard.writeText(text);
		copied.value = true;
		window.setTimeout(() => {
			copied.value = false;
		}, 1800);
	} catch {
		// Sharing is optional; a cancelled native share must not show an error state.
	}
}

function percent(value: number): string {
	return `${Math.round(value * 100)}%`;
}
</script>

<template>
  <div class="paper-grid min-h-screen bg-paper">
    <GameHeader active="play" :streak="game.stats.value?.current_streak ?? 0" @navigate="() => undefined" @help="() => undefined" />

    <main class="mx-auto max-w-6xl px-4 py-8 sm:px-6 sm:py-12">
      <section v-if="game.loading.value" class="grid min-h-[55vh] place-items-center">
        <div class="text-center text-black/55"><LoaderCircle class="mx-auto animate-spin text-moss" :size="28" /><p class="mt-3 text-sm">Cargando el reto desde LXDB…</p></div>
      </section>

      <section v-else-if="game.error.value" class="mx-auto max-w-lg rounded-md border border-coral/30 bg-[#f8e5df] p-6 text-center">
        <WifiOff class="mx-auto text-coral" :size="28" /><h1 class="mt-3 text-xl font-semibold">No se puede iniciar la partida</h1><p class="mt-2 text-sm leading-6 text-black/62">{{ game.error.value }}</p>
        <button class="mt-5 rounded-sm bg-graphite px-4 py-2 text-sm font-semibold text-white transition hover:bg-black" type="button" @click="game.initialize">Reintentar</button>
      </section>

      <section v-else-if="game.game.value" class="animate-in fade-in duration-300">
        <div class="mb-7 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div><p class="text-[10px] font-bold uppercase tracking-[.14em] text-black/46">Reto diario · {{ game.challenge.value?.date }}</p><h1 class="mt-2 text-3xl font-semibold tracking-tight sm:text-4xl">Encuentra el nexo.</h1><p class="mt-2 max-w-xl text-sm leading-6 text-black/56">Una palabra está escondida en el grafo semántico. Cada intento se resuelve en el servidor contra el dataset LXDB.</p></div>
          <div class="flex gap-5 font-mono text-xs text-black/55"><span><b class="text-ink">{{ game.game.value.attempts.length }}</b> intentos</span><span><b class="text-ink">{{ game.stats.value?.current_streak ?? 0 }}</b> días</span></div>
        </div>

        <div class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_330px]">
          <section class="rounded-md border border-black/14 bg-panel p-4 shadow-[0_20px_55px_rgba(23,23,19,.08)] sm:p-6">
            <div class="semantic-field relative min-h-48 overflow-hidden rounded-sm border border-black/8 p-5">
              <div class="pulse-ring absolute left-1/2 top-1/2 h-16 w-16 -translate-x-1/2 -translate-y-1/2 rounded-full border border-coral/35" /><div class="float-node absolute left-[18%] top-[26%] rounded-full border border-black/10 bg-white/85 px-3 py-1.5 font-mono text-[10px] text-black/55">idea</div><div class="float-node-delayed absolute bottom-[21%] right-[16%] rounded-full border border-black/10 bg-white/85 px-3 py-1.5 font-mono text-[10px] text-black/55">conexión</div>
              <div class="relative z-10 grid min-h-36 place-items-center text-center"><div><p class="text-[10px] font-bold uppercase tracking-[.14em] text-black/45">Proximidad del último intento</p><strong class="mt-1 block font-mono text-5xl font-medium">{{ game.progress.value }}%</strong><p class="mt-2 text-xs text-black/52">{{ game.feedback.value }}</p></div></div>
            </div>
            <div class="mt-4 h-2 overflow-hidden rounded-sm bg-black/8"><div class="h-full bg-moss transition-[width] duration-500" :style="{ width: `${game.progress.value}%` }" /></div>

            <form class="mt-6" @submit.prevent="submit"><label for="daily-word" class="text-[10px] font-bold uppercase tracking-[.12em] text-black/46">Tu siguiente palabra</label><div class="mt-2 flex gap-2"><input id="daily-word" v-model="word" autocomplete="off" maxlength="64" :disabled="game.game.value.status !== 'in_progress'" class="h-12 min-w-0 flex-1 rounded-sm border border-black/18 bg-white px-3 text-sm outline-none transition placeholder:text-black/32 focus:border-black/55 disabled:bg-black/4" placeholder="Escribe una idea…" /><button class="grid h-12 w-12 shrink-0 place-items-center rounded-sm bg-graphite text-white transition hover:bg-black disabled:cursor-not-allowed disabled:opacity-40" :disabled="!word.trim() || game.game.value.status !== 'in_progress'" type="submit" aria-label="Probar palabra"><Send :size="17" /></button></div></form>
            <div class="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-black/10 pt-4"><button class="flex cursor-pointer items-center gap-1.5 text-xs text-black/55 transition hover:text-ink disabled:opacity-35" :disabled="game.game.value.status !== 'in_progress'" type="button" @click="game.requestHint"><Lightbulb :size="15" class="text-coral" />Pedir pista <span class="font-mono text-[10px]">{{ game.game.value.hints_used }}/3</span></button><button class="cursor-pointer text-xs text-black/42 transition hover:text-coral disabled:opacity-35" :disabled="game.game.value.status !== 'in_progress'" type="button" @click="game.abandon">Abandonar reto</button></div>
            <div v-if="game.hint.value" class="mt-3 rounded-sm border border-[#bfcda5] bg-[#eef4e4] px-3 py-2 text-xs text-black/65"><b class="mr-2 font-semibold">Pista:</b>{{ game.hint.value }}</div>
          </section>

          <aside class="space-y-4"><section class="rounded-md border border-black/12 bg-white/70 p-4"><p class="text-[10px] font-bold uppercase tracking-[.12em] text-black/45">Intentos</p><ol v-if="game.game.value.attempts.length" class="mt-3 space-y-2"><li v-for="attempt in [...game.game.value.attempts].reverse()" :key="`${attempt.word}-${attempt.created_at}`" class="flex items-center gap-2 rounded-sm border border-black/8 bg-panel px-3 py-2"><span class="min-w-0 flex-1 truncate text-sm font-medium">{{ attempt.word }}</span><span class="font-mono text-xs" :class="attempt.is_exact ? 'text-moss' : 'text-black/58'">{{ percent(attempt.score) }}</span><span v-if="attempt.distance !== null" class="font-mono text-[10px] text-black/40">{{ attempt.distance }} salto{{ attempt.distance === 1 ? '' : 's' }}</span></li></ol><p v-else class="mt-3 text-sm leading-6 text-black/48">Tu ruta aparecerá aquí.</p></section>
          <section class="rounded-md border border-black/12 bg-graphite p-4 text-white"><p class="text-[10px] font-bold uppercase tracking-[.12em] text-white/45">Tu racha</p><div class="mt-2 flex items-end justify-between"><strong class="font-mono text-4xl font-medium">{{ game.stats.value?.current_streak ?? 0 }}</strong><span class="text-xs text-white/58">Mejor: {{ game.stats.value?.best_streak ?? 0 }}</span></div></section></aside>
        </div>
      </section>
    </main>

    <div v-if="game.game.value && game.game.value.status !== 'in_progress' && showResult" class="fixed inset-0 z-50 grid place-items-center bg-black/35 p-4" @click.self="showResult = false"><section class="w-full max-w-md rounded-md border border-black/15 bg-panel p-6 shadow-xl"><button class="float-right grid h-7 w-7 cursor-pointer place-items-center rounded-sm text-black/45 hover:bg-black/5" type="button" aria-label="Cerrar resultado" @click="showResult = false"><X :size="17" /></button><div class="grid h-11 w-11 place-items-center rounded-full" :class="game.game.value.status === 'won' ? 'bg-[#eef4e4] text-moss' : 'bg-[#f8e5df] text-coral'"><Trophy :size="21" /></div><p class="mt-5 text-[10px] font-bold uppercase tracking-[.12em] text-black/45">{{ game.game.value.status === 'won' ? 'Reto resuelto' : 'Reto cerrado' }}</p><h2 class="mt-2 text-2xl font-semibold tracking-tight">{{ game.game.value.status === 'won' ? 'Encontraste el nexo.' : `El nexo era ${game.game.value.target_word}.` }}</h2><p class="mt-3 text-sm leading-6 text-black/58">{{ game.game.value.attempts.length }} intentos · {{ game.game.value.score ?? 0 }} puntos · racha {{ game.stats.value?.current_streak ?? 0 }}</p><button class="mt-5 flex h-10 w-full cursor-pointer items-center justify-center gap-2 rounded-sm bg-graphite text-sm font-semibold text-white transition hover:bg-black" type="button" @click="share"><Share2 :size="16" />{{ copied ? 'Copiado' : 'Compartir resultado' }}</button></section></div>
  </div>
</template>
