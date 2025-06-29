<script lang="ts">
    import { gameStore, gameActions } from '$lib/stores/gameStore';
    import { ChessService } from '$lib/services/chessService';
    import type { ChessSquare } from '$lib/types/chess';

    export let onMove: (from: string, to: string) => void;

    $: board = $gameStore.currentGame ? ChessService.parseFEN($gameStore.currentGame.fen) : [];
    $: selectedSquare = $gameStore.selectedSquare;
    $: possibleMoves = $gameStore.possibleMoves;

    let draggedPiece: ChessSquare | null = null;
    let draggedFrom: string | null = null;

    // ...existing drag handlers...
    function handleDragStart(event: DragEvent, square: ChessSquare) {
        if (!square.piece || square.piece.color !== 'white') {
            event.preventDefault();
            return;
        }

        draggedPiece = square;
        draggedFrom = `${square.file}${square.rank}`;
        gameActions.selectSquare(draggedFrom);
        generatePossibleMoves(square);

        if (event.dataTransfer) {
            event.dataTransfer.effectAllowed = 'move';
            event.dataTransfer.setData('text/plain', '');
        }
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        if (event.dataTransfer) {
            event.dataTransfer.dropEffect = 'move';
        }
    }

    function handleDrop(event: DragEvent, square: ChessSquare) {
        event.preventDefault();
        
        if (!draggedFrom || !draggedPiece) return;

        const targetSquare = `${square.file}${square.rank}`;
        
        if (draggedFrom === targetSquare) {
            resetDragState();
            return;
        }

        if (possibleMoves.includes(targetSquare)) {
            onMove(draggedFrom, targetSquare);
        }

        resetDragState();
    }

    function handleSquareClick(square: ChessSquare) {
        const squareNotation = `${square.file}${square.rank}`;

        if (selectedSquare === squareNotation) {
            gameActions.selectSquare(null);
            gameActions.setPossibleMoves([]);
        } else if (selectedSquare && possibleMoves.includes(squareNotation)) {
            onMove(selectedSquare, squareNotation);
            gameActions.selectSquare(null);
            gameActions.setPossibleMoves([]);
        } else if (square.piece?.color === 'white') {
            gameActions.selectSquare(squareNotation);
            generatePossibleMoves(square);
        }
    }

    function generatePossibleMoves(square: ChessSquare) {
        const moves = getRealMoves(square);
        gameActions.setPossibleMoves(moves);
    }

    // Helper function to get square at position
    function getSquareAt(file: string, rank: number): ChessSquare | null {
        if (rank < 1 || rank > 8) return null;
        const fileNum = file.charCodeAt(0) - 97;
        if (fileNum < 0 || fileNum > 7) return null;
        
        return board[8 - rank]?.[fileNum] || null;
    }

    function isSquareEmpty(file: string, rank: number): boolean {
        const square = getSquareAt(file, rank);
        return square ? !square.piece : false;
    }

    function hasEnemyPiece(file: string, rank: number): boolean {
        const square = getSquareAt(file, rank);
        return square ? (square.piece?.color === 'black') : false;
    }

    function hasOurPiece(file: string, rank: number): boolean {
        const square = getSquareAt(file, rank);
        return square ? (square.piece?.color === 'white') : false;
    }

    // Check if king or rook has moved (simplified - in real game, track this in backend)
    function hasKingMoved(): boolean {
        // Check if king is on starting square
        const king = getSquareAt('e', 1);
        return !king?.piece || king.piece.type !== 'king';
    }

    function hasRookMoved(file: string): boolean {
        // Check if rook is on starting square
        const rook = getSquareAt(file, 1);
        return !rook?.piece || rook.piece.type !== 'rook';
    }

    function canCastle(kingside: boolean): boolean {
        if (hasKingMoved()) return false;

        if (kingside) {
            // Kingside castling (O-O)
            if (hasRookMoved('h')) return false;
            
            // Check squares between king and rook are empty
            if (!isSquareEmpty('f', 1) || !isSquareEmpty('g', 1)) return false;
            
            // TODO: Check king not in check and doesn't move through check
            return true;
        } else {
            // Queenside castling (O-O-O)
            if (hasRookMoved('a')) return false;
            
            // Check squares between king and rook are empty
            if (!isSquareEmpty('b', 1) || !isSquareEmpty('c', 1) || !isSquareEmpty('d', 1)) return false;
            
            // TODO: Check king not in check and doesn't move through check
            return true;
        }
    }

    function getRealMoves(square: ChessSquare): string[] {
        const moves: string[] = [];
        const { file, rank, piece } = square;
        
        if (!piece) return moves;

        const fileNum = file.charCodeAt(0) - 97;

        switch (piece.type) {
            case 'pawn':
                // Forward move
                if (rank < 8 && isSquareEmpty(file, rank + 1)) {
                    moves.push(`${file}${rank + 1}`);
                    
                    // Double move from starting position
                    if (rank === 2 && isSquareEmpty(file, rank + 2)) {
                        moves.push(`${file}${rank + 2}`);
                    }
                }
                
                // Diagonal captures
                if (fileNum > 0 && hasEnemyPiece(String.fromCharCode(96 + fileNum), rank + 1)) {
                    moves.push(`${String.fromCharCode(96 + fileNum)}${rank + 1}`);
                }
                if (fileNum < 7 && hasEnemyPiece(String.fromCharCode(98 + fileNum), rank + 1)) {
                    moves.push(`${String.fromCharCode(98 + fileNum)}${rank + 1}`);
                }
                break;
                
            case 'rook':
                moves.push(...getRookMoves(file, rank, fileNum));
                break;
                
            case 'bishop':
                moves.push(...getBishopMoves(file, rank, fileNum));
                break;
                
            case 'queen':
                // Queen = Rook + Bishop moves
                moves.push(...getRookMoves(file, rank, fileNum));
                moves.push(...getBishopMoves(file, rank, fileNum));
                break;
                
            case 'knight':
                const knightMoves = [
                    [fileNum + 2, rank + 1], [fileNum + 2, rank - 1],
                    [fileNum - 2, rank + 1], [fileNum - 2, rank - 1],
                    [fileNum + 1, rank + 2], [fileNum + 1, rank - 2],
                    [fileNum - 1, rank + 2], [fileNum - 1, rank - 2]
                ];
                
                knightMoves.forEach(([f, r]) => {
                    if (f >= 0 && f < 8 && r >= 1 && r <= 8) {
                        const newFile = String.fromCharCode(97 + f);
                        if (isSquareEmpty(newFile, r) || hasEnemyPiece(newFile, r)) {
                            moves.push(`${newFile}${r}`);
                        }
                    }
                });
                break;
                
            case 'king':
                // King normal moves
                for (let deltaF = -1; deltaF <= 1; deltaF++) {
                    for (let deltaR = -1; deltaR <= 1; deltaR++) {
                        if (deltaF === 0 && deltaR === 0) continue;
                        
                        const newF = fileNum + deltaF;
                        const newR = rank + deltaR;
                        
                        if (newF >= 0 && newF < 8 && newR >= 1 && newR <= 8) {
                            const newFile = String.fromCharCode(97 + newF);
                            if (isSquareEmpty(newFile, newR) || hasEnemyPiece(newFile, newR)) {
                                moves.push(`${newFile}${newR}`);
                            }
                        }
                    }
                }
                
                // Castling moves
                if (file === 'e' && rank === 1) {
                    if (canCastle(true)) {
                        moves.push('g1'); // Kingside castling
                    }
                    if (canCastle(false)) {
                        moves.push('c1'); // Queenside castling
                    }
                }
                break;
        }

        return moves;
    }

    // Complete Rook moves function
    function getRookMoves(file: string, rank: number, fileNum: number): string[] {
        const moves: string[] = [];
        
        // Vertical moves (up)
        for (let r = rank + 1; r <= 8; r++) {
            if (isSquareEmpty(file, r)) {
                moves.push(`${file}${r}`);
            } else {
                if (hasEnemyPiece(file, r)) {
                    moves.push(`${file}${r}`);
                }
                break; // Stop at first piece
            }
        }
        
        // Vertical moves (down)
        for (let r = rank - 1; r >= 1; r--) {
            if (isSquareEmpty(file, r)) {
                moves.push(`${file}${r}`);
            } else {
                if (hasEnemyPiece(file, r)) {
                    moves.push(`${file}${r}`);
                }
                break; // Stop at first piece
            }
        }
        
        // Horizontal moves (right)
        for (let f = fileNum + 1; f < 8; f++) {
            const newFile = String.fromCharCode(97 + f);
            if (isSquareEmpty(newFile, rank)) {
                moves.push(`${newFile}${rank}`);
            } else {
                if (hasEnemyPiece(newFile, rank)) {
                    moves.push(`${newFile}${rank}`);
                }
                break; // Stop at first piece
            }
        }
        
        // Horizontal moves (left)
        for (let f = fileNum - 1; f >= 0; f--) {
            const newFile = String.fromCharCode(97 + f);
            if (isSquareEmpty(newFile, rank)) {
                moves.push(`${newFile}${rank}`);
            } else {
                if (hasEnemyPiece(newFile, rank)) {
                    moves.push(`${newFile}${rank}`);
                }
                break; // Stop at first piece
            }
        }
        
        return moves;
    }

    // Complete Bishop moves function
    function getBishopMoves(file: string, rank: number, fileNum: number): string[] {
        const moves: string[] = [];
        
        // Diagonal moves (up-right)
        for (let i = 1; i < 8; i++) {
            const newFile = String.fromCharCode(97 + fileNum + i);
            const newRank = rank + i;
            if (newRank > 8 || fileNum + i > 7) break;
            
            if (isSquareEmpty(newFile, newRank)) {
                moves.push(`${newFile}${newRank}`);
            } else {
                if (hasEnemyPiece(newFile, newRank)) {
                    moves.push(`${newFile}${newRank}`);
                }
                break;
            }
        }
        
        // Diagonal moves (up-left)
        for (let i = 1; i < 8; i++) {
            const newFile = String.fromCharCode(97 + fileNum - i);
            const newRank = rank + i;
            if (newRank > 8 || fileNum - i < 0) break;
            
            if (isSquareEmpty(newFile, newRank)) {
                moves.push(`${newFile}${newRank}`);
            } else {
                if (hasEnemyPiece(newFile, newRank)) {
                    moves.push(`${newFile}${newRank}`);
                }
                break;
            }
        }
        
        // Diagonal moves (down-right)
        for (let i = 1; i < 8; i++) {
            const newFile = String.fromCharCode(97 + fileNum + i);
            const newRank = rank - i;
            if (newRank < 1 || fileNum + i > 7) break;
            
            if (isSquareEmpty(newFile, newRank)) {
                moves.push(`${newFile}${newRank}`);
            } else {
                if (hasEnemyPiece(newFile, newRank)) {
                    moves.push(`${newFile}${newRank}`);
                }
                break;
            }
        }
        
        // Diagonal moves (down-left)
        for (let i = 1; i < 8; i++) {
            const newFile = String.fromCharCode(97 + fileNum - i);
            const newRank = rank - i;
            if (newRank < 1 || fileNum - i < 0) break;
            
            if (isSquareEmpty(newFile, newRank)) {
                moves.push(`${newFile}${newRank}`);
            } else {
                if (hasEnemyPiece(newFile, newRank)) {
                    moves.push(`${newFile}${newRank}`);
                }
                break;
            }
        }
        
        return moves;
    }

    function resetDragState() {
        draggedPiece = null;
        draggedFrom = null;
        gameActions.selectSquare(null);
        gameActions.setPossibleMoves([]);
    }

    function getPieceSymbol(piece: ChessSquare['piece']): string {
        if (!piece) return '';
        
        const symbols = {
            white: { king: '♔', queen: '♕', rook: '♖', bishop: '♗', knight: '♘', pawn: '♙' },
            black: { king: '♚', queen: '♛', rook: '♜', bishop: '♝', knight: '♞', pawn: '♟' }
        };
        
        return symbols[piece.color][piece.type];
    }

    function getSquareClass(square: ChessSquare): string {
        const isLight = (square.file.charCodeAt(0) - 97 + square.rank) % 2 === 1;
        const squareNotation = `${square.file}${square.rank}`;
        const isSelected = selectedSquare === squareNotation;
        const isPossibleMove = possibleMoves.includes(squareNotation);
        const isCastlingMove = (squareNotation === 'g1' || squareNotation === 'c1') && 
                               selectedSquare === 'e1' && isPossibleMove;
        
        let classes = isLight ? 'square-light' : 'square-dark';
        if (isSelected) classes += ' selected';
        if (isPossibleMove) classes += ' possible-move';
        if (isCastlingMove) classes += ' castling-move';
        
        return classes;
    }
</script>

<!-- HTML reste identique -->
<div class="chess-board-container">
    <div class="board-coordinates">
        <div class="rank-labels">
            {#each [8,7,6,5,4,3,2,1] as rank}
                <div class="rank-label">{rank}</div>
            {/each}
        </div>
        
        <div class="chess-board">
            {#each board as rank}
                {#each rank as square}
                    <div 
                        class={getSquareClass(square)}
                        on:click={() => handleSquareClick(square)}
                        on:dragover={handleDragOver}
                        on:drop={(e) => handleDrop(e, square)}
                        role="button"
                        tabindex="0"
                    >
                        {#if square.piece}
                            <div 
                                class="piece {square.piece.color === 'white' ? 'draggable' : ''}"
                                draggable={square.piece.color === 'white'}
                                on:dragstart={(e) => handleDragStart(e, square)}
                            >
                                {getPieceSymbol(square.piece)}
                            </div>
                        {/if}
                        
                        <!-- Possible move indicator -->
                        {#if possibleMoves.includes(`${square.file}${square.rank}`) && !square.piece}
                            <div class="move-dot"></div>
                        {/if}
                        
                        <!-- Capture indicator -->
                        {#if possibleMoves.includes(`${square.file}${square.rank}`) && square.piece}
                            <div class="capture-ring"></div>
                        {/if}
                        
                        <!-- Castling indicator -->
                        {#if (square.file === 'g' && square.rank === 1) || (square.file === 'c' && square.rank === 1)}
                            {#if possibleMoves.includes(`${square.file}${square.rank}`) && selectedSquare === 'e1'}
                                <div class="castling-indicator">🏰</div>
                            {/if}
                        {/if}
                    </div>
                {/each}
            {/each}
        </div>
        
        <div class="file-labels">
            {#each ['a','b','c','d','e','f','g','h'] as file}
                <div class="file-label">{file}</div>
            {/each}
        </div>
    </div>
</div>

<style>
    .chess-board-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        margin: 20px auto;
    }

    .board-coordinates {
        display: flex;
        flex-direction: column;
        background: #312e2b;
        border-radius: 8px;
        padding: 8px;
        position: relative;
    }

    .rank-labels {
        display: flex;
        flex-direction: column;
        position: absolute;
        left: -20px;
        height: 480px;
        justify-content: space-around;
        align-items: center;
    }

    .rank-label {
        color: #b9b7b4;
        font-weight: bold;
        font-size: 14px;
        height: 60px;
        display: flex;
        align-items: center;
    }

    .file-labels {
        display: flex;
        width: 480px;
        justify-content: space-around;
        margin-top: 4px;
    }

    .file-label {
        color: #b9b7b4;
        font-weight: bold;
        font-size: 14px;
        width: 60px;
        text-align: center;
    }

    .chess-board {
        display: grid;
        grid-template-columns: repeat(8, 60px);
        grid-template-rows: repeat(8, 60px);
        border-radius: 4px;
        overflow: hidden;
        position: relative;
    }

    .chess-board > div {
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .square-light {
        background-color: #f0d9b5;
    }

    .square-dark {
        background-color: #b58863;
    }

    .square-light:hover {
        background-color: #e6cc99;
    }

    .square-dark:hover {
        background-color: #a67c5a;
    }

    .selected {
        background-color: #baca2b !important;
        box-shadow: inset 0 0 0 3px #7f8b00;
    }

    .possible-move {
        position: relative;
    }

    .castling-move {
        background-color: #daa520 !important;
        box-shadow: inset 0 0 0 2px #b8860b;
    }

    .piece {
        font-size: 44px;
        line-height: 1;
        user-select: none;
        transition: transform 0.1s ease;
        z-index: 10;
    }

    .piece.draggable {
        cursor: grab;
    }

    .piece.draggable:active {
        cursor: grabbing;
        transform: scale(1.1);
    }

    .move-dot {
        position: absolute;
        width: 18px;
        height: 18px;
        background-color: rgba(20, 85, 30, 0.8);
        border-radius: 50%;
        pointer-events: none;
    }

    .capture-ring {
        position: absolute;
        width: 54px;
        height: 54px;
        border: 3px solid rgba(20, 85, 30, 0.8);
        border-radius: 50%;
        pointer-events: none;
    }

    .castling-indicator {
        position: absolute;
        font-size: 20px;
        pointer-events: none;
        animation: bounce 1s infinite;
    }

    @keyframes bounce {
        0%, 100% { transform: translateY(0); }
        50% { transform: translateY(-5px); }
    }

    .piece:hover {
        transform: scale(1.05);
    }

    .chess-board > div {
        animation: fadeIn 0.3s ease-in-out;
    }

    @keyframes fadeIn {
        from { opacity: 0.8; }
        to { opacity: 1; }
    }
</style>