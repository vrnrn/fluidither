// Damped springs, stepped with the exact solution of the oscillator equation so
// the path does not depend on the frame rate, and retargetable without losing
// velocity. gpui/src/spring.rs is the same math, and fixtures/spring.json holds
// both to the same numbers.

import { prefersReducedMotion } from './motion.js';

export { springs } from './tokens.js';

const LANDED_DISTANCE = 0.01;
const LANDED_SPEED = 0.1;
const LONGEST_STEP = 0.05;

/** Move a spring on by `dt` seconds. Returns its new [position, velocity]. */
export function stepSpring({ omega, damping }, position, velocity, target, dt) {
  const away = position - target;
  if (damping >= 1) {
    const fade = Math.exp(-omega * dt);
    const slope = velocity + omega * away;
    return [target + fade * (away + slope * dt), fade * (velocity - omega * slope * dt)];
  }
  const rate = damping * omega;
  const ringing = omega * Math.sqrt(1 - damping * damping);
  const fade = Math.exp(-rate * dt);
  const cos = Math.cos(ringing * dt);
  const sin = Math.sin(ringing * dt);
  const swing = (velocity + rate * away) / ringing;
  return [
    target + fade * (away * cos + swing * sin),
    fade * (velocity * cos - (rate * swing + ringing * away) * sin),
  ];
}

export class Spring {
  velocity = 0;

  constructor(config, value = 0) {
    this.config = config;
    this.position = value;
    this.target = value;
  }

  get landed() {
    return this.position === this.target && this.velocity === 0;
  }

  jump(value) {
    this.position = value;
    this.target = value;
    this.velocity = 0;
  }

  /** Advance by `dt` seconds. Returns true once it has landed. */
  step(dt) {
    if (!this.landed) {
      [this.position, this.velocity] = stepSpring(this.config, this.position, this.velocity, this.target, dt);
      const near = Math.abs(this.position - this.target) < LANDED_DISTANCE;
      if (near && Math.abs(this.velocity) < LANDED_SPEED) this.jump(this.target);
    }
    return this.landed;
  }
}

// Every moving set of springs shares one frame loop, which stops when they land.
const moving = new Set();
let frame = 0;
let lastFrame = 0;

function onFrame(now) {
  frame = 0;
  const dt = Math.min((now - lastFrame) / 1000, LONGEST_STEP);
  lastFrame = now;
  for (const set of moving) {
    if (set.advance(dt)) moving.delete(set);
  }
  if (moving.size > 0) frame = requestAnimationFrame(onFrame);
}

/** Named springs that move as one and report their values each frame. */
export class Glide {
  constructor(start, config, report) {
    this.config = config;
    this.report = report;
    this.springs = Object.fromEntries(Object.entries(start).map(([name, value]) => [name, new Spring(config, value)]));
  }

  get values() {
    return Object.fromEntries(Object.entries(this.springs).map(([name, spring]) => [name, spring.position]));
  }

  /** Move to `targets`. Pass `config` for one move on another spring, such as a faster exit. */
  to(targets, config = this.config) {
    for (const [name, target] of Object.entries(targets)) {
      this.springs[name].config = config;
      if (prefersReducedMotion()) this.springs[name].jump(target);
      else this.springs[name].target = target;
    }
    this.report(this.values);
    if (!Object.values(this.springs).every((spring) => spring.landed)) {
      moving.add(this);
      if (frame === 0) {
        lastFrame = performance.now();
        frame = requestAnimationFrame(onFrame);
      }
    }
  }

  jump(targets) {
    for (const [name, target] of Object.entries(targets)) this.springs[name].jump(target);
    this.report(this.values);
  }

  advance(dt) {
    let landed = true;
    for (const spring of Object.values(this.springs)) landed = spring.step(dt) && landed;
    this.report(this.values);
    return landed;
  }

  stop() {
    moving.delete(this);
  }
}
