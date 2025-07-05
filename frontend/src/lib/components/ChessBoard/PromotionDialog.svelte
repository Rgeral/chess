<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import type { PromotionChoice } from '$lib/types/chess';
    
    export let from: string;
    export let to: string;
    export let visible: boolean = false;
    
    const dispatch = createEventDispatcher<{
        promote: PromotionChoice;
        cancel: void;
    }>();
    
    const promotionPieces = [
        { type: 'queen', symbol: '♕', name: 'Queen' },
        { type: 'rook', symbol: '♖', name: 'Rook' },
        { type: 'bishop', symbol: '♗', name: 'Bishop' },
        { type: 'knight', symbol: '♘', name: 'Knight' }
    ] as const;
    
    function handlePromotion(piece: 'queen' | 'rook' | 'bishop' | 'knight') {
        dispatch('promote', { from, to, piece });
    }
    
    function handleCancel() {
        dispatch('cancel');
    }
</script>

{#if visible}
    <div
        class="promotion-overlay"
        on:click={handleCancel}
        on:keydown={(e) => {
            if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') handleCancel();
        }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="promotion-title"
        tabindex="0"
    >
        <div class="promotion-dialog" on:click|stopPropagation tabindex="-1">
            <h3 id="promotion-title">🎯 Promote your pawn!</h3>
            <p>Choose which piece to promote to:</p>
            
            <div class="promotion-choices">
                {#each promotionPieces as piece}
                    <button 
                        class="promotion-button"
                        on:click={() => handlePromotion(piece.type)}
                        aria-label="Promote to {piece.name}"
                    >
                        <span class="piece-symbol">{piece.symbol}</span>
                        <span class="piece-name">{piece.name}</span>
                    </button>
                {/each}
            </div>
            
            <button class="cancel-button" on:click={handleCancel} aria-label="Cancel promotion">
                Cancel
            </button>
        </div>
    </div>
{/if}
<style>
    .promotion-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        backdrop-filter: blur(2px);
    }
    
    .promotion-dialog {
        background: white;
        padding: 30px;
        border-radius: 20px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        text-align: center;
        max-width: 400px;
        width: 90%;
        animation: slideIn 0.3s ease-out;
    }
    
    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateY(-20px) scale(0.95);
        }
        to {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }
    
    .promotion-choices {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 15px;
        margin: 20px 0;
    }
    
    .promotion-button {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 20px;
        background: linear-gradient(135deg, #667eea, #764ba2);
        color: white;
        border: none;
        border-radius: 15px;
        cursor: pointer;
        transition: all 0.3s;
        font-size: 16px;
    }
    
    .promotion-button:hover {
        transform: translateY(-3px);
        box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
    }
    
    .promotion-button:focus {
        outline: 3px solid #667eea40;
    }
    
    .piece-symbol {
        font-size: 36px;
        margin-bottom: 8px;
    }
    
    .piece-name {
        font-weight: bold;
        font-size: 14px;
    }
    
    .cancel-button {
        background: #6c757d;
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 10px;
        cursor: pointer;
        margin-top: 15px;
        transition: background 0.2s;
    }
    
    .cancel-button:hover {
        background: #5a6268;
    }
    
    .cancel-button:focus {
        outline: 3px solid #6c757d40;
    }
</style>