<script lang="ts">
    /**
     * Board coordinates component - displays file and rank labels around the chessboard
     * @param flipped - Whether the board is flipped (black's perspective)
     * @param showCoordinates - Whether to show coordinates at all
     */
    export let flipped: boolean = false;
    export let showCoordinates: boolean = true;
    
    // Files (columns) a-h
    const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    
    // Ranks (rows) 1-8
    const ranks = [1, 2, 3, 4, 5, 6, 7, 8];
    
    // Reverse order if board is flipped
    $: displayFiles = flipped ? [...files].reverse() : files;
    $: displayRanks = flipped ? ranks : [...ranks].reverse();
</script>

{#if showCoordinates}
    <div class="board-coordinates">
        <!-- Top file labels -->
        <div class="file-labels top">
            <div class="corner"></div>
            {#each displayFiles as file}
                <div class="file-label">{file}</div>
            {/each}
            <div class="corner"></div>
        </div>
        
        <!-- Board content (slot for the actual board) -->
        <div class="board-content">
            <!-- Left rank labels -->
            <div class="rank-labels left">
                {#each displayRanks as rank}
                    <div class="rank-label">{rank}</div>
                {/each}
            </div>
            
            <!-- Board slot -->
            <div class="board-slot">
                <slot />
            </div>
            
            <!-- Right rank labels -->
            <div class="rank-labels right">
                {#each displayRanks as rank}
                    <div class="rank-label">{rank}</div>
                {/each}
            </div>
        </div>
        
        <!-- Bottom file labels -->
        <div class="file-labels bottom">
            <div class="corner"></div>
            {#each displayFiles as file}
                <div class="file-label">{file}</div>
            {/each}
            <div class="corner"></div>
        </div>
    </div>
{:else}
    <!-- No coordinates, just the board -->
    <slot />
{/if}

<style>
    .board-coordinates {
        display: flex;
        flex-direction: column;
        gap: 4px;
        user-select: none;
    }
    
    .file-labels {
        display: flex;
        gap: 4px;
        align-items: center;
        justify-content: center;
    }
    
    .board-content {
        display: flex;
        gap: 4px;
        align-items: center;
    }
    
    .rank-labels {
        display: flex;
        flex-direction: column;
        gap: 4px;
        align-items: center;
        justify-content: center;
    }
    
    .board-slot {
        flex: 1;
    }
    
    .file-label,
    .rank-label {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.9rem;
        font-weight: bold;
        color: #6c757d;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
    }
    
    .corner {
        width: 32px;
        height: 32px;
    }
    
    /* Responsive adjustments */
    @media (max-width: 768px) {
        .file-label,
        .rank-label {
            width: 24px;
            height: 24px;
            font-size: 0.8rem;
        }
        
        .corner {
            width: 24px;
            height: 24px;
        }
    }
</style>