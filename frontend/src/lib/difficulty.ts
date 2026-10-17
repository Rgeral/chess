/** Niveaux affichés dans l'UI (1 à 10) ↔ niveaux Stockfish du backend (1 à 20). */
export const UI_LEVELS = 10;
const BACKEND_STEP = 2;

export function toBackendLevel(uiLevel: number): number {
	return uiLevel * BACKEND_STEP;
}

/** Niveau UI le plus proche d'un niveau backend (ex. 5 → 3, 20 → 10). */
export function fromBackendLevel(backendLevel: number): number {
	return Math.min(UI_LEVELS, Math.max(1, Math.ceil(backendLevel / BACKEND_STEP)));
}

export function levelName(uiLevel: number): string {
	if (uiLevel <= 2) return 'Débutant';
	if (uiLevel <= 4) return 'Novice';
	if (uiLevel <= 6) return 'Intermédiaire';
	if (uiLevel <= 8) return 'Avancé';
	return 'Expert';
}
