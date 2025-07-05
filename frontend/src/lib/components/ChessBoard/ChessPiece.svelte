<script lang="ts">
    import type { ChessPiece } from '$lib/types/chess';
    
    /**
     * Chess piece component - renders a single chess piece
     * @param piece - The chess piece to display (type and color)
     * @param size - Size of the piece (default: 'normal')
     * @param draggable - Whether the piece can be dragged
     */
    export let piece: ChessPiece;
    export let size: 'small' | 'normal' | 'large' = 'normal';
    export let draggable: boolean = false;
    
    // Unicode symbols for chess pieces
    const pieceSymbols = {
        white: {
            king: '♔',
            queen: '♕',
            rook: '♖',
            bishop: '♗',
            knight: '♘',
            pawn: '♙'
        },
        black: {
            king: '♚',
            queen: '♛',
            rook: '♜',
            bishop: '♝',
            knight: '♞',
            pawn: '♟'
        }
    };
    
    $: symbol = pieceSymbols[piece.color][piece.type];
</script>

<span 
    class="chess-piece {size} {piece.color}"
    class:draggable
    {draggable}
    aria-label="{piece.color} {piece.type}"
    role="img"
>
    {symbol}
</span>

<style>
    .chess-piece {
        display: inline-block;
        font-size: 2rem;
        line-height: 1;
        user-select: none;
        transition: transform 0.2s ease;
    }
    
    .chess-piece.small {
        font-size: 1.5rem;
    }
    
    .chess-piece.large {
        font-size: 2.5rem;
    }
    
    .chess-piece.draggable {
        cursor: grab;
    }
    
    .chess-piece.draggable:active {
        cursor: grabbing;
    }
    
    .chess-piece:hover {
        transform: scale(1.1);
    }
    
    .chess-piece.white {
        color: #ffffff;
        text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8);
    }
    
    .chess-piece.black {
        color: #2c3e50;
        text-shadow: 1px 1px 2px rgba(255, 255, 255, 0.3);
    }
</style>