use std::time::Instant;
use std::f64::consts::PI;

use crate::{DasherInput, input::{InputFilter, Coordinates, VirtualKey}};
use crate::model::DasherModel;
use crate::view::DasherView;


/// Circle start handler configuration
#[derive(Debug, Clone)]
pub struct CircleStartConfig {
    /// Radius of the start circle
    pub radius: f64,
    /// Activation angle threshold (radians)
    pub activation_angle: f64,
    /// Minimum time in circle before activation (ms)
    pub min_dwell_time: u64,
    /// Maximum time to complete circle (ms)
    pub max_circle_time: u64,
    /// Speed multiplier when active
    pub speed_multiplier: f64,
    /// Smoothing factor for angle calculations
    pub angle_smoothing: f64,
}

impl Default for CircleStartConfig {
    fn default() -> Self {
        Self {
            radius: 50.0,
            activation_angle: PI * 1.5, // 270 degrees
            min_dwell_time: 200,
            max_circle_time: 2000,
            speed_multiplier: 1.5,
            angle_smoothing: 0.8,
        }
    }
}

/// Circle start state
#[derive(Debug)]
enum CircleState {
    /// Not in circle
    Outside,
    /// Inside circle, tracking angle
    Tracking {
        start_time: Instant,
        start_angle: f64,
        current_angle: f64,
        total_angle: f64,
        last_update: Instant,
    },
    /// Circle completed, active
    Active {
        start_time: Instant,
        angle_velocity: f64,
    },
}

/// Circle start handler
#[derive(Debug)]
pub struct CircleStartHandler {
    /// Configuration
    config: CircleStartConfig,
    /// Current state
    state: CircleState,
    /// Center coordinates
    center: Coordinates,
    /// Current coordinates
    current_coords: Coordinates,
    /// Smoothed angle velocity
    smoothed_velocity: f64,
}

impl CircleStartHandler {
    /// Create a new circle start handler
    pub fn new(config: CircleStartConfig) -> Self {
        Self {
            config,
            state: CircleState::Outside,
            center: Coordinates::default(),
            current_coords: Coordinates::default(),
            smoothed_velocity: 0.0,
        }
    }

    /// Set center coordinates
    pub fn set_center(&mut self, x: f64, y: f64) {
        self.center = Coordinates { x, y };
    }

    /// Calculate angle between two points
    fn calculate_angle(&self, p1: &Coordinates, p2: &Coordinates) -> f64 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        dy.atan2(dx)
    }

    /// Calculate distance between two points
    fn calculate_distance(&self, p1: &Coordinates, p2: &Coordinates) -> f64 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Normalize angle to [0, 2π)
    fn normalize_angle(&self, angle: f64) -> f64 {
        let mut result = angle % (2.0 * PI);
        if result < 0.0 {
            result += 2.0 * PI;
        }
        result
    }

    /// Calculate angle difference (shortest path)
    fn angle_difference(&self, a1: f64, a2: f64) -> f64 {
        let diff = self.normalize_angle(a2 - a1);
        if diff > PI {
            diff - 2.0 * PI
        } else {
            diff
        }
    }

    /// Update tracking state
    fn update_tracking(&mut self, coords: &Coordinates, now: Instant) {
        let (center, current_angle, start_time, _total_angle, _last_update) = if let CircleState::Tracking {
            start_time,
            start_angle: _,
            current_angle,
            total_angle,
            last_update,
        } = &mut self.state {
            (self.center, *current_angle, *start_time, *total_angle, *last_update)
        } else {
            return;
        };

        // Calculate new angle
        let new_angle = self.calculate_angle(&center, coords);
        let angle_diff = self.angle_difference(current_angle, new_angle);

        if let CircleState::Tracking {
            current_angle,
            total_angle,
            last_update,
            ..
        } = &mut self.state {
            // Update total angle and current angle
            *total_angle += angle_diff.abs();
            *current_angle = new_angle;
            *last_update = now;

            let dt = now.duration_since(*last_update).as_secs_f64();

            // Check if circle is complete
            if *total_angle >= self.config.activation_angle
                && now.duration_since(start_time).as_millis() >= self.config.min_dwell_time as u128
                && now.duration_since(start_time).as_millis() <= self.config.max_circle_time as u128 {
                // Transition to active state
                self.state = CircleState::Active {
                    start_time: now,
                    angle_velocity: angle_diff / dt,
                };
            }
        }
    }

    /// Check if point is inside start circle
    fn is_in_circle(&self, coords: &Coordinates) -> bool {
        // TODO: define or compute center_x/center_y
self.calculate_distance(&Coordinates { x: 0.0, y: 0.0 }, coords) <= self.config.radius
    }
}

impl InputFilter for CircleStartHandler {
    fn key_down(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Circle start doesn't use key events
    }

    fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Circle start doesn't use key events
    }

    fn supports_pause(&self) -> bool {
        true
    }

    fn pause(&mut self) {
        // self.paused = true; // field does not exist
    }

    fn unpause(&mut self) {
        // self.paused = false; // field does not exist
    }

    fn is_paused(&self) -> bool {
    false
}

    fn activate(&mut self) {
        // Reset state when activated
        self.reset();
    }

    fn deactivate(&mut self) {
        // Reset state when deactivated
        self.reset();
    }

    fn decorate_view(&mut self, _view: &mut dyn DasherView) -> bool {
        // No view decoration needed
        false
    }

    fn process(&mut self, input: &mut dyn DasherInput, _time: u64, _model: &mut DasherModel, view: &mut dyn DasherView) {
        let now = Instant::now();
        if let Some((x, y)) = input.get_dasher_coordinates(view) {
            let coords = Coordinates { x: x as f64, y: y as f64 };
            match self.state {
                CircleState::Outside => {
                    // Check if entered circle
                    if self.is_in_circle(&coords) {
                        self.state = CircleState::Tracking {
                            start_time: now,
                            start_angle: self.calculate_angle(&Coordinates { x: 0.0, y: 0.0 }, &coords),
                            current_angle: 0.0,
                            total_angle: 0.0,
                            last_update: now,
                        };
                    }
                }
                CircleState::Tracking { .. } => {
                    if !self.is_in_circle(&coords) {
                        // Left circle, reset
                        self.state = CircleState::Outside;
                    } else {
                        // Update tracking
                        self.update_tracking(&coords, now);
                    }
                }
                CircleState::Active { .. } => {
                    if !self.is_in_circle(&coords) {
                        // Left circle, stop
                        self.state = CircleState::Outside;
                        // TODO: model.stop() not implemented
                    } else {
                        // Update velocity
                        // TODO: self.update_velocity(&coords, now) not implemented
                        // TODO: model.set_velocity(self.smoothed_velocity) not implemented
                    }
                }
            }
            self.current_coords = coords.clone();
        }
    }

    fn reset(&mut self) {
        self.state = CircleState::Outside;
        self.current_coords = Coordinates::default();
        self.smoothed_velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_circle_start_basic() {
        let config = CircleStartConfig::default();
        let mut handler = CircleStartHandler::new(config);

        // Test initial state
        assert!(matches!(handler.state, CircleState::Outside));

        // Set center and test point inside circle
        handler.set_center(0.0, 0.0);
        assert!(handler.is_in_circle(&Coordinates { x: 10.0, y: 10.0 }));
        assert!(!handler.is_in_circle(&Coordinates { x: 100.0, y: 100.0 }));
    }

    #[test]
    fn test_angle_calculations() {
        let config = CircleStartConfig::default();
        let handler = CircleStartHandler::new(config);

        // Test angle calculations
        let center = Coordinates { x: 0.0, y: 0.0 };
        let p1 = Coordinates { x: 1.0, y: 0.0 };
        let p2 = Coordinates { x: 0.0, y: 1.0 };

        assert_eq!(handler.calculate_angle(&center, &p1), 0.0);
        assert!((handler.calculate_angle(&center, &p2) - PI / 2.0).abs() < 1e-10);
    }
}
