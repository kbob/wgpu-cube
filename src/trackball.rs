use std::time:: {
    Duration,
    Instant,
};

use cgmath::prelude::*;
use cgmath:: {
    Matrix4,
    Quaternion,
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

const MOUSE_INACTIVE: Duration = Duration::from_millis(50);

pub trait Responder {
    fn handle_window_event(&mut self, evt: &WindowEvent) -> bool;
}

pub trait Manipulable {
    fn set_viewport_size(&mut self, size: &PhysicalSize<u32>);
    // fn set_viewport_center(&mut self, new_center: &PhysicalPosition<u32>);
    // fn reset(&mut self);
    fn mouse_down(&mut self, pos: &PhysicalPosition<f64>, t: Instant);
    fn mouse_drag(&mut self, pos: &PhysicalPosition<f64>, t: Instant);
    fn mouse_up(&mut self, t: Instant);
    fn orientation(&mut self, t: Instant) -> Matrix4<f32>;
}

#[derive(Clone, Copy, Debug)]
pub struct Trackball {
    cached_xform: Option<Matrix4<f32>>,
    cur_orientation: Quaternion<f32>,
    prev_orientation: Quaternion<f32>,
    drag_dt: Duration,
    rot_per_dt: Option<Quaternion<f32>>,
    prev_orientation_time: Instant,

    mouse_state: ElementState,
    viewport_size: PhysicalSize<u32>,
    // viewport_center: ???,
    physical_position: PhysicalPosition<f64>,
    prev_drag_pos: PhysicalPosition<f64>,
    last_drag_point: Vector3<f32>,
    last2_drag_point: Vector3<f32>,
    last_drag_time: Instant,
    last2_drag_time: Instant,
}

impl Trackball {

    pub fn new(viewport_size: &PhysicalSize<u32>) -> Self {
        let now = Instant::now();

        Self {
            cached_xform: None,
            cur_orientation: Quaternion::<f32>::one(),
            prev_orientation: Quaternion::<f32>::one(),
            rot_per_dt: Some(
                Quaternion::<f32>::from_axis_angle(
                    Vector3::<f32>::unit_y(),
                    Rad::<f32>(std::f32::consts::PI / 512.0),
                )
            ),
            drag_dt: Duration::new(0, 1_000_000_000 / 60),
            prev_orientation_time: now,

            mouse_state: ElementState::Released,
            viewport_size: *viewport_size,
            physical_position: PhysicalPosition::new(0.0, 0.0),
            prev_drag_pos: PhysicalPosition::new(0.0, 0.0),
            last_drag_point: Vector3::<f32>::unit_z(),
            last2_drag_point: Vector3::<f32>::unit_z(),
            last_drag_time: now,
            last2_drag_time: now,
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
            if r2 < 0.5f64.sqrt() {
                (1.0 - r2).sqrt()       // inside the circle: pt on unit sphere
            } else {

                0.5 / r2.sqrt()         // outside: on hyperbola
            } as f32
        ).normalize()
    }
}

impl Manipulable for Trackball {

    fn set_viewport_size(&mut self, new_size: &PhysicalSize<u32>)
    {
        self.viewport_size = *new_size;
    }

    // fn set_viewport_center(&mut self, new_center: &PhysicalPosition<u32>);

    fn mouse_down(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        self.cached_xform = None;
        self.prev_orientation = self.cur_orientation;
        self.rot_per_dt = None;
        self.prev_orientation_time = t;
        self.prev_drag_pos = *pos;
        let surface_point = self.surface_point(pos);
        self.last2_drag_point = surface_point;
        self.last_drag_point = surface_point;
        self.last2_drag_time = t - MOUSE_INACTIVE;
        self.last_drag_time = t;
    }

    fn mouse_drag(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        if self.prev_drag_pos != *pos {
            self.prev_drag_pos = *pos;
            self.last2_drag_point = self.last_drag_point;
            self.last2_drag_time = self.last_drag_time;
            self.last_drag_point = self.surface_point(pos);
            self.last_drag_time = t;
            let rotation = Quaternion::from_arc(
                self.last2_drag_point,
                self.last_drag_point,
                None
            );
            self.cur_orientation = rotation * self.cur_orientation;
            self.cached_xform = None;
        }
    }

    fn mouse_up(&mut self, t: Instant) {
        if t.duration_since(self.last_drag_time) < MOUSE_INACTIVE {
            self.cached_xform = None;
            let rotation = Quaternion::from_arc(
                self.last2_drag_point,
                self.last_drag_point,
                None
            );
            let dt = self.last_drag_time - self.last2_drag_time;
            self.rot_per_dt = Some(rotation);
            self.drag_dt = dt;
        } else {
            self.rot_per_dt = None;
        }
    }

    fn orientation(&mut self, t: Instant) -> Matrix4<f32> {
        if self.mouse_state == ElementState::Released {
            if let Some(vel) = self.rot_per_dt {
                let dt = t.duration_since(self.prev_orientation_time);
                if !dt.is_zero() {
                    let dest = vel * self.cur_orientation;
                    let amount = dt.as_secs_f32() / self.drag_dt.as_secs_f32();
                    self.cur_orientation = self.cur_orientation.nlerp(
                        dest,
                        amount,
                    );
                    self.cur_orientation = vel * self.cur_orientation;
                    self.cached_xform = None;
                } 
            }
            self.prev_orientation_time = t;
        }

        *self.cached_xform.get_or_insert_with(|| self.cur_orientation.into())
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
                if self.mouse_state == *new_state {
                    return true;
                }
                let now = Instant::now(); // event system has no timestamps :=(
                let pos = self.physical_position;
                self.mouse_state = *new_state;
                match new_state {
                    ElementState::Pressed => self.mouse_down(&pos, now),
                    ElementState::Released => self.mouse_up(now),
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
