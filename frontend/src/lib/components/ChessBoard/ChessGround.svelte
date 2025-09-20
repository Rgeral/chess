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
    let container: HTMLDivElement; // conteneur DOM du board
    let cg: any;                   // instance chessground (on l'initialisera ensuite)

    // Petite utilitaire: déduit la couleur au trait depuis le FEN
    function turnFromFen(f: string): 'white' | 'black' {
        return f.split(' ')[1] === 'w' ? 'white' : 'black';
    }

    // Crée l'instance Chessground au montage (client-only)
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
            movable: { free: false, showDests: true },
            draggable: { enabled: !viewOnly, showGhost: true },
            events: {
                move: (orig: string, dest: string) => dispatch('move', { from: orig, to: dest }),
                select: (sq: string) => dispatch('select', { square: sq })
            }
        });
    });

    // Détruit proprement l'instance à la désinstallation
    onDestroy(() => cg?.destroy?.());

    // Réagit aux changements de props et resynchronise le plateau
    $: if (cg) {
        cg.set({
            fen,
            orientation,
            viewOnly,
            turnColor: turnFromFen(fen)
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