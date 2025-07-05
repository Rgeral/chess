<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import type { ChessSquare } from '$lib/types/chess';
    import ChessPiece from './ChessPiece.svelte';
    
    /**
     * Chess square component - represents a single square on the chessboard
     * @param square - The square data (position, piece, etc.)
     * @param isSelected - Whether this square is currently selected
     * @param isHighlighted - Whether this square should be highlighted (possible move)
     * @param isLastMove - Whether this square was part of the last move
     * @param isDarkSquare - Whether this is a dark square (for alternating colors)
     * @param allowDrag - Whether pieces on this square can be dragged
     * @param allowDrop - Whether pieces can be dropped on this square
     */
    export let square: ChessSquare;
    export let isSelected: boolean = false;
    export let isHighlighted: boolean = false;
    export let isLastMove: boolean = false;
    export let isDarkSquare: boolean = false;
    export let allowDrag: boolean = true;
    export let allowDrop: boolean = true;
    export let isPossibleMove: boolean = false;
    export let isLastMoveFrom: boolean = false;
    export let isLastMoveTo: boolean = false;
    export let hidepiece: boolean = false;
    
    const dispatch = createEventDispatcher<{
        click: { square: ChessSquare };
        dragstart: { square: ChessSquare, event: DragEvent };
        dragover: { square: ChessSquare, event: DragEvent };
        drop: { square: ChessSquare, event: DragEvent };
    }>();
    
    /**
     * Handle square click events
     */
    function handleClick() {
        // Passe isPossibleMove dans l'event pour que ChessBoard puisse le traiter
        dispatch('click', { square, isPossibleMove });
    }
    
    /**
     * Handle drag start events
     */
    function handleDragStart(event: DragEvent) {
        if (!allowDrag || !square.piece) return;
        dispatch('dragstart', { square, event });
    }
    
    /**
     * Handle drag over events
     */
    function handleDragOver(event: DragEvent) {
        if (!allowDrop) return;
        event.preventDefault();
        dispatch('dragover', { square, event });
    }
    
    /**
     * Handle drop events
     */
    function handleDrop(event: DragEvent) {
        if (!allowDrop) return;
        event.preventDefault();
        dispatch('drop', { square, event });
    }
    
    $: squareId = `${square.file}${square.rank}`;
</script>

<div
    class="chess-square"
    class:dark={isDarkSquare}
    class:light={!isDarkSquare}
    class:selected={isSelected}
    class:highlighted={isHighlighted}
    class:last-move-from={isLastMoveFrom}
    class:last-move-to={isLastMoveTo}
    class:last-move={isLastMove}
    class:has-piece={!!square.piece}
    data-square={squareId}
    on:click={handleClick}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
    role="button"
    tabindex="0"
    aria-label="Square {squareId}"
>
    {#if square.piece && !hidepiece}
        <div
            class="piece-container"
            draggable={allowDrag}
            on:dragstart={handleDragStart}
        >
            <ChessPiece 
                piece={square.piece} 
                draggable={allowDrag}
            />
        </div>
    {/if}
    {#if isPossibleMove}
        <div class="move-dot"></div>
    {/if}
    <span class="square-label">{squareId}</span>
</div>

<style>
    .chess-square {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s ease;
        border: 2px solid transparent;
    }
    
    .chess-square.light {
        background-color: #f0d9b5;
    }
    
    .chess-square.dark {
        background-color: #b58863;
    }
    
    .chess-square.selected {
        border-color: #ffeb3b;
        box-shadow: inset 0 0 0 3px #ffeb3b;
    }
    
    .chess-square.highlighted {
        border-color: #4caf50;
        box-shadow: inset 0 0 0 2px #4caf50;
    }
    
    .chess-square.last-move {
        border-color: #2196f3;
        box-shadow: inset 0 0 0 2px #2196f3;
    }
    
    .chess-square.last-move-from {
        box-shadow: 0 0 0 4px #f7d26a inset !important;
        background: linear-gradient(135deg, #ffe082 60%, transparent 100%) !important;
        transition: box-shadow 0.7s cubic-bezier(.4,2,.6,1), background 0.7s cubic-bezier(.4,2,.6,1);
    }
    .chess-square.last-move-to {
        box-shadow: 0 0 0 4px #f7b26a inset !important;
        background: linear-gradient(135deg, #ffcc80 60%, transparent 100%) !important;
        transition: box-shadow 0.7s cubic-bezier(.4,2,.6,1), background 0.7s cubic-bezier(.4,2,.6,1);
    }
    
    .chess-square:hover {
        filter: brightness(1.1);
    }
    
    .piece-container {
        position: relative;
        z-index: 10;
    }
    
    .square-label {
        position: absolute;
        bottom: 2px;
        left: 2px;
        font-size: 0.7rem;
        color: rgba(0, 0, 0, 0.3);
        pointer-events: none;
        user-select: none;
    }
    
    .chess-square.dark .square-label {
        color: rgba(255, 255, 255, 0.4);
    }
    
    .move-dot {
        position: absolute;
        width: 18px;
        height: 18px;
        background-color: rgba(20, 200, 30, 0.8);
        border-radius: 50%;
        pointer-events: none;
        z-index: 5;
    }
    
    /* Accessibility */
    .chess-square:focus {
        outline: 3px solid #ff9800;
        outline-offset: -3px;
    }
</style>