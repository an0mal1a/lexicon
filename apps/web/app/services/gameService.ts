import type {
	DailyChallenge,
	GameSession,
	HintResponse,
	Stats,
} from "~/types/game";

export class GameApiError extends Error {
	constructor(
		message: string,
		readonly status: number,
	) {
		super(message);
	}
}

export class GameApi {
	constructor(private readonly baseUrl: string) {}

	async health(): Promise<{ dataset_loaded: boolean }> {
		return this.request("/api/health");
	}

	async daily(): Promise<DailyChallenge> {
		return this.request("/api/game/daily");
	}

	async startDaily(): Promise<GameSession> {
		return this.request("/api/game/daily/start", { method: "POST", body: {} });
	}

	async attempt(gameId: string, word: string): Promise<GameSession> {
		const result = await this.request<{
			attempt: unknown;
			status: GameSession["status"];
			attempt_count: number;
			score: number | null;
		}>(`/api/game/${encodeURIComponent(gameId)}/attempt`, {
			method: "POST",
			body: { word },
		});
		return this.game(gameId, result.status, result.score);
	}

	async hint(gameId: string): Promise<HintResponse> {
		return this.request(`/api/game/${encodeURIComponent(gameId)}/hint`, {
			method: "POST",
		});
	}

	async abandon(gameId: string): Promise<GameSession> {
		return this.request(`/api/game/${encodeURIComponent(gameId)}/abandon`, {
			method: "POST",
		});
	}

	async game(
		gameId: string,
		status?: GameSession["status"],
		score?: number | null,
	): Promise<GameSession> {
		const game = await this.request<GameSession>(
			`/api/game/${encodeURIComponent(gameId)}`,
		);
		return {
			...game,
			status: status ?? game.status,
			score: score ?? game.score,
		};
	}

	async stats(): Promise<Stats> {
		return this.request("/api/stats");
	}

	private async request<T>(
		path: string,
		init: { method?: string; body?: unknown } = {},
	): Promise<T> {
		const response = await fetch(`${this.baseUrl}${path}`, {
			method: init.method ?? "GET",
			credentials: "include",
			headers: init.body ? { "content-type": "application/json" } : undefined,
			body: init.body ? JSON.stringify(init.body) : undefined,
		});
		const payload = (await response.json().catch(() => ({}))) as {
			error?: string;
		} & T;
		if (!response.ok)
			throw new GameApiError(
				payload.error ?? "No se pudo conectar con Lexicon.",
				response.status,
			);
		return payload;
	}
}
