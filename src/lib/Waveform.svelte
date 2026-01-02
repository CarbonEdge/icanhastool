<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { isRecording } from './stores/app';

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let animationId: number | null = null;
  let bars: number[] = [];
  let recording = false;

  const BAR_COUNT = 32;
  const BAR_GAP = 2;

  isRecording.subscribe((v) => {
    recording = v;
    if (v) {
      startAnimation();
    } else {
      stopAnimation();
    }
  });

  onMount(() => {
    ctx = canvas.getContext('2d');
    bars = new Array(BAR_COUNT).fill(0);
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
  });

  onDestroy(() => {
    stopAnimation();
    window.removeEventListener('resize', resizeCanvas);
  });

  function resizeCanvas() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = rect.height * window.devicePixelRatio;
    if (ctx) {
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    }
    draw();
  }

  function startAnimation() {
    if (animationId !== null) return;
    animate();
  }

  function stopAnimation() {
    if (animationId !== null) {
      cancelAnimationFrame(animationId);
      animationId = null;
    }
    // Fade out bars
    fadeOut();
  }

  function animate() {
    if (!recording) return;

    // Simulate audio levels with random values
    // In a real implementation, this would use actual audio data
    bars = bars.map(() => Math.random() * 0.8 + 0.1);

    draw();
    animationId = requestAnimationFrame(animate);
  }

  function fadeOut() {
    const fade = () => {
      let allZero = true;
      bars = bars.map((b) => {
        const newVal = b * 0.9;
        if (newVal > 0.01) allZero = false;
        return newVal;
      });

      draw();

      if (!allZero) {
        requestAnimationFrame(fade);
      }
    };
    fade();
  }

  function draw() {
    if (!ctx || !canvas) return;

    const width = canvas.width / window.devicePixelRatio;
    const height = canvas.height / window.devicePixelRatio;

    // Clear
    ctx.clearRect(0, 0, width, height);

    // Calculate bar dimensions
    const barWidth = (width - BAR_GAP * (BAR_COUNT - 1)) / BAR_COUNT;
    const maxBarHeight = height * 0.8;

    // Draw bars
    bars.forEach((level, i) => {
      const x = i * (barWidth + BAR_GAP);
      const barHeight = level * maxBarHeight;
      const y = (height - barHeight) / 2;

      // Gradient from blue to purple
      const gradient = ctx!.createLinearGradient(0, y, 0, y + barHeight);
      gradient.addColorStop(0, '#3b82f6');
      gradient.addColorStop(1, '#8b5cf6');

      ctx!.fillStyle = gradient;
      ctx!.beginPath();
      ctx!.roundRect(x, y, barWidth, barHeight, 2);
      ctx!.fill();
    });
  }
</script>

<canvas bind:this={canvas} class="waveform"></canvas>

<style>
  .waveform {
    width: 100%;
    height: 48px;
    display: block;
  }
</style>
