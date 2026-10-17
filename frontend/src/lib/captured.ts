const INITIAL: Record<string, number> = { p: 8, n: 2, b: 2, r: 2, q: 1 };
const GLYPHS: Record<string, string> = { p: '♟', n: '♞', b: '♝', r: '♜', q: '♛' };
const ORDER = ['q', 'r', 'b', 'n', 'p'];

/**
 * Pièces capturées déduites du FEN (matériel manquant par rapport au départ).
 * `byPlayer` = pièces noires prises par les blancs (le joueur), `byAi` = pièces blanches prises par l'IA.
 * Les promotions peuvent fausser le décompte (une dame promue masque une prise de pion).
 */
export function capturedFromFen(fen: string): { byPlayer: string[]; byAi: string[] } {
	const counts: Record<string, number> = {};
	for (const ch of fen.split(' ')[0]) counts[ch] = (counts[ch] ?? 0) + 1;

	const missing = (color: 'white' | 'black') => {
		const out: string[] = [];
		for (const type of ORDER) {
			const key = color === 'white' ? type.toUpperCase() : type;
			const n = Math.max(0, INITIAL[type] - (counts[key] ?? 0));
			for (let i = 0; i < n; i++) out.push(GLYPHS[type]);
		}
		return out;
	};

	return { byPlayer: missing('black'), byAi: missing('white') };
}
