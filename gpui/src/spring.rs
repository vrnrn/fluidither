//! Damped springs, stepped with the exact solution of the oscillator equation so
//! the path does not depend on the frame rate, and retargetable without losing
//! velocity. `web/src/core/spring.js` is the same math, and `fixtures/spring.json`
//! holds both to the same numbers.

use crate::generated::SpringConfig;

const LANDED_DISTANCE: f64 = 0.01;
const LANDED_SPEED: f64 = 0.1;

/// The longest step one frame may take, so a stall never flings anything.
pub const LONGEST_STEP: f64 = 0.05;

/// Move a spring on by `dt` seconds. Returns its new position and velocity.
pub fn step_spring(
    config: &SpringConfig,
    position: f64,
    velocity: f64,
    target: f64,
    dt: f64,
) -> (f64, f64) {
    let SpringConfig { omega, damping, .. } = *config;
    let away = position - target;
    if damping >= 1.0 {
        let fade = (-omega * dt).exp();
        let slope = velocity + omega * away;
        return (
            target + fade * (away + slope * dt),
            fade * (velocity - omega * slope * dt),
        );
    }
    let rate = damping * omega;
    let ringing = omega * (1.0 - damping * damping).sqrt();
    let fade = (-rate * dt).exp();
    let (sin, cos) = (ringing * dt).sin_cos();
    let swing = (velocity + rate * away) / ringing;
    (
        target + fade * (away * cos + swing * sin),
        fade * (velocity * cos - (rate * swing + ringing * away) * sin),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub config: SpringConfig,
    pub position: f64,
    pub velocity: f64,
    pub target: f64,
}

impl Spring {
    pub fn new(config: SpringConfig, value: f64) -> Self {
        Self { config, position: value, velocity: 0.0, target: value }
    }

    pub fn landed(&self) -> bool {
        self.position == self.target && self.velocity == 0.0
    }

    pub fn jump(&mut self, value: f64) {
        self.position = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Advance by `dt` seconds. Returns true once it has landed.
    pub fn step(&mut self, dt: f64) -> bool {
        if !self.landed() {
            (self.position, self.velocity) =
                step_spring(&self.config, self.position, self.velocity, self.target, dt);
            let near = (self.position - self.target).abs() < LANDED_DISTANCE;
            if near && self.velocity.abs() < LANDED_SPEED {
                self.jump(self.target);
            }
        }
        self.landed()
    }
}
