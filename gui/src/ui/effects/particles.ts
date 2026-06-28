// Adapted from an internal effects library.
// ========== PARTICLE SYSTEM ==========
// Canvas-based floating particle field with mouse repulsion

export interface ThemeColors {
  accentR: number;
  accentG: number;
  accentB: number;
  text: string; // hex color e.g. '#c8cad0'
}

export interface ParticleSystemOptions {
  /** Canvas element to render on */
  canvas: HTMLCanvasElement;
  /** Particle count override. Default: auto (30 mobile / 60 desktop) */
  count?: number;
  /** Mouse repulsion radius in px. Default: 120 */
  repulsionRadius?: number;
  /** Callback that returns current theme colors */
  getColors?: () => ThemeColors;
  /** External anim loop add function. If not provided, uses own rAF loop */
  animLoopAdd?: (fn: () => void) => void;
  animLoopRemove?: (fn: () => void) => void;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
  color: string;
}

const DEFAULT_COLORS: ThemeColors = {
  accentR: 94,
  accentG: 186,
  accentB: 239,
  text: '#c8cad0',
};

function defaultGetColors(): ThemeColors {
  const s = getComputedStyle(document.documentElement);
  return {
    accentR: parseInt(s.getPropertyValue('--accent-r')) || DEFAULT_COLORS.accentR,
    accentG: parseInt(s.getPropertyValue('--accent-g')) || DEFAULT_COLORS.accentG,
    accentB: parseInt(s.getPropertyValue('--accent-b')) || DEFAULT_COLORS.accentB,
    text: s.getPropertyValue('--text').trim() || DEFAULT_COLORS.text,
  };
}

export class ParticleSystem {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private particles: Particle[] = [];
  private mouseX = -1000;
  private mouseY = -1000;
  private repulsionRadius: number;
  private getColors: () => ThemeColors;
  private animLoopAdd: ((fn: () => void) => void) | null = null;
  private animLoopRemove: ((fn: () => void) => void) | null = null;
  private rafId = 0;
  private useOwnLoop: boolean;

  constructor(opts: ParticleSystemOptions) {
    this.canvas = opts.canvas;
    const ctx = this.canvas.getContext('2d');
    if (!ctx) throw new Error('Failed to get 2D context from canvas');
    this.ctx = ctx;

    this.repulsionRadius = opts.repulsionRadius ?? 120;
    this.getColors = opts.getColors ?? defaultGetColors;
    this.animLoopAdd = opts.animLoopAdd ?? null;
    this.animLoopRemove = opts.animLoopRemove ?? null;
    this.useOwnLoop = !opts.animLoopAdd;

    const count = opts.count ?? (window.innerWidth < 600 ? 30 : 60);

    this.resize();
    window.addEventListener('resize', this.resize);
    document.addEventListener('mousemove', this.onMouseMove);

    for (let i = 0; i < count; i++) {
      const p: Particle = {
        x: Math.random() * (this.canvas.width || 1920),
        y: Math.random() * (this.canvas.height || 1080),
        vx: (Math.random() - 0.5) * 0.3,
        vy: -Math.random() * 0.4 - 0.1,
        r: Math.random() * 1.5 + 0.5,
        color: '',
      };
      this.assignColor(p);
      this.particles.push(p);
    }

    if (this.useOwnLoop) {
      const loop = (): void => {
        this.animate();
        this.rafId = requestAnimationFrame(loop);
      };
      this.rafId = requestAnimationFrame(loop);
    } else {
      this.animLoopAdd?.(this.animate);
    }
  }

  private resize = (): void => {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
  };

  private onMouseMove = (e: MouseEvent): void => {
    this.mouseX = e.clientX;
    this.mouseY = e.clientY;
  };

  private assignColor(p: Particle): void {
    const tc = this.getColors();
    if (Math.random() > 0.7) {
      p.color = `rgba(${tc.accentR}, ${tc.accentG}, ${tc.accentB}, ${0.2 + Math.random() * 0.3})`;
    } else {
      const hex = tc.text.replace('#', '');
      const tr = parseInt(hex.substring(0, 2), 16) || 200;
      const tg = parseInt(hex.substring(2, 4), 16) || 202;
      const tb = parseInt(hex.substring(4, 6), 16) || 208;
      p.color = `rgba(${tr}, ${tg}, ${tb}, ${0.08 + Math.random() * 0.12})`;
    }
  }

  updateColors(): void {
    for (const p of this.particles) {
      this.assignColor(p);
    }
  }

  private animate = (): void => {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    for (const p of this.particles) {
      const dx = p.x - this.mouseX;
      const dy = p.y - this.mouseY;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < this.repulsionRadius && dist > 0) {
        const force = (this.repulsionRadius - dist) / this.repulsionRadius * 0.6;
        p.x += (dx / dist) * force;
        p.y += (dy / dist) * force;
      }

      p.x += p.vx;
      p.y += p.vy;

      if (p.y < -10) { p.y = this.canvas.height + 10; p.x = Math.random() * this.canvas.width; }
      if (p.x < -10) p.x = this.canvas.width + 10;
      if (p.x > this.canvas.width + 10) p.x = -10;

      this.ctx.beginPath();
      this.ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      this.ctx.fillStyle = p.color;
      this.ctx.fill();
    }
  };

  destroy(): void {
    window.removeEventListener('resize', this.resize);
    document.removeEventListener('mousemove', this.onMouseMove);
    if (this.useOwnLoop) {
      cancelAnimationFrame(this.rafId);
    } else {
      this.animLoopRemove?.(this.animate);
    }
  }
}
