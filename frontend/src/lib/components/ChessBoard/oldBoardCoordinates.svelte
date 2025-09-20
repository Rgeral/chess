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
        <!-- Top file labels (optionnel, peut être masqué si non voulu) -->
        <div class="file-labels top">
            <!-- <div class="corner"></div> -->
            {#each displayFiles as file}
                <div class="file-label">{file}</div>
            {/each}
            <!-- <div class="corner"></div> -->
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
        <!-- Bottom file labels (sous l'échiquier, alignement parfait) -->
        <div class="file-labels bottom">
            {#each displayFiles as file}
                <div class="file-label">{file}</div>
            {/each}
        </div>
    </div>
{:else}
    <!-- No coordinates, just the board -->
    <slot />
{/if}

<style>
    .board-coordinates {
        position: relative;
        width: 500px;
        margin: 0 auto;
        user-select: none;
    }
    .file-labels {
        display: grid;
        grid-template-columns: repeat(8, 1fr);
        width: 500px;
        position: absolute;
        left: 0;
        pointer-events: none;
    }
    .file-labels.top {
        top: 0;
    }
    .file-labels.bottom {
        top: 500px;
    }
    .file-label {
        width: 62.5px;
        text-align: center;
        font-size: 16px;
        color: #b9b7b4;
        font-weight: bold;
        height: 24px;
        line-height: 24px;
        background: none;
        border-radius: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
    }
    .board-content {
        display: flex;
        align-items: center;
        position: relative;
        width: 500px;
        height: 500px;
    }
    .rank-labels {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        align-items: center;
        position: absolute;
        top: 0;
        height: 500px;
        pointer-events: none;
    }
    .rank-labels.left {
        left: 0;
    }
    .rank-labels.right {
        left: 500px;
    }
    .rank-label {
        width: 24px;
        height: 62.5px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 16px;
        color: #b9b7b4;
        font-weight: bold;
        background: none;
        border-radius: 0;
        padding: 0;
    }
    .board-slot {
        width: 500px;
        height: 500px;
        position: relative;
        z-index: 1;
    }
    /* Responsive design */
    @media (max-width: 768px) {
        .board-coordinates, .file-labels, .board-content, .board-slot {
            width: 350px;
            height: 350px;
        }
        .file-labels.bottom {
            top: 350px;
        }
        .file-label {
            width: 43.75px;
        }
        .rank-labels.right {
            left: 350px;
        }
        .rank-label {
            width: 18px;
            height: 43.75px;
            font-size: 13px;
        }
    }
    @media (max-width: 480px) {
        .board-coordinates, .file-labels, .board-content, .board-slot {
            width: 280px;
            height: 280px;
        }
        .file-labels.bottom {
            top: 280px;
        }
        .file-label {
            width: 35px;
        }
        .rank-labels.right {
            left: 280px;
        }
        .rank-label {
            width: 14px;
            height: 35px;
            font-size: 11px;
        }
    }
</style>