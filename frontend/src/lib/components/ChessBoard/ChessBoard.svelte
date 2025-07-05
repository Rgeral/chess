<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { get } from 'svelte/store';
    import type { ChessSquare as ChessSquareType, ChessPiece, PromotionChoice } from '$lib/types/chess';
    import ChessSquareComponent from './ChessSquare.svelte';
    import BoardCoordinates from './BoardCoordinates.svelte';
    import PromotionDialog from './PromotionDialog.svelte';
    import { ChessService } from '$lib/services/chessService';
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import { getPossibleMoves as getMovesFromService } from '$lib/services/chessMoves';
    
    /**
     * Main ChessBoard component - manages the entire chess board display and interaction
     * @param fen - Current board position in FEN notation
     * @param flipped - Whether to show board from black's perspective
     * @param showCoordinates - Whether to show file/rank labels
     * @param allowMoves - Whether moves are allowed (interactive mode)
     * @param highlightedSquares - Array of squares to highlight
     * @param selectedSquare - Currently selected square
     * @param lastMove - Last move made (from/to squares)
     */
    export let fen: string = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
    export let flipped: boolean = false;
    export let showCoordinates: boolean = true;
    export let allowMoves: boolean = true;
    export let highlightedSquares: string[] = [];
    export let selectedSquare: string | null = null;
    export let lastMove: { from: string; to: string } | null = null;
    
    const dispatch = createEventDispatcher<{
        move: { from: string; to: string; promotion?: string };
        squareClick: { square: string };
        pieceSelect: { square: string; piece: ChessPiece };
    }>();
    
    // Board state
    let board: ChessSquareType[][] = [];
    let promotionDialog = {
        visible: false,
        from: '',
        to: ''
    };
    
    // Files and ranks for board generation
    const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const ranks = [8, 7, 6, 5, 4, 3, 2, 1];
    
    // Ajout de l'état pour la sélection et les coups possibles
    let selectedSquareId: string | null = null;
    let possibleMoves: string[] = [];
    
    // Animation state for adversary move
    let animating = false;
    let animationPiece: ChessPiece | null = null;
    let animationFrom: string | null = null;
    let animationTo: string | null = null;
    let animationTimeout: any = null;
    let previousLastMove: { from: string; to: string } | null = null;
    let showLastMoveHighlight: { from: string; to: string } | null = null;
    let isProcessingAnimation = false; // Flag to prevent multiple concurrent animations
    const ANIMATION_DURATION = 1200; // ms
    
    /**
     * Parse FEN string and generate board representation
     */
    function parseFEN(fen: string): ChessSquareType[][] {
        const parts = fen.split(' ');
        const position = parts[0];
        const ranks = position.split('/');
        
        const board: ChessSquareType[][] = [];
        
        for (let rankIndex = 0; rankIndex < 8; rankIndex++) {
            const rank: ChessSquareType[] = [];
            const rankString = ranks[rankIndex];
            let fileIndex = 0;
            
            for (const char of rankString) {
                if (char >= '1' && char <= '8') {
                    // Empty squares
                    const emptyCount = parseInt(char);
                    for (let i = 0; i < emptyCount; i++) {
                        rank.push({
                            file: files[fileIndex],
                            rank: 8 - rankIndex,
                            piece: null
                        });
                        fileIndex++;
                    }
                } else {
                    // Piece
                    const piece = parsePiece(char);
                    rank.push({
                        file: files[fileIndex],
                        rank: 8 - rankIndex,
                        piece
                    });
                    fileIndex++;
                }
            }
            
            board.push(rank);
        }
        
        return board;
    }
    
    /**
     * Parse a single piece character from FEN
     */
    function parsePiece(char: string): ChessPiece {
        const pieceMap: Record<string, ChessPiece['type']> = {
            'k': 'king', 'q': 'queen', 'r': 'rook',
            'b': 'bishop', 'n': 'knight', 'p': 'pawn'
        };
        
        const type = pieceMap[char.toLowerCase()];
        const color = char === char.toUpperCase() ? 'white' : 'black';
        
        return { type, color };
    }
    
    /**
     * Fonction pour calculer les coups possibles d'une pièce (exemple simple, à adapter selon ta logique)
     */
    function getPossibleMoves(square: ChessSquareType): string[] {
        // Passe le FEN au service pour le roque
        return getMovesFromService(board, square, fen);
    }
    
    /**
     * Handle square click events
     */
    function handleSquareClick(event: CustomEvent) {
        const { square, isPossibleMove } = event.detail;
        const squareId = `${square.file}${square.rank}`;
        console.log('[handleSquareClick] Clicked square:', squareId, 'piece:', square.piece, 'isPossibleMove:', isPossibleMove);

        // 1. Si une pièce est sélectionnée et on clique sur une destination valide (case verte)
        if (selectedSquareId && possibleMoves.includes(squareId)) {
            console.log('[handleSquareClick] Move from', selectedSquareId, 'to', squareId);
            handleMove({ from: selectedSquareId, to: squareId });
            selectedSquareId = null;
            possibleMoves = [];
            return;
        }
        // 2. Sinon, si la case contient une pièce, on la sélectionne
        if (square.piece) {
            selectedSquareId = squareId;
            possibleMoves = getPossibleMoves(square);
            console.log('[handleSquareClick] Piece selected:', squareId, 'Possible moves:', possibleMoves);
            return;
        }
        // 3. Sinon, on désélectionne
        console.log('[handleSquareClick] Deselect');
        selectedSquareId = null;
        possibleMoves = [];
    }
    
    /**
     * Handle drag start events
     */
    function handleDragStart(event: CustomEvent) {
        const { square } = event.detail;
        // Set drag data
        event.detail.event.dataTransfer?.setData('text/plain', `${square.file}${square.rank}`);
    }
    
    /**
     * Handle drop events
     */
    function handleDrop(event: CustomEvent) {
        const { square } = event.detail;
        const fromSquare = event.detail.event.dataTransfer?.getData('text/plain');
        const toSquare = `${square.file}${square.rank}`;

        if (fromSquare && fromSquare !== toSquare) {
            // Check if this is a pawn promotion
            if (isPawnPromotion(fromSquare, toSquare)) {
                promotionDialog = {
                    visible: true,
                    from: fromSquare,
                    to: toSquare
                };
            } else {
                handleMove({ from: fromSquare, to: toSquare });
            }
        }
    }
    
    /**
     * Check if a move is a pawn promotion
     */
    function isPawnPromotion(from: string, to: string): boolean {
        const fromRank = parseInt(from[1]);
        const toRank = parseInt(to[1]);
        const fromSquare = getSquareFromId(from);
        
        if (!fromSquare?.piece || fromSquare.piece.type !== 'pawn') {
            return false;
        }
        
        return (fromSquare.piece.color === 'white' && fromRank === 7 && toRank === 8) ||
               (fromSquare.piece.color === 'black' && fromRank === 2 && toRank === 1);
    }
    
    /**
     * Get square object from square ID
     */
    function getSquareFromId(squareId: string): ChessSquareType | null {
        for (const rank of board) {
            for (const square of rank) {
                if (`${square.file}${square.rank}` === squareId) {
                    return square;
                }
            }
        }
        return null;
    }
    
    /**
     * Handle promotion dialog events
     */
    function handlePromotion(event: CustomEvent<PromotionChoice>) {
        const { from, to, piece } = event.detail;
        promotionDialog.visible = false;
        handleMove({ from, to, promotion: piece });
    }
    
    function handlePromotionCancel() {
        promotionDialog.visible = false;
    }
    
    /**
     * Check if a square should be highlighted
     */
    function isSquareHighlighted(squareId: string): boolean {
        return highlightedSquares.includes(squareId);
    }
    
    /**
     * Check if a square is part of the last move
     */
    function isLastMoveSquare(squareId: string): boolean {
        return lastMove ? (lastMove.from === squareId || lastMove.to === squareId) : false;
    }
    
    /**
     * Check if a square is dark (for alternating colors)
     */
    function isDarkSquare(file: string, rank: number): boolean {
        const fileIndex = files.indexOf(file);
        return (fileIndex + rank) % 2 === 0;
    }
    
    // Reactive statements
    $: board = parseFEN(fen);
    $: displayBoard = flipped ? [...board].reverse().map(rank => [...rank].reverse()) : board;
    
    async function handleMove({ from, to, promotion }: { from: string, to: string, promotion?: string }) {
        // Récupère l'id de la partie et le FEN actuel (à adapter selon ton store)
        const game = get(gameStore).currentGame;
        if (!game) {
            console.error('[handleMove] No current game');
            return;
        }
        // Format du move pour le backend (ex: e2e4, g1f3, e7e8q)
        let move = `${from}${to}`;
        if (promotion) move += promotion[0]; // q, r, b, n
        console.log('[handleMove] Sending move to backend:', move);
        try {
            const result = await ChessService.makeMove(game.id, move);
            // Met à jour le store avec le résultat complet (inclut lastMove)
            gameActions.updateGameAfterMove(result);
            // Met à jour le FEN du board avec la réponse
            if (result && result.game && result.game.fen) {
                fen = result.game.fen;
                console.log('[handleMove] New FEN:', fen);
            }
        } catch (e) {
            console.error('[handleMove] Error:', e);
        }
    }
    
    // Remove or comment out debug logs for production
    // console.log('[ANIMATION-DEBUG] Component render, lastMove:', lastMove, 'board.length:', board.length);

    // Watch for new moves and trigger animation
    $: if (lastMove && board.length && !previousLastMove || 
           (lastMove && previousLastMove && 
            (lastMove.from !== previousLastMove.from || lastMove.to !== previousLastMove.to))) {
        // console.log('[ANIMATION-DEBUG] New move detected:', lastMove, 'previous:', previousLastMove);
        handleNewMove(lastMove);
        previousLastMove = lastMove ? { from: lastMove.from, to: lastMove.to } : null;
    }

    // Liste des types valides pour une pièce
    const validPieceTypes = ['king', 'queen', 'rook', 'bishop', 'knight', 'pawn'];

    function normalizePieceType(type: string): ChessPiece['type'] {
        if (validPieceTypes.includes(type)) return type as ChessPiece['type'];
        // Keep this warning for backend issues
        console.warn('[ANIMATION] Type de pièce inconnu reçu du backend:', type, '-> fallback pawn');
        return 'pawn';
    }

    function handleNewMove(move: { from: string; to: string }) {
        // Prevent multiple concurrent animation processing
        if (isProcessingAnimation) {
            // console.log('[ANIMATION] Already processing animation, skipping');
            return;
        }
        
        // console.log('[ANIMATION] Processing new move:', move);
        
        // Get the piece info from the backend's lastMove data
        const gameState = get(gameStore);
        const lastMoveData = gameState.lastMove;
        
        // console.log('[ANIMATION] Current game state lastMove:', lastMoveData);
        
        // Only animate if this is a move from the backend (opponent move)
        if (lastMoveData && lastMoveData.from === move.from && lastMoveData.to === move.to) {
            // console.log('[ANIMATION] Detected opponent move, starting animation');
            
            // Set processing flag
            isProcessingAnimation = true;
            
            // Clear any existing animation
            if (animationTimeout) {
                clearTimeout(animationTimeout);
                animating = false;
            }
            
            // Show the last move highlight
            showLastMoveHighlight = { from: move.from, to: move.to };
            
            // Utilise le type de pièce du backend, normalisé
            const pieceTypeRaw = lastMoveData.piece || 'pawn';
            const pieceType = normalizePieceType(pieceTypeRaw);
            let piece: ChessPiece = {
                type: pieceType,
                color: 'black'
            };
            
            // console.log('[ANIMATION] Animation piece:', piece);
            
            animating = true;
            animationPiece = piece;
            animationFrom = move.from;
            animationTo = move.to;
            
            animationTimeout = setTimeout(() => {
                // console.log('[ANIMATION] Animation finished for move:', move);
                animating = false;
                animationPiece = null;
                animationFrom = null;
                animationTo = null;
                animationTimeout = null;
                isProcessingAnimation = false; // Reset processing flag
                
                // Clear the lastMove highlighting after animation
                setTimeout(() => {
                    showLastMoveHighlight = null;
                }, 1000); // Give time to see the final position
            }, ANIMATION_DURATION);
        } // else {
        //     console.log('[ANIMATION] No animation needed - not an opponent move');
        // }
    }

    function getAnimationStyle(from: string | null, to: string | null) {
        if (!from || !to) {
            // console.log('[ANIMATION] getAnimationStyle called with invalid from/to:', from, to);
            return '';
        }
        
        // console.log('[ANIMATION] Creating animation style for:', from, '→', to);
        
        // Convert chess notation to board coordinates (0-7)
        const fileToIdx = (f: string) => files.indexOf(f);
        // For rank indexing: rank 8 -> index 0, rank 7 -> index 1, etc.
        const rankToIdx = (r: number) => 8 - r;
        
        const fromFile = from[0];
        const fromRank = parseInt(from[1]);
        const toFile = to[0];
        const toRank = parseInt(to[1]);
        
        const fromX = fileToIdx(fromFile);
        const fromY = rankToIdx(fromRank);
        const toX = fileToIdx(toFile);
        const toY = rankToIdx(toRank);
        
        // console.log('[ANIMATION] Coordinate mapping:', {
        //     from, to,
        //     fromFile, fromRank, toFile, toRank,
        //     fromX, fromY, toX, toY
        // });
        
        // Calculate positions in pixels (assuming 62.5px per square for a 500px board)
        const squareSize = 62.5; // 500px / 8 squares
        const fromLeft = fromX * squareSize;
        const fromTop = fromY * squareSize;
        const toLeft = toX * squareSize;
        const toTop = toY * squareSize;
        
        // console.log('[ANIMATION] Pixel positions:', {
        //     fromLeft, fromTop, toLeft, toTop,
        //     translateX: toLeft - fromLeft,
        //     translateY: toTop - fromTop
        // });
        
        const style = `
            position: absolute;
            left: ${fromLeft}px;
            top: ${fromTop}px;
            width: ${squareSize}px;
            height: ${squareSize}px;
            transform: translate(${toLeft - fromLeft}px, ${toTop - fromTop}px);
            transition: transform ${ANIMATION_DURATION}ms cubic-bezier(.4,2,.6,1);
            z-index: 20;
            pointer-events: none;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 44px;
            line-height: 1;
            box-sizing: border-box;
        `;
        
        return style;
    }

    // Call test on mount
    onMount(() => {
        console.log('[CHESS] ChessBoard component mounted');
    });
</script>

<div class="chess-board-container">
    <BoardCoordinates {flipped} {showCoordinates}>
        <div class="chess-board" class:flipped>
            {#each displayBoard as rank, rankIndex}
                {#each rank as square, fileIndex}
                    {@const squareId = `${square.file}${square.rank}`}
                    <ChessSquareComponent
                        {square}
                        isSelected={selectedSquareId === squareId}
                        isHighlighted={possibleMoves.includes(squareId)}
                        isPossibleMove={possibleMoves.includes(squareId)}
                        isLastMove={showLastMoveHighlight ? (showLastMoveHighlight.from === squareId || showLastMoveHighlight.to === squareId) : false}
                        isLastMoveFrom={!!(showLastMoveHighlight && showLastMoveHighlight.from === squareId)}
                        isLastMoveTo={!!(showLastMoveHighlight && showLastMoveHighlight.to === squareId)}
                        isDarkSquare={isDarkSquare(square.file, square.rank)}
                        allowDrag={allowMoves}
                        allowDrop={allowMoves}
                        hidepiece={animating && animationFrom === squareId}
                        on:click={handleSquareClick}
                        on:dragstart={handleDragStart}
                        on:drop={handleDrop}
                    />
                {/each}
            {/each}
            
            <!-- Floating animation piece -->
            {#if animating && animationPiece}
                <div
                    class="animating-piece"
                    style={getAnimationStyle(animationFrom, animationTo)}
                >
                    {animationPiece.type === 'king' ? (animationPiece.color === 'white' ? '♔' : '♚')
                    : animationPiece.type === 'queen' ? (animationPiece.color === 'white' ? '♕' : '♛')
                    : animationPiece.type === 'rook' ? (animationPiece.color === 'white' ? '♖' : '♜')
                    : animationPiece.type === 'bishop' ? (animationPiece.color === 'white' ? '♗' : '♝')
                    : animationPiece.type === 'knight' ? (animationPiece.color === 'white' ? '♘' : '♞')
                    : animationPiece.type === 'pawn' ? (animationPiece.color === 'white' ? '♙' : '♟')
                    : ''}
                </div>
            {/if}
        </div>
    </BoardCoordinates>
    
    <PromotionDialog
        visible={promotionDialog.visible}
        from={promotionDialog.from}
        to={promotionDialog.to}
        on:promote={handlePromotion}
        on:cancel={handlePromotionCancel}
    />
</div>

<style>
    .chess-board-container {
        display: inline-block;
        margin: 20px;
        border-radius: 8px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
        background: #f8f9fa;
        padding: 20px;
    }
    
    .chess-board {
        display: grid;
        grid-template-columns: repeat(8, 1fr);
        grid-template-rows: repeat(8, 1fr);
        gap: 1px;
        width: 500px;
        height: 500px;
        border: 2px solid #8b4513;
        border-radius: 4px;
        overflow: hidden;
        position: relative; /* Important for absolute positioning of animated piece */
    }
    
    .chess-board.flipped {
        transform: rotate(180deg);
    }
    
    .animating-piece {
        user-select: none;
        text-shadow: 1px 1px 2px rgba(0,0,0,0.5);
    }
    
    /* Responsive design */
    @media (max-width: 768px) {
        .chess-board {
            width: 350px;
            height: 350px;
        }
        
        .chess-board-container {
            margin: 10px;
            padding: 15px;
        }
    }
    
    @media (max-width: 480px) {
        .chess-board {
            width: 280px;
            height: 280px;
        }
        
        .chess-board-container {
            margin: 5px;
            padding: 10px;
        }
    }
</style>