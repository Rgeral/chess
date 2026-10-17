import { get } from 'svelte/store';
import { gameStore, gameActions } from '$lib/stores/gameStore';
import { ChessService } from '$lib/services/chessService';
import { loadSession, clearSession } from '$lib/session';

/**
 * Recharge l'utilisateur depuis le stockage local si le store est vide.
 * Retourne true si un profil est disponible.
 */
export async function restoreSession(): Promise<boolean> {
	if (get(gameStore).user) return true;
	const saved = loadSession();
	if (!saved) return false;
	try {
		const profile = await ChessService.getUserProfile(saved.id);
		gameActions.setUser(profile.user);
		gameActions.setUserProfile(profile);
		return true;
	} catch {
		// profil introuvable côté serveur : on repart de zéro
		clearSession();
		return false;
	}
}
