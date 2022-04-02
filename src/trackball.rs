use std::time:: {
    // Duration,
    Instant,
};

use cgmath::prelude::*;
use cgmath:: {
    Matrix4,
    Rad,
    Vector2,
    Vector3,
};
use winit::dpi:: {
    PhysicalPosition,
    PhysicalSize,
};
use winit::event:: {
    ElementState,
    MouseButton,
    WindowEvent,
};

pub trait Responder {
    fn handle_window_event(&mut self, evt: &WindowEvent) -> bool;
}

pub trait Manipulable {
    fn set_viewport_size(&mut self, size: &PhysicalSize<u32>);
    fn mouse_down(&mut self, pos: &PhysicalPosition<f64>, t: Instant);
    fn mouse_drag(&mut self, pos: &PhysicalPosition<f64>, t: Instant);
    fn mouse_up(&mut self, pos: &PhysicalPosition<f64>, t: Instant);
    fn orientation(&mut self, t: Instant) -> Matrix4<f32>;
}

#[derive(Clone, Copy, Debug)]
pub struct Trackball {
    cur_xform: Matrix4<f32>,
    first_xform: Matrix4<f32>,
    axis: Vector3<f32>,
    velocity: Rad<f32>,

    mouse_state: ElementState,
    viewport_size: PhysicalSize<u32>,
    physical_position: PhysicalPosition<f64>,
    first_pos: Vector3<f32>,
    last_pos: Vector3<f32>,
    last_instant: Instant,
}

impl Trackball {

    pub fn new(viewport_size: &PhysicalSize<u32>) -> Self {
        Self {
            cur_xform: Matrix4::<f32>::identity(),
            first_xform: Matrix4::<f32>::identity(),
            axis: Vector3::<f32>::unit_y(),
            velocity: Rad::<f32>(0.01),

            mouse_state: ElementState::Released,
            viewport_size: *viewport_size,
            physical_position: PhysicalPosition::new(0.0, 0.0),
            first_pos: Vector3::<f32>::unit_z(),
            last_pos: Vector3::<f32>::unit_z(),
            last_instant: Instant::now(),
        }
    }

    fn surface_point(&self, pos: &PhysicalPosition<f64>) -> Vector3<f32>
    {
        // Implements the Bell virtual trackball in
        // Henriksen, Sporing, Hornbaek
        // Virtual Trackballs Revisited
        // DOI:10.1109/TVCG.2004.1260772

        let pos = Vector2::<f64>::new(
            pos.x / (self.viewport_size.width as f64) * 2.0 - 1.0,
            pos.y / (self.viewport_size.height as f64) * -2.0 + 1.0,
        );
        let r2 = pos.x * pos.x + pos.y * pos.y;
        Vector3::new(
            pos.x as f32,
            pos.y as f32,
            if r2 < (0.5f64).sqrt() {
                (1.0 - r2).sqrt()       // inside the circle: pt on unit sphere
            } else {

                0.5 / r2.sqrt()         // outside: on hyperbola
            } as f32
        ).normalize()
    }

    fn compose_xform(&self) -> Matrix4<f32> {
        let mut xform = self.first_xform;
        if self.first_pos != self.last_pos {
            let axis = self.first_pos.cross(self.last_pos).normalize();
            let angle = cgmath::Rad(
                cgmath::dot(self.first_pos, self.last_pos).acos()
            );
            let rot = Matrix4::from_axis_angle(axis, angle);
            xform = rot * xform;
        }
        xform
    }
}

impl Manipulable for Trackball {

    fn set_viewport_size(&mut self, new_size: &PhysicalSize<u32>)
    {
        self.viewport_size = *new_size;
    }

    fn mouse_down(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        self.velocity = Rad::<f32>(0.0);
        let surface_pos = self.surface_point(pos);
        self.first_pos = surface_pos;
        self.last_pos = surface_pos;
        self.last_instant = t;
        self.cur_xform = self.first_xform;
    }

    fn mouse_drag(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        self.last_pos = self.surface_point(pos);
        self.last_instant = t;
        self.cur_xform = self.compose_xform();
    }

    fn mouse_up(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {

        // update orientation
        let new_pos = self.surface_point(pos);
        if self.last_pos != new_pos {
            self.last_pos = new_pos;
            self.last_instant = t;
        }
        self.first_xform = self.compose_xform();
    }

    fn orientation(&mut self, _t: Instant) -> Matrix4<f32> {
        if self.mouse_state == ElementState::Released &&
            self.velocity.0 != 0.0 {
            self.cur_xform =
            Matrix4::<f32>::from_axis_angle(self.axis, self.velocity) *
            self.cur_xform;
        }
        self.cur_xform
    }
}

impl Responder for Trackball {
    fn handle_window_event(&mut self, evt: &WindowEvent) -> bool {
        match evt {

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: new_state,
                ..
            } => {
                let now = Instant::now(); // event system has no timestamps :=(
                let pos = self.physical_position;
                self.mouse_state = *new_state;
                match new_state {
                    ElementState::Pressed => self.mouse_down(&pos, now),
                    ElementState::Released => self.mouse_up(&pos, now),
                }
                true
            },

            WindowEvent::CursorMoved {
                position: pos,
                ..
            } => {
                let now = Instant::now();
                self.physical_position = *pos;
                if self.mouse_state == ElementState::Pressed {
                    self.mouse_drag(pos, now);
                }
                true
            },

            _ => false,
        }
    }
}
