import { GameApi, GameApiError } from "~/services/gameService";
import type { DailyChallenge, GameSession, Stats } from "~/types/game";

export function useGame() {
	const config = useRuntimeConfig();
	const api = new GameApi(config.public.apiBase as string);
	const challenge = useState<DailyChallenge | null>(
		"daily-challenge",
		() => null,
	);
	const game = useState<GameSession | null>("daily-game", () => null);
	const stats = useState<Stats | null>("player-stats", () => null);
	const feedback = useState(
		"daily-feedback",
		() => "Conecta ideas para acercarte al nexo oculto.",
	);
	const hint = useState<string | null>("daily-hint", () => null);
	const loading = useState("daily-loading", () => false);
	const error = useState<string | null>("daily-error", () => null);

	const progress = computed(() =>
		Math.round((game.value?.attempts.at(-1)?.score ?? 0) * 100),
	);

	async function refreshGame(): Promise<void> {
		if (!game.value) return;
		game.value = await api.game(game.value.game_id);
	}

	async function initialize(): Promise<void> {
		loading.value = true;
		error.value = null;
		try {
			await api.health();
			challenge.value = await api.daily();
			game.value = await api.startDaily();
			stats.value = await api.stats();
		} catch (cause) {
			error.value =
				cause instanceof Error ? cause.message : "La API no está disponible.";
		} finally {
			loading.value = false;
		}
	}

	async function submit(word: string): Promise<void> {
		if (game.value?.status !== "in_progress") return;
		feedback.value = "";
		try {
			const updated = await api.attempt(game.value.game_id, word);
			game.value = updated;
			feedback.value =
				updated.status === "won"
					? "Has encontrado el nexo."
					: "Intento registrado con una proximidad calculada por LXDB.";
			if (updated.status === "won") stats.value = await api.stats();
		} catch (cause) {
			feedback.value =
				cause instanceof GameApiError
					? cause.message
					: "No se pudo registrar el intento.";
		}
	}

	async function requestHint(): Promise<void> {
		if (!game.value) return;
		try {
			const result = await api.hint(game.value.game_id);
			hint.value = result.hint.text;
			await refreshGame();
		} catch (cause) {
			feedback.value =
				cause instanceof Error ? cause.message : "No se pudo pedir una pista.";
		}
	}

	async function abandon(): Promise<void> {
		if (!game.value) return;
		try {
			game.value = await api.abandon(game.value.game_id);
			stats.value = await api.stats();
		} catch (cause) {
			feedback.value =
				cause instanceof Error
					? cause.message
					: "No se pudo abandonar el reto.";
		}
	}

	return {
		challenge,
		game,
		stats,
		feedback,
		hint,
		loading,
		error,
		progress,
		initialize,
		submit,
		requestHint,
		abandon,
	};
}
