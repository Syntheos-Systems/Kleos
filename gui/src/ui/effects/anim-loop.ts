// Adapted from an internal effects library.
// Shared requestAnimationFrame loop -- multiple renderers share one rAF tick

type FrameCallback = () => void;

export class AnimLoop {
  private callbacks: FrameCallback[] = [];
  private running = false;
  private rafId = 0;

  private tick = (): void => {
    for (let i = 0; i < this.callbacks.length; i++) {
      this.callbacks[i]?.();
    }
    this.running = this.callbacks.length > 0;
    if (this.running) {
      this.rafId = requestAnimationFrame(this.tick);
    }
  };

  add(fn: FrameCallback): void {
    if (this.callbacks.indexOf(fn) === -1) {
      this.callbacks.push(fn);
    }
    if (!this.running) {
      this.running = true;
      this.rafId = requestAnimationFrame(this.tick);
    }
  }

  remove(fn: FrameCallback): void {
    const idx = this.callbacks.indexOf(fn);
    if (idx !== -1) {
      this.callbacks.splice(idx, 1);
    }
    if (this.callbacks.length === 0) {
      this.running = false;
      cancelAnimationFrame(this.rafId);
    }
  }

  destroy(): void {
    this.callbacks = [];
    this.running = false;
    cancelAnimationFrame(this.rafId);
  }
}

// Shared singleton for convenience
export const sharedAnimLoop = new AnimLoop();
