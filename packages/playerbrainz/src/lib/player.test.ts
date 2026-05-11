import { describe, it, expect, beforeEach } from "vitest";
import { Player } from "./player.svelte";

describe("Player", () => {
    let player: Player;

    beforeEach(() => {
        player = new Player();
    });

    it("should initialize with default state", () => {
        expect(player.queue).toEqual([]);
        expect(player.queueIndex).toBeUndefined();
        expect(player.currentTrackId).toBeUndefined();
        expect(player.currentTime).toBe(0);
        expect(player.paused).toBe(true);
    });

    it("should play a track immediately", () => {
        player.playTrack(123);
        expect(player.queue).toEqual([123]);
        expect(player.queueIndex).toBe(0);
        expect(player.currentTrackId).toBe(123);
        expect(player.currentTime).toBe(0);
    });

    it("should add track to queue", () => {
        player.addTrackToQueue(1);
        expect(player.queue).toEqual([1]);
        expect(player.queueIndex).toBe(0);
        expect(player.currentTrackId).toBe(1);

        player.addTrackToQueue(2);
        expect(player.queue).toEqual([1, 2]);
        expect(player.queueIndex).toBe(0);
        expect(player.currentTrackId).toBe(1);
    });

    it("should go to next track", () => {
        player.playTrack(1);
        player.addTrackToQueue(2);

        player.nextTrack();
        expect(player.queueIndex).toBe(1);
        expect(player.currentTrackId).toBe(2);
        expect(player.currentTime).toBe(0);

        player.nextTrack(); // At end of queue
        expect(player.queueIndex).toBe(1);
        expect(player.currentTrackId).toBe(2);
    });

    it("should load state from an object", () => {
        player.load({
            queue: [10, 20],
            queueIndex: 1,
            currentTrackId: 20,
            currentTime: 45.5,
            paused: false,
        });

        expect(player.queue).toEqual([10, 20]);
        expect(player.queueIndex).toBe(1);
        expect(player.currentTrackId).toBe(20);
        expect(player.currentTime).toBe(45.5);
        expect(player.paused).toBe(false);
    });

    it("should serialize to JSON correctly", () => {
        player.playTrack(5);
        player.addTrackToQueue(10);
        player.currentTime = 30;
        player.paused = false;

        const serialized = JSON.stringify(player);
        const parsed = JSON.parse(serialized);

        expect(parsed.queue).toEqual([5, 10]);
        expect(parsed.queueIndex).toBe(0);
        expect(parsed.currentTrackId).toBe(5);
        expect(parsed.currentTime).toBe(30);
        expect(parsed.paused).toBe(false);
    });
});
