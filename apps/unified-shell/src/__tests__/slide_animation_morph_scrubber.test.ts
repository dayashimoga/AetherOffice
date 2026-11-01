import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  SlideAnimationCoordinator,
  type SlideAnimationState,
} from '../state/SlideAnimationCoordinator';

describe('SlideAnimationCoordinator & Morph / SMIL Scrubber (Sub-Phase 48.2.1)', () => {
  let coordinator: SlideAnimationCoordinator;

  beforeEach(() => {
    coordinator = new SlideAnimationCoordinator();
  });

  it('1. Initial state has null tracks, zero time, zero clicks, and stopped playback', () => {
    const state = coordinator.getState();
    expect(state.animationTracks).toBeNull();
    expect(state.currentScene).toBeNull();
    expect(state.currentTimeMs).toBe(0.0);
    expect(state.currentClickIndex).toBe(0);
    expect(state.totalClicks).toBe(0);
    expect(state.totalDurationMs).toBe(0.0);
    expect(state.isPlaying).toBe(false);
    expect(state.playbackSpeed).toBe(1.0);
    expect(state.loopPlayback).toBe(false);
    expect(state.isMorphPreviewActive).toBe(false);
    expect(state.morphProgress).toBe(0.0);
  });

  it('2. Loads slide animation tracks and samples initial scene at 0ms', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    const tracks = await coordinator.loadSlideAnimations(dummyBytes, 0);

    expect(tracks).toBeDefined();
    expect(tracks.slide_index).toBe(0);
    expect(tracks.tracks.length).toBeGreaterThan(0);
    expect(coordinator.getState().totalDurationMs).toBeGreaterThan(0);
    expect(coordinator.getState().currentScene).not.toBeNull();
  });

  it('3. Scrubs animation timeline directly to an absolute timestamp in milliseconds', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideAnimations(dummyBytes, 0);

    const scene = await coordinator.scrubToTime(750);
    expect(scene).toBeDefined();
    expect(scene.time_ms).toBe(750);
    expect(coordinator.getState().currentTimeMs).toBe(750);
  });

  it('4. Scrubs animation timeline to a normalized progress ratio', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideAnimations(dummyBytes, 0);

    // 50% of 2500ms = 1250ms
    const scene = await coordinator.scrubToProgress(0.5);
    expect(scene).toBeDefined();
    expect(coordinator.getState().currentTimeMs).toBe(1250);
  });

  it('5. Advances and steps back through animation click steps', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideAnimations(dummyBytes, 0);

    const nextScene = await coordinator.advanceClick(true);
    expect(nextScene).toBeDefined();
    expect(coordinator.getState().currentClickIndex).toBe(1);

    const prevScene = await coordinator.advanceClick(false);
    expect(prevScene).toBeDefined();
    expect(coordinator.getState().currentClickIndex).toBe(0);
  });

  it('6. Starts, pauses, and toggles playback simulation', () => {
    vi.useFakeTimers();
    expect(coordinator.getState().isPlaying).toBe(false);

    coordinator.play();
    expect(coordinator.getState().isPlaying).toBe(true);

    coordinator.pause();
    expect(coordinator.getState().isPlaying).toBe(false);

    coordinator.togglePlay();
    expect(coordinator.getState().isPlaying).toBe(true);

    coordinator.togglePlay();
    expect(coordinator.getState().isPlaying).toBe(false);
    vi.useRealTimers();
  });

  it('7. Configures loop playback and playback speed multipliers', () => {
    coordinator.setLoop(true);
    expect(coordinator.getState().loopPlayback).toBe(true);

    coordinator.setPlaybackSpeed(2.0);
    expect(coordinator.getState().playbackSpeed).toBe(2.0);

    // Clamp speed limits
    coordinator.setPlaybackSpeed(10.0);
    expect(coordinator.getState().playbackSpeed).toBe(4.0);

    coordinator.setPlaybackSpeed(0.01);
    expect(coordinator.getState().playbackSpeed).toBe(0.25);
  });

  it('8. Activates interactive Morph transition preview mode', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideAnimations(dummyBytes, 0);

    const morphScene = await coordinator.startMorphPreview(0, 1, 'byObject');
    expect(morphScene).toBeDefined();
    expect(morphScene.t).toBe(0.0);
    expect(coordinator.getState().isMorphPreviewActive).toBe(true);
    expect(coordinator.getState().morphSourceSlideIdx).toBe(0);
    expect(coordinator.getState().morphTargetSlideIdx).toBe(1);
  });

  it('9. Evaluates continuous Morph transition progress interpolation', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideAnimations(dummyBytes, 0);
    await coordinator.startMorphPreview(0, 1, 'byObject');

    const morphMid = await coordinator.setMorphProgress(0.5);
    expect(morphMid).toBeDefined();
    expect(morphMid.t).toBe(0.5);
    expect(coordinator.getState().morphProgress).toBe(0.5);
    expect(morphMid.shapes.length).toBeGreaterThan(0);
  });

  it('10. Stops Morph preview and manages listener subscription lifecycle', () => {
    let callCount = 0;
    let observedState: SlideAnimationState | null = null;

    const unsubscribe = coordinator.subscribe((state) => {
      callCount++;
      observedState = state;
    });

    // 1. Initial subscription notification
    expect(callCount).toBe(1);

    coordinator.setLoop(true);
    expect(callCount).toBe(2);
    expect((observedState as SlideAnimationState | null)?.loopPlayback).toBe(true);

    coordinator.stopMorphPreview();
    expect(callCount).toBe(3);
    expect((observedState as SlideAnimationState | null)?.isMorphPreviewActive).toBe(false);

    unsubscribe();
    coordinator.setLoop(false);
    expect(callCount).toBe(3); // No updates after unsubscribe
  });
});

