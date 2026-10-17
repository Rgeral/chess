const KEY = 'chessclub:user';

export interface SavedSession {
	id: string;
	username: string;
}

export function loadSession(): SavedSession | null {
	try {
		const raw = localStorage.getItem(KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw);
		return parsed?.id && parsed?.username ? parsed : null;
	} catch {
		return null;
	}
}

export function saveSession(session: SavedSession): void {
	try {
		localStorage.setItem(KEY, JSON.stringify(session));
	} catch {
		// stockage indisponible (navigation privée…) : le profil reste en mémoire
	}
}

export function clearSession(): void {
	try {
		localStorage.removeItem(KEY);
	} catch {
		// rien à faire
	}
}
