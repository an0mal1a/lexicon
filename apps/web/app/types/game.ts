export type GameStatus = "in_progress" | "won" | "abandoned";

export interface Attempt {
	word: string;
	score: number;
	distance: number | null;
	rank: number | null;
	is_exact: boolean;
	created_at: string;
}

export interface DailyChallenge {
	challenge_id: string;
	date: string;
	language: string;
	mode: "daily";
	status: "available";
	completed: boolean;
}

export interface GameSession {
	game_id: string;
	challenge_id: string;
	date: string;
	language: string;
	mode: "daily";
	status: GameStatus;
	attempts: Attempt[];
	hints_used: number;
	score: number | null;
	target_word?: string;
}

export interface HintResponse {
	hint: { number: number; text: string };
	hints_used: number;
}

export interface Stats {
	games_played: number;
	games_won: number;
	current_streak: number;
	best_streak: number;
	total_attempts: number;
	average_attempts: number;
}

// Retained only for the currently unmounted exploratory components. The daily
// screen consumes the API types above and never creates these mock shapes.
export interface LegacyAttempt {
	word: string;
	similarity: number;
	rank?: number;
	createdAt: string;
}

export interface SemanticConnection {
	from: string;
	to: string;
	strength: number;
}

export interface PlayerStats {
	currentStreak: number;
	bestStreak: number;
	gamesPlayed: number;
	gamesWon: number;
	averageAttempts: number;
	recentDays: boolean[];
}

export interface GameModeCard {
	id: string;
	title: string;
	description: string;
	accent: "ink" | "coral" | "sage" | "moss";
	available: boolean;
}
