use std::time::{Duration, Instant};
use crate::{DasherInput, input::{InputFilter, InputDevice, Coordinates, VirtualKey}};
use crate::model::DasherModel;
use crate::input::filter::DasherInputExt;
use crate::view::DasherView;


/// One button dynamic filter configuration
#[derive(Debug, Clone)]
pub struct DynamicFilterConfig {
    /// Minimum time between clicks (ms)
    pub min_click_interval: u64,
    /// Maximum time between clicks for double click (ms)
    pub double_click_time: u64,
    /// Acceleration factor
    pub acceleration: f64,
    /// Deceleration factor
    pub deceleration: f64,
    /// Maximum speed
    pub max_speed: f64,
    /// Minimum speed
    pub min_speed: f64,
    /// Speed noise factor
    pub speed_noise: f64,
    /// Bias towards forward motion
    pub forward_bias: f64,
}

impl Default for DynamicFilterConfig {
    fn default() -> Self {
        Self {
            min_click_interval: 50,
            double_click_time: 250,
            acceleration: 1.5,
            deceleration: 0.8,
            max_speed: 10.0,
            min_speed: 0.1,
            speed_noise: 0.1,
            forward_bias: 0.2,
        }
    }
}

/// One button dynamic filter state
#[derive(Debug, Clone)]
enum DynamicFilterState {
    /// Waiting for first click
    Waiting,
    /// Moving forward
    Forward {
        start_time: Instant,
        speed: f64,
    },
    /// Moving backward
    Backward {
        start_time: Instant,
        speed: f64,
    },
    /// Paused after click
    Paused {
        last_click: Instant,
        last_state: Box<DynamicFilterState>,
    },
}

/// One button dynamic filter
#[derive(Debug)]
pub struct OneButtonDynamicFilter {
    /// Filter configuration
    config: DynamicFilterConfig,
    /// Current state
    state: DynamicFilterState,
    /// Last click time
    last_click: Option<Instant>,
    /// Current coordinates
    current_coords: Coordinates,
}

impl OneButtonDynamicFilter {
    /// Create a new one button dynamic filter
    pub fn new(config: DynamicFilterConfig) -> Self {
        Self {
            config,
            state: DynamicFilterState::Waiting,
            last_click: None,
            current_coords: Coordinates::default(),
        }
    }

    /// Handle button click
    fn handle_click(&mut self, now: Instant) -> bool {
        // Check minimum click interval
        if let Some(last) = self.last_click {
            if now.duration_since(last).as_millis() < self.config.min_click_interval as u128 {
                return false;
            }
        }

        // Update state based on click
        self.state = match &self.state {
            DynamicFilterState::Waiting => DynamicFilterState::Forward {
                start_time: now,
                speed: self.config.min_speed,
            },
            DynamicFilterState::Forward { speed, .. } => {
                if let Some(last) = self.last_click {
                    if now.duration_since(last).as_millis() < self.config.double_click_time as u128 {
                        // Double click - switch direction
                        DynamicFilterState::Backward {
                            start_time: now,
                            speed: *speed,
                        }
                    } else {
                        // Single click - pause
                        DynamicFilterState::Paused {
                            last_click: now,
                            last_state: Box::new(DynamicFilterState::Forward {
                                start_time: now,
                                speed: *speed,
                            }),
                        }
                    }
                } else {
                    DynamicFilterState::Paused {
                        last_click: now,
                        last_state: Box::new(DynamicFilterState::Forward {
                            start_time: now,
                            speed: *speed,
                        }),
                    }
                }
            },
            DynamicFilterState::Backward { speed, .. } => {
                if let Some(last) = self.last_click {
                    if now.duration_since(last).as_millis() < self.config.double_click_time as u128 {
                        // Double click - switch direction
                        DynamicFilterState::Forward {
                            start_time: now,
                            speed: *speed,
                        }
                    } else {
                        // Single click - pause
                        DynamicFilterState::Paused {
                            last_click: now,
                            last_state: Box::new(DynamicFilterState::Backward {
                                start_time: now,
                                speed: *speed,
                            }),
                        }
                    }
                } else {
                    DynamicFilterState::Paused {
                        last_click: now,
                        last_state: Box::new(DynamicFilterState::Backward {
                            start_time: now,
                            speed: *speed,
                        }),
                    }
                }
            },
            DynamicFilterState::Paused { last_state, .. } => {
                // Resume previous state
                match &**last_state {
                    DynamicFilterState::Forward { speed, .. } => DynamicFilterState::Forward {
                        start_time: now,
                        speed: *speed,
                    },
                    DynamicFilterState::Backward { speed, .. } => DynamicFilterState::Backward {
                        start_time: now,
                        speed: *speed,
                    },
                    _ => DynamicFilterState::Waiting,
                }
            },
        };

        self.last_click = Some(now);
        true
    }

    /// Update coordinates based on state
    fn update_coordinates(&mut self, _now: Instant, dt: Duration) {
        let dt_secs = dt.as_secs_f64();

        match &mut self.state {
            DynamicFilterState::Forward { speed, start_time: _ } => {
                // Accelerate forward
                *speed = (*speed * self.config.acceleration).min(self.config.max_speed);
                let dx = *speed * dt_secs * (1.0 + self.config.forward_bias);
                self.current_coords.x += dx;
            },
            DynamicFilterState::Backward { speed, start_time: _ } => {
                // Accelerate backward
                *speed = (*speed * self.config.acceleration).min(self.config.max_speed);
                let dx = *speed * dt_secs;
                self.current_coords.x -= dx;
            },
            DynamicFilterState::Paused { .. } => {
                // No movement while paused
            },
            DynamicFilterState::Waiting => {
                // Decelerate to stop
                self.current_coords.x *= self.config.deceleration;
            },
        }

        // Add noise
        let noise = (rand::random::<f64>() - 0.5) * self.config.speed_noise;
        self.current_coords.y += noise;
    }
}

impl InputFilter for OneButtonDynamicFilter {
    fn reset(&mut self) {
        // Reset to default state
        // Removed fields: velocity, direction, last_button_state, paused (not present in C++).
    }

    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        let now = Instant::now();

        // Handle button press
        if input.is_button_pressed(0) {
            self.handle_click(now);
        }

        // Update coordinates
        let dt = Duration::from_millis(16); // ~60 FPS
        self.update_coordinates(now, dt);

        // Apply coordinates to model
        model.apply_input_coordinates((self.current_coords.x as i64, self.current_coords.y as i64));
    }

    fn key_down(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}

    fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}

    fn supports_pause(&self) -> bool { true }

    fn pause(&mut self) {
        self.state = DynamicFilterState::Paused {
            last_click: Instant::now(),
            last_state: Box::new(DynamicFilterState::Waiting),
        };
    }

    fn unpause(&mut self) {
        if let DynamicFilterState::Paused { last_state, .. } = &self.state {
            self.state = (**last_state).clone();
        }
    }

    fn is_paused(&self) -> bool {
        matches!(self.state, DynamicFilterState::Paused { .. })
    }

    fn activate(&mut self) {
        self.reset();
    }

    fn deactivate(&mut self) {
        self.reset();
    }

    fn decorate_view(&mut self, _view: &mut dyn DasherView) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_dynamic_filter_basic() {
        let config = DynamicFilterConfig::default();
        let mut filter = OneButtonDynamicFilter::new(config);

        // Test initial state
        assert!(matches!(filter.state, DynamicFilterState::Waiting));

        // Test click handling
        let now = Instant::now();
        assert!(filter.handle_click(now));
        assert!(matches!(filter.state, DynamicFilterState::Forward { .. }));

        // Test double click
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        assert!(filter.handle_click(now));
        assert!(matches!(filter.state, DynamicFilterState::Backward { .. }));
    }

    #[test]
    fn test_dynamic_filter_coordinates() {
        let config = DynamicFilterConfig::default();
        let mut filter = OneButtonDynamicFilter::new(config);

        // Test forward movement
        let now = Instant::now();
        filter.handle_click(now);
        filter.update_coordinates(now, Duration::from_millis(100));
        assert!(filter.current_coords.x > 0.0);

        // Test backward movement
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        filter.handle_click(now);
        filter.update_coordinates(now, Duration::from_millis(100));
        assert!(filter.current_coords.x < 0.0);
    }
}
