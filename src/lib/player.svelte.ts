export class Player {
    queue: number[] = $state([]);
    queueIndex: number | undefined = $state(undefined);
    currentTrackId: number | undefined = $state(undefined);
    currentTime: number = $state(0);
    paused: boolean = $state(true);

    playTrack(id: number) {
        this.queue = [id];
        this.queueIndex = 0;
        this.currentTrackId = id;
        this.currentTime = 0;
    }

    addTrackToQueue(id: number) {
        this.queue.push(id);
        if (this.queueIndex === undefined || this.queue.length === 1) {
            this.queueIndex = 0;
            this.currentTrackId = id;
            this.currentTime = 0;
        }
    }

    nextTrack() {
        if (this.queueIndex !== undefined && this.queueIndex < this.queue.length - 1) {
            this.queueIndex++;
            this.currentTrackId = this.queue[this.queueIndex];
            this.currentTime = 0;
        }
    }

    load(state: Partial<Player>) {
        if (!state) return;
        if (state.queue) this.queue = state.queue;
        if (state.queueIndex !== undefined) this.queueIndex = state.queueIndex;
        if (state.currentTrackId !== undefined) this.currentTrackId = state.currentTrackId;
        if (state.currentTime !== undefined) this.currentTime = state.currentTime;
        if (state.paused !== undefined) this.paused = state.paused;
    }

    toJSON() {
        return {
            queue: this.queue,
            queueIndex: this.queueIndex,
            currentTrackId: this.currentTrackId,
            currentTime: this.currentTime,
            paused: this.paused
        };
    }
}
