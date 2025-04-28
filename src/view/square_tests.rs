#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::model::node::DasherNode;
    use crate::view::{Color, DasherScreen, DasherView, Label, Orientation};
    use crate::view::square::{DasherViewSquare, NodeShape, SquareViewConfig};

    // Mock implementation of DasherScreen for testing
    pub struct MockScreen {
        width: i32,
        height: i32,
        draw_calls: RefCell<Vec<DrawCall>>,
    }

    // Enum to track different drawing operations
    #[derive(Debug, PartialEq, Clone)]
    enum DrawCall {
        Rectangle {
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            fill_color: Color,
            outline_color: Color,
            line_width: i32,
        },
        Circle {
            cx: i32,
            cy: i32,
            r: i32,
            fill_color: Color,
            line_color: Color,
            line_width: i32,
        },
        Line {
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            color: Color,
            line_width: i32,
        },
        Polygon {
            points: Vec<(i32, i32)>,
            fill_color: Color,
            outline_color: Color,
            line_width: i32,
        },
        String {
            text: String,
            x: i32,
            y: i32,
            font_size: u32,
            color: Color,
        },
        Display,
    }

    // Mock implementation of Label for testing
    struct MockLabel {
        text: String,
        wrap_size: u32,
    }

    impl Label for MockLabel {
        fn get_text(&self) -> &str {
            &self.text
        }

        fn get_wrap_size(&self) -> u32 {
            self.wrap_size
        }
    }

    impl MockScreen {
        fn new(width: i32, height: i32) -> Self {
            Self {
                width,
                height,
                draw_calls: RefCell::new(Vec::new()),
            }
        }

        fn clear_draw_calls(&self) {
            self.draw_calls.borrow_mut().clear();
        }

        fn get_draw_calls(&self) -> Vec<DrawCall> {
            self.draw_calls.borrow().clone()
        }
    }

    impl DasherScreen for MockScreen {
        fn get_width(&self) -> i32 {
            self.width
        }

        fn get_height(&self) -> i32 {
            self.height
        }

        fn make_label(&self, text: &str, wrap_size: u32) -> Box<dyn Label> {
            Box::new(MockLabel {
                text: text.to_string(),
                wrap_size,
            })
        }

        fn text_size(&self, _label: &dyn Label, font_size: u32) -> (i32, i32) {
            // Simple approximation: each character is 10x20 pixels
            let width = 10 * font_size as i32 / 24;
            let height = 20 * font_size as i32 / 24;
            (width, height)
        }

        fn draw_string(&mut self, label: &dyn Label, x: i32, y: i32, font_size: u32, color: Color) {
            self.draw_calls.borrow_mut().push(DrawCall::String {
                text: label.get_text().to_string(),
                x,
                y,
                font_size,
                color,
            });
        }

        fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, fill_color: Color, outline_color: Color, line_width: i32) {
            self.draw_calls.borrow_mut().push(DrawCall::Rectangle {
                x1,
                y1,
                x2,
                y2,
                fill_color,
                outline_color,
                line_width,
            });
        }

        fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, fill_color: Color, line_color: Color, line_width: i32) {
            self.draw_calls.borrow_mut().push(DrawCall::Circle {
                cx,
                cy,
                r,
                fill_color,
                line_color,
                line_width,
            });
        }

        fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, line_width: i32) {
            self.draw_calls.borrow_mut().push(DrawCall::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                line_width,
            });
        }

        fn draw_polygon(&mut self, points: &[(i32, i32)], fill_color: Color, outline_color: Color, line_width: i32) {
            self.draw_calls.borrow_mut().push(DrawCall::Polygon {
                points: points.to_vec(),
                fill_color,
                outline_color,
                line_width,
            });
        }

        fn display(&mut self) {
            self.draw_calls.borrow_mut().push(DrawCall::Display);
        }

        fn is_point_visible(&self, _x: i32, _y: i32) -> bool {
            true
        }
    }

    #[test]
    fn test_square_view_initialization() {
        let screen = Box::new(MockScreen::new(800, 600));
        let view = DasherViewSquare::new(screen);

        // Check default configuration
        assert_eq!(view.config().node_shape, NodeShape::Rectangle);
        assert!(view.config().x_nonlinear);
        assert!(view.config().y_nonlinear);
        assert!(view.config().text_3d);
    }

    #[test]
    fn test_square_view_custom_config() {
        let screen = Box::new(MockScreen::new(800, 600));
        let config = SquareViewConfig {
            node_shape: NodeShape::Triangle,
            x_nonlinear: false,
            x_nonlinear_factor: 2.0,
            y_nonlinear: false,
            y1: 10,
            y2: 1000,
            y3: 100,
            text_3d: false,
            text_3d_depth: 1,
            base_font_size: 16,
            font_size_scaling: 0.3,
        };
        let view = DasherViewSquare::with_config(screen, config);

        // Check custom configuration
        assert_eq!(view.config().node_shape, NodeShape::Triangle);
        assert!(!view.config().x_nonlinear);
        assert!(!view.config().y_nonlinear);
        assert!(!view.config().text_3d);
        assert_eq!(view.config().base_font_size, 16);
    }

    #[test]
    fn test_square_view_coordinate_transformations() {
        let screen = Box::new(MockScreen::new(800, 600));
        let view = DasherViewSquare::new(screen);

        // Test dasher_to_screen
        let (dasher_x, dasher_y) = (1000, 2000);
        let (screen_x, screen_y) = view.dasher_to_screen(dasher_x, dasher_y);

        // Just check that the conversion produces some output
        // The screen coordinates could be negative depending on the orientation
        assert!(screen_x != 0 || screen_y != 0);
    }

    #[test]
    fn test_square_view_orientation() {
        let screen = Box::new(MockScreen::new(800, 600));
        let mut view = DasherViewSquare::new(screen);

        // Test default orientation
        assert_eq!(view.get_orientation(), Orientation::LeftToRight);

        // Test changing orientation
        view.set_orientation(Orientation::RightToLeft);
        assert_eq!(view.get_orientation(), Orientation::RightToLeft);

        view.set_orientation(Orientation::TopToBottom);
        assert_eq!(view.get_orientation(), Orientation::TopToBottom);

        view.set_orientation(Orientation::BottomToTop);
        assert_eq!(view.get_orientation(), Orientation::BottomToTop);
    }

    #[test]
    fn test_square_view_node_shapes() {
        // Import the trait to use get_screen_for_testing
        use crate::view::square_tests::DasherViewSquareExt;

        // Test rectangle shape
        {
            let screen = Box::new(MockScreen::new(800, 600));
            let mut view = DasherViewSquare::new(screen);

            // Create a simple node for testing
            let mut node = DasherNode::new(0, None);
            node.set_bounds(0, 1000);
            let node = Rc::new(RefCell::new(node));

            view.set_node_shape(NodeShape::Rectangle);
            view.render_node(node);

            let mock_screen = view.get_screen_for_testing();
            let draw_calls = mock_screen.get_draw_calls();

            // Check that at least one rectangle was drawn
            assert!(draw_calls.iter().any(|call| matches!(call, DrawCall::Rectangle { .. })));
        }

        // Test triangle shape
        {
            let screen = Box::new(MockScreen::new(800, 600));
            let mut view = DasherViewSquare::new(screen);

            // Create a simple node for testing
            let mut node = DasherNode::new(0, None);
            node.set_bounds(0, 1000);
            let node = Rc::new(RefCell::new(node));

            view.set_node_shape(NodeShape::Triangle);
            view.render_node(node);

            let mock_screen = view.get_screen_for_testing();
            let draw_calls = mock_screen.get_draw_calls();

            // Check that at least one polygon was drawn (for triangle)
            assert!(draw_calls.iter().any(|call| matches!(call, DrawCall::Polygon { .. })));
        }

        // Test circle shape
        {
            let screen = Box::new(MockScreen::new(800, 600));
            let mut view = DasherViewSquare::new(screen);

            // Create a simple node for testing
            let mut node = DasherNode::new(0, None);
            node.set_bounds(0, 1000);
            let node = Rc::new(RefCell::new(node));

            view.set_node_shape(NodeShape::Circle);
            view.render_node(node);

            let mock_screen = view.get_screen_for_testing();
            let draw_calls = mock_screen.get_draw_calls();

            // Check that at least one circle was drawn
            assert!(draw_calls.iter().any(|call| matches!(call, DrawCall::Circle { .. })));
        }

        // Test quadric shape
        {
            let screen = Box::new(MockScreen::new(800, 600));
            let mut view = DasherViewSquare::new(screen);

            // Create a simple node for testing
            let mut node = DasherNode::new(0, None);
            node.set_bounds(0, 1000);
            let node = Rc::new(RefCell::new(node));

            view.set_node_shape(NodeShape::Quadric);
            view.render_node(node);

            let mock_screen = view.get_screen_for_testing();
            let draw_calls = mock_screen.get_draw_calls();

            // Check that at least one polygon was drawn (for quadric)
            assert!(draw_calls.iter().any(|call| matches!(call, DrawCall::Polygon { .. })));
        }
    }

    #[test]
    fn test_square_view_3d_text() {
        // Test configuration options for 3D text
        let screen = Box::new(MockScreen::new(800, 600));
        let mut view = DasherViewSquare::new(screen);

        // Test enabling 3D text
        view.config_mut().text_3d = true;
        assert!(view.config().text_3d);

        // Test setting 3D text depth
        view.config_mut().text_3d_depth = 3;
        assert_eq!(view.config().text_3d_depth, 3);

        // Test disabling 3D text
        view.config_mut().text_3d = false;
        assert!(!view.config().text_3d);
    }

    #[test]
    fn test_square_view_nonlinearity() {
        let screen = Box::new(MockScreen::new(800, 600));
        let mut view = DasherViewSquare::new(screen);

        // Test enabling X nonlinearity
        view.config_mut().x_nonlinear = true;
        assert!(view.config().x_nonlinear);

        // Test setting X nonlinearity factor
        view.config_mut().x_nonlinear_factor = 4.8;
        assert_eq!(view.config().x_nonlinear_factor, 4.8);

        // Test disabling X nonlinearity
        view.config_mut().x_nonlinear = false;
        assert!(!view.config().x_nonlinear);
    }
}

// Add this extension trait to access the screen for testing
pub trait DasherViewSquareExt {
    fn get_screen_for_testing(&self) -> &tests::MockScreen;
}

impl DasherViewSquareExt for crate::view::square::DasherViewSquare {
    fn get_screen_for_testing(&self) -> &tests::MockScreen {
        // This is unsafe and only for testing
        // It assumes the screen is a MockScreen
        unsafe {
            let screen_ref = &**self.screen();
            let screen_ptr = screen_ref as *const dyn crate::view::DasherScreen;
            &*(screen_ptr as *const tests::MockScreen)
        }
    }
}
