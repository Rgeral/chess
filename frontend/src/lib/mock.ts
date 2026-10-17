// TODO: données factices conservées de la maquette — à remplacer par de vraies données backend
// (historique de parties, succès débloqués, contenus d'apprentissage).

export const MOCK_RECENT_GAMES = [
	{ level: 3, moves: 32, minutes: 15, result: 'win' as const },
	{ level: 2, moves: 28, minutes: 12, result: 'loss' as const },
	{ level: 4, moves: 45, minutes: 22, result: 'win' as const }
];

export const MOCK_ACHIEVEMENTS = [
	{ name: 'Première victoire', glyph: '♟', unlocked: true },
	{ name: '10 parties', glyph: '♜', unlocked: true },
	{ name: 'Maître tactique', glyph: '♝', unlocked: false },
	{ name: 'Roi des échecs', glyph: '♚', unlocked: false }
];

export const MOCK_LEARNING = [
	{ title: 'Tutoriel débutant', subtitle: 'Apprends les bases', glyph: '♙' },
	{ title: 'Tactiques avancées', subtitle: 'Perfectionne ton jeu', glyph: '♞' },
	{ title: 'Analyse de parties', subtitle: 'Étudie tes erreurs', glyph: '♛' }
];
