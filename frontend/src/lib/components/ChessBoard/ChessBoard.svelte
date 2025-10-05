<!-- Wrapper that wires Chessground to the app logic (select/move/promotion) -->
<script lang="ts">
	import ChessGround from './ChessGround.svelte';
	import PromotionDialog from './PromotionDialog.svelte';
	import { gameStore, gameActions } from '$lib/stores/gameStore';
	import { ChessService } from '$lib/services/chessService';
	import { getPossibleMoves } from '$lib/services/chessMoves';
	import type {
		ChessSquare,
		BoardRef,
		SelectEvent,
		MoveEvent,
		PromoteEvent
	} from '$lib/types/chess';

	export let lastMove: { from: string; to: string } | null = null;
	export let allowMoves: boolean = false;

	const START_FEN = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';

	let boardRef: BoardRef | null = null;
	let pendingPromotion: { from: string; to: string } | null = null;

	// Cache for last selection (used to render highlighted destinations on the board)
	let lastSelectedFrom: string | null = null;

	// Helpers
	function turnFromFen(fen: string): 'white' | 'black' {
		if (!fen) return 'white';
		const parts = fen.split(' ');
		return parts[1] === 'w' ? 'white' : 'black';
	}
	function findSquareById(board: ChessSquare[][], id: string): ChessSquare | null {
		for (const row of board) for (const sq of row) if (`${sq.file}${sq.rank}` === id) return sq;
		return null;
	}
	function isPawnPromotion(from: string, to: string): boolean {
		if (!from || !to) return false;
		const fromRank = parseInt(from[1]);
		const toRank = parseInt(to[1]);
		return (fromRank === 7 && toRank === 8) || (fromRank === 2 && toRank === 1);
	}
	async function getPossibleMovesForSquare(squareId: string): Promise<string[]> {
		try {
			const fen = $gameStore.currentGame?.fen ?? START_FEN;
			const board = ChessService.parseFEN(fen);
			const square = findSquareById(board, squareId);
			if (!square) return [];
			const toMove = turnFromFen(fen);
			if (!square.piece || square.piece.color !== toMove) return [];
			const moves = getPossibleMoves(board, square, fen);
			return moves || [];
		} catch (e) {
			console.error('[ChessBoard] getPossibleMovesForSquare failed', squareId, e);
			return [];
		}
	}

	// Selection handler (compute and show destinations)
	async function onSelectFromBoard(e: SelectEvent) {
		const from = e.detail.square;
		lastSelectedFrom = from;
		const dests = await getPossibleMovesForSquare(from);
		// Update global possible moves for other parts of UI if needed
		gameActions.setPossibleMoves(dests);
		// Show destinations on the board
		boardRef?.setDests({ [from]: dests });
	}

	// Move handler (validate turn, legality, promotion, then submit move)
	async function onMoveFromBoard(e: MoveEvent) {
		const { from, to } = e.detail;

		// Validate turn (player is white)
		const fen = $gameStore.currentGame?.fen ?? START_FEN;
		const toMove = turnFromFen(fen);
		if (toMove !== 'white') {
			console.warn('[ChessBoard] Blocked move: not player turn', toMove);
			boardRef?.resetToFen($gameStore.currentGame?.fen ?? START_FEN);
			return;
		}

		// Validate legality using generator
		const allowed = await getPossibleMovesForSquare(from);
		if (!allowed.includes(to)) {
			console.warn('[ChessBoard] Blocked move (not allowed)', from, '→', to, 'allowed:', allowed);
			boardRef?.resetToFen($gameStore.currentGame?.fen ?? START_FEN);
			return;
		}

		// Promotion case
		const boardForPromotion = ChessService.parseFEN(fen);
		const fromSq = findSquareById(boardForPromotion, from);
		if (fromSq?.piece?.type === 'pawn' && isPawnPromotion(from, to)) {
			pendingPromotion = { from, to };
			boardRef?.clearDests();
			return;
		}

		await submitMove(from, to);
	}

	async function submitMove(
		from: string,
		to: string,
		promotion: 'queen' | 'rook' | 'bishop' | 'knight' | null = null
	) {
		if (!$gameStore.currentGame) return;

		let playerMove = `${from}${to}`;
		if (promotion) {
			const promotionLetters = { queen: 'q', rook: 'r', bishop: 'b', knight: 'n' } as const;
			playerMove += promotionLetters[promotion];
		}

		gameActions.setLoading(true);
		gameActions.setError(null);
		try {
			const result = await ChessService.makeMove($gameStore.currentGame.id, playerMove);
			gameActions.updateGameAfterMove(result);

			if (result.gameOver) {
				const outcome =
					result.winner === 'white'
						? 'You won! 🏆'
						: result.winner === 'black'
							? 'You lost! 😔'
							: 'Draw! 🤝';
				const time = gameActions.formatTime(result.totalTimeSeconds || $gameStore.elapsedTime);
				alert(`${outcome}\n⏱️ Time: ${time}\n♟️ Moves: ${result.game.movesCount}`);
			}
		} catch (error: unknown) {
			const msg = error instanceof Error ? error.message : String(error);
			gameActions.setError(`Invalid move: ${msg}`);
			console.error('[ChessBoard] Move error:', msg);
		} finally {
			gameActions.setLoading(false);
		}
	}

	function onPromote(e: PromoteEvent) {
		// support both { detail: { piece } } and simple payloads
		const detail =
			e.detail && typeof e.detail === 'object' ? (e.detail as any) : { piece: e.detail };
		const piece = detail.piece ?? detail;
		if (!pendingPromotion) return;
		submitMove(pendingPromotion.from, pendingPromotion.to, piece);
		pendingPromotion = null;
	}

	// Keep last move highlight in sync
	$: if (boardRef && lastMove?.from && lastMove?.to) {
		try {
			boardRef.setLastMove(lastMove.from, lastMove.to);
		} catch {}
	}
</script>

<ChessGround
	bind:this={boardRef}
	fen={$gameStore.currentGame?.fen ?? START_FEN}
	orientation="white"
	viewOnly={!allowMoves || $gameStore.currentGame?.status !== 'active'}
	on:select={onSelectFromBoard}
	on:move={onMoveFromBoard}
/>

{#if !!pendingPromotion}
	<PromotionDialog
		visible={!!pendingPromotion}
		from={pendingPromotion.from}
		to={pendingPromotion.to}
		on:promote={onPromote}
		on:cancel={() => {
			pendingPromotion = null;
			boardRef?.clearDests();
		}}
	/>
{/if}

<style>
	/* No extra styles here; container sizing handled by ChessGround */
</style>
