<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { createEventDispatcher } from 'svelte';
    import 'chessground/assets/chessground.base.css';
    import 'chessground/assets/chessground.brown.css';
    import 'chessground/assets/chessground.cburnett.css';

    export let fen: string;
    export let orientation: 'white' | 'black' = 'white';
    export let viewOnly = false;

    const dispatch = createEventDispatcher();
    let container: HTMLDivElement; // container DOM element
    let cg: any;                   // chessground instance
    // current destinations set by the parent (preserve across reactive updates)
    let currentDests: Record<string, string[]> = {};

    // chessground types expect a Map<string, string[]>, convert helper
    function destsToMap(d: Record<string, string[]>) {
        try {
            return new Map(Object.entries(d || {}));
        } catch {
            return new Map();
        }
    }

    /**
	 * Defines the turn color from a FEN string
	 * @param f FEN string
	 * @returns 'white' or 'black'
	 */
    function turnFromFen(f: string): 'white' | 'black' {
        return f.split(' ')[1] === 'w' ? 'white' : 'black';
    }

	 // Exposed API — parent peut binder le composant (bind:this) et appeler ces fonctions
    export function setDests(dests: Record<string, string[]>) {
        if (!cg) return;
        currentDests = dests || {};
        console.debug('[ChessGround] setDests', currentDests);
        // chessground accepte movable.dests pour afficher destinations
        // Set the full movable config so we don't accidentally enable free moves
        cg.set({ movable: { free: false, showDests: true, dests: destsToMap(currentDests), color: turnFromFen(fen) } });
    }

    export function clearDests() {
        if (!cg) return;
    currentDests = {};
        console.debug('[ChessGround] clearDests');
    cg.set({ movable: { free: false, showDests: true, dests: destsToMap(currentDests), color: turnFromFen(fen) } });
    }

    export function setLastMove(from?: string, to?: string) {
        if (!cg) return;
        // essaie de définir le lastMove si l'API le supporte; sinon on active le highlight
        try {
            // chessground expects an array [from, to] for lastMove
            cg.set({ lastMove: from && to ? [from, to] : null });
        } catch {
            // fallback: toggle highlight option (cela garde l'affichage par défaut)
            cg.set({ highlight: { lastMove: !!(from && to) } });
        }
    }

    // Force the board to a specific FEN (useful to cancel illegal/free moves)
    export function resetToFen(f: string) {
        if (!cg) return;
        try {
            console.debug('[ChessGround] resetToFen', f);
            cg.set({ fen: f });
        } catch (err) {
            console.warn('ChessGround.resetToFen failed', err);
        }
    }


	/**
	 * Initializes the Chessground instance when the component is mounted (client-side only)
	 * and sets up event listeners for move and select events.
	 * Cleans up the instance on component destruction.
	 * Reactively updates the Chessground instance when props change.
	*/
    onMount(async () => {
        const { Chessground } = await import('chessground');

        cg = Chessground(container, {
            fen,
            orientation,
            viewOnly,
            coordinates: true,
            turnColor: turnFromFen(fen),
            highlight: { lastMove: true, check: true },
            animation: { enabled: true, duration: 200 },
            movable: { free: false, showDests: true, dests: destsToMap(currentDests), color: turnFromFen(fen) },
            draggable: { enabled: !viewOnly, showGhost: true },
            events: {
                move: (orig: string, dest: string) => dispatch('move', { from: orig, to: dest }),
                select: (sq: string) => dispatch('select', { square: sq })
            }
        });
		// ready event — utile si le parent veut récupérer la référence par event plutôt que bind:this
        dispatch('ready', { setDests, clearDests, setLastMove });
    });

    /**
	 * Cleans up the Chessground instance when the component is destroyed
	 * to prevent memory leaks.
	*/
    onDestroy(() => cg?.destroy?.());

    // Réagit aux changements de props et resynchronise seulement les propriétés
    // essentielles (fen, orientation, viewOnly, turnColor). On évite de réécrire
    // la config `movable` ici pour ne pas écraser `dests` fournis par le parent.
    $: if (cg) {
        console.debug('[ChessGround] reactive sync fen/orientation', { fen, orientation, viewOnly });
        cg.set({
            fen,
            orientation,
            viewOnly,
            turnColor: turnFromFen(fen),
            // highlight kept as configured on init
            highlight: { lastMove: true, check: true }
        });
    }

</script>

<div class="chessground-container" bind:this={container}></div>
<style>
    .chessground-container {
        width: 100%;
        aspect-ratio: 1;
        margin : auto;
        border-radius: 8px;
        overflow: hidden;
    }
</style>