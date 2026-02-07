use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
const AMBIENT_BACKGROUND_SETUP_JS: &str = r#"
(() => {
  const existingCleanup = window.__ruxlogAmbientBgCleanup;
  if (typeof existingCleanup === "function") {
    existingCleanup();
  }

  const canvas = document.getElementById("ambient-bg-canvas");
  if (!canvas) return false;

  const ctx = canvas.getContext("2d", { alpha: true });
  if (!ctx) return false;

  const prefersReducedMotionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  let reducedMotion = prefersReducedMotionQuery.matches;
  const devicePixelRatio = Math.min(window.devicePixelRatio || 1, 2);

  const pointer = {
    x: 0,
    y: 0,
    active: false,
  };

  let width = 0;
  let height = 0;
  let frameId = null;
  let running = true;
  let particles = [];

  function isDarkMode() {
    return document.documentElement.classList.contains("dark");
  }

  function particleCountForViewport() {
    const area = width * height;
    return Math.max(22, Math.min(66, Math.round(area / 26000)));
  }

  function createParticle() {
    const speed = 0.16 + Math.random() * 0.45;
    const angle = Math.random() * Math.PI * 2;
    return {
      x: Math.random() * width,
      y: Math.random() * height,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed,
      size: 0.9 + Math.random() * 1.8,
    };
  }

  function syncParticlePool() {
    const desired = particleCountForViewport();
    if (particles.length < desired) {
      while (particles.length < desired) {
        particles.push(createParticle());
      }
    } else if (particles.length > desired) {
      particles.length = desired;
    }
  }

  function resizeCanvas() {
    width = Math.max(window.innerWidth || 0, 1);
    height = Math.max(window.innerHeight || 0, 1);
    canvas.width = Math.floor(width * devicePixelRatio);
    canvas.height = Math.floor(height * devicePixelRatio);
    canvas.style.width = width + "px";
    canvas.style.height = height + "px";
    ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
    syncParticlePool();
    if (reducedMotion) {
      drawStaticFrame();
    }
  }

  function updateParticle(particle) {
    particle.x += particle.vx;
    particle.y += particle.vy;

    if (particle.x < -10) particle.x = width + 10;
    if (particle.x > width + 10) particle.x = -10;
    if (particle.y < -10) particle.y = height + 10;
    if (particle.y > height + 10) particle.y = -10;

    if (pointer.active) {
      const dx = particle.x - pointer.x;
      const dy = particle.y - pointer.y;
      const dist = Math.hypot(dx, dy);
      if (dist > 0 && dist < 170) {
        const influence = (1 - dist / 170) * 0.011;
        particle.vx += (dx / dist) * influence;
        particle.vy += (dy / dist) * influence;
      }
    }

    particle.vx *= 0.998;
    particle.vy *= 0.998;
  }

  function drawFrame(animateParticles = true) {
    const dark = isDarkMode();
    const nodeAlpha = dark ? 0.42 : 0.24;
    const lineAlpha = dark ? 0.18 : 0.11;
    const glowAlpha = dark ? 0.15 : 0.09;
    const rgb = dark ? "235,239,255" : "36,72,122";

    ctx.clearRect(0, 0, width, height);

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      if (animateParticles) {
        updateParticle(p);
      }
      ctx.beginPath();
      ctx.fillStyle = `rgba(${rgb},${nodeAlpha})`;
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fill();
    }

    const maxDistance = 132;
    for (let i = 0; i < particles.length; i++) {
      const a = particles[i];
      for (let j = i + 1; j < particles.length; j++) {
        const b = particles[j];
        const dx = a.x - b.x;
        const dy = a.y - b.y;
        const dist = Math.hypot(dx, dy);
        if (dist < maxDistance) {
          const alpha = (1 - dist / maxDistance) * lineAlpha;
          ctx.strokeStyle = `rgba(${rgb},${alpha})`;
          ctx.lineWidth = 0.8;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }
    }

    if (pointer.active) {
      const gradient = ctx.createRadialGradient(pointer.x, pointer.y, 0, pointer.x, pointer.y, 210);
      gradient.addColorStop(0, `rgba(${rgb},${glowAlpha})`);
      gradient.addColorStop(1, "rgba(0,0,0,0)");
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.arc(pointer.x, pointer.y, 210, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function drawStaticFrame() {
    drawFrame(false);
  }

  function frame() {
    if (!running) return;
    drawFrame(true);
    frameId = window.requestAnimationFrame(frame);
  }

  function onPointerMove(event) {
    pointer.x = event.clientX;
    pointer.y = event.clientY;
    pointer.active = true;
    if (reducedMotion) {
      drawStaticFrame();
    }
  }

  function onPointerLeave() {
    pointer.active = false;
    if (reducedMotion) {
      drawStaticFrame();
    }
  }

  function onVisibilityChange() {
    if (document.hidden) {
      if (frameId !== null) {
        window.cancelAnimationFrame(frameId);
        frameId = null;
      }
      return;
    }

    if (!reducedMotion && frameId === null && running) {
      frameId = window.requestAnimationFrame(frame);
    } else if (reducedMotion) {
      drawStaticFrame();
    }
  }

  function onMotionPreferenceChange(event) {
    reducedMotion = event.matches;
    if (reducedMotion) {
      if (frameId !== null) {
        window.cancelAnimationFrame(frameId);
        frameId = null;
      }
      drawStaticFrame();
      return;
    }

    if (!document.hidden && frameId === null && running) {
      frameId = window.requestAnimationFrame(frame);
    }
  }

  const themeObserver = new MutationObserver(() => {
    if (reducedMotion) {
      drawStaticFrame();
    }
  });

  resizeCanvas();
  if (reducedMotion) {
    drawStaticFrame();
  } else {
    drawFrame(true);
  }

  if (!reducedMotion) {
    frameId = window.requestAnimationFrame(frame);
  }

  window.addEventListener("resize", resizeCanvas, { passive: true });
  window.addEventListener("pointermove", onPointerMove, { passive: true });
  window.addEventListener("pointerleave", onPointerLeave, { passive: true });
  window.addEventListener("blur", onPointerLeave, { passive: true });
  document.addEventListener("visibilitychange", onVisibilityChange, { passive: true });
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });

  if (typeof prefersReducedMotionQuery.addEventListener === "function") {
    prefersReducedMotionQuery.addEventListener("change", onMotionPreferenceChange);
  } else if (typeof prefersReducedMotionQuery.addListener === "function") {
    prefersReducedMotionQuery.addListener(onMotionPreferenceChange);
  }

  window.__ruxlogAmbientBgCleanup = () => {
    running = false;
    if (frameId !== null) {
      window.cancelAnimationFrame(frameId);
      frameId = null;
    }
    window.removeEventListener("resize", resizeCanvas);
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerleave", onPointerLeave);
    window.removeEventListener("blur", onPointerLeave);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    themeObserver.disconnect();

    if (typeof prefersReducedMotionQuery.removeEventListener === "function") {
      prefersReducedMotionQuery.removeEventListener("change", onMotionPreferenceChange);
    } else if (typeof prefersReducedMotionQuery.removeListener === "function") {
      prefersReducedMotionQuery.removeListener(onMotionPreferenceChange);
    }
    delete window.__ruxlogAmbientBgCleanup;
  };

  return true;
})();
"#;

#[component]
pub fn AmbientCanvasBackground() -> Element {
    #[cfg(target_arch = "wasm32")]
    use_effect(|| {
        spawn(async move {
            let _ = document::eval(AMBIENT_BACKGROUND_SETUP_JS).await;
        });
    });

    #[cfg(target_arch = "wasm32")]
    use_drop(|| {
        spawn(async move {
            let _ = document::eval(
                "if (typeof window.__ruxlogAmbientBgCleanup === 'function') { window.__ruxlogAmbientBgCleanup(); }",
            )
            .await;
        });
    });

    rsx! {
        canvas {
            id: "ambient-bg-canvas",
            aria_hidden: "true",
            class: "fixed inset-0 z-40 pointer-events-none opacity-30",
            style: "mask-image: radial-gradient(ellipse 78% 64% at 50% 44%, black 45%, transparent 100%); -webkit-mask-image: radial-gradient(ellipse 78% 64% at 50% 44%, black 45%, transparent 100%);",
        }
    }
}
