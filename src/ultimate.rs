use crate::config::*;
use std::time::Duration;

const START_BLINK: Duration = Duration::from_millis(3000);
const STOP_BLINK: Duration = Duration::from_millis(200);
const BLINK_TIME: Duration = Duration::from_millis(200);
const CAP_T: f64 = 0.1;
const SCALE_T: f64 = 0.5;

const SLOW_CHARGING: f64 = 2.0;
const CHARGE_CONST: f64 = 1.0 / ((HEIGHT - 1) as f64 * SLOW_CHARGING);

#[derive(Debug)]
pub enum Ultimate {
    Charging(f64), // progress in 0.0..1.0
    Ready,
    Active(Duration), // time remaining
}

impl Default for Ultimate {
    fn default() -> Self {
        Self::Charging(0.0)
        // Self::Charging(1.0) // for debug
    }
}

impl Ultimate {
    pub fn progress(&self) -> f64 {
        match self {
            Ultimate::Charging(p) => p.clamp(0.0, 1.0),
            Ultimate::Ready | Ultimate::Active(_) => 1.0,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Ultimate::Active(_))
    }

    pub fn update(&mut self, dt: Duration, altitude: usize) {
        match self {
            Ultimate::Charging(p) => {
                if *p >= 1.0 {
                    *self = Ultimate::Ready
                } else {
                    *p += CHARGE_CONST * altitude as f64 * dt.as_secs_f64();
                }
            }
            Ultimate::Ready => {}
            Ultimate::Active(left) => {
                *left = left.saturating_sub(dt);
                if left.is_zero() {
                    *self = Ultimate::Charging(0.0)
                }
            }
        }
    }

    pub fn activate(&mut self) -> bool {
        if matches!(self, Ultimate::Ready) {
            *self = Ultimate::Active(ULT_DURATION);
            true
        } else {
            false
        }
    }

    pub fn barrier_visible(&self) -> bool {
        match *self {
            Ultimate::Active(left) if left > START_BLINK => true,
            Ultimate::Active(left) if left > STOP_BLINK => {
                let t = left.as_secs_f64() / START_BLINK.as_secs_f64(); // 1.0 -> 0.0
                let period = BLINK_TIME.as_secs_f64() * (CAP_T + SCALE_T * t); // 1.0 -> 0.0
                // period -> 0 => phase changes faster in in time
                let phase = (left.as_secs_f64() / period) as u64;
                phase % 2 == 1
            }
            // Drop last few ms to prevent flickering
            _ => false,
        }
    }
}
