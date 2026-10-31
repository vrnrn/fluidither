// One clock for everything that animates continuously. It runs a single
// animation frame loop, only while something is visible, so several dithers on
// a page tick together and a hidden page does no work.

const reducedQuery = typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;

export const prefersReducedMotion = () => reducedQuery?.matches ?? false;

const LONGEST_STEP = 0.1;
const watchers = new Set();
const byElement = new WeakMap();
let frame = 0;
let lastTick = 0;
let visibility = null;

function tick(now) {
  frame = 0;
  const step = Math.min((now - lastTick) / 1000, LONGEST_STEP);
  lastTick = now;
  let anyVisible = false;
  for (const watcher of watchers) {
    if (!watcher.visible) continue;
    anyVisible = true;
    watcher.time += step;
    watcher.wait -= step;
    if (watcher.wait <= 0.002) {
      watcher.wait = Math.max(watcher.wait + watcher.interval, 0);
      watcher.draw(watcher.time);
    }
  }
  if (anyVisible) frame = requestAnimationFrame(tick);
}

function wake() {
  const blocked = document.hidden || prefersReducedMotion();
  if (blocked && frame !== 0) {
    cancelAnimationFrame(frame);
    frame = 0;
  } else if (!blocked && frame === 0 && [...watchers].some((watcher) => watcher.visible)) {
    lastTick = performance.now();
    frame = requestAnimationFrame(tick);
  }
}

function stillFrames() {
  if (prefersReducedMotion()) for (const watcher of watchers) watcher.draw(0);
  wake();
}

function startWatching() {
  visibility = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const watcher = byElement.get(entry.target);
        if (watcher) watcher.visible = entry.isIntersecting;
      }
      wake();
    },
    { rootMargin: '120px' },
  );
  document.addEventListener('visibilitychange', wake);
  reducedQuery?.addEventListener('change', stillFrames);
}

function stopWatching() {
  visibility.disconnect();
  visibility = null;
  document.removeEventListener('visibilitychange', wake);
  reducedQuery?.removeEventListener('change', stillFrames);
}

/**
 * Calls `draw(seconds)` up to `fps` times a second while `element` is on screen.
 * Under reduced motion it draws one still frame instead. Returns a stop function.
 */
export function animate(element, draw, fps) {
  if (!visibility) startWatching();
  const watcher = { draw, interval: 1 / fps, time: 0, visible: false, wait: 0 };
  watchers.add(watcher);
  byElement.set(element, watcher);
  visibility.observe(element);
  if (prefersReducedMotion()) draw(0);
  return () => {
    watchers.delete(watcher);
    visibility.unobserve(element);
    if (watchers.size === 0) stopWatching();
  };
}
