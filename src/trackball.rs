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

// cur_orientation: Quaternion<f32>,
// prev_orientation: Quaternion<f32>,
// drag_dt: Duration,
// cached_xform: Matrix4<f32>,
// axis: Vector3<f32>,
// velocity: Rad<f32>,
// last_orientation_time: Instant,

// last_physical_pos
// cur_drag_point
// prev_drag_point
// last_drag_instant

// viewport_size
// viewport_center  (what units?)
// mouse_state

// cur - 
// last - one before cur
// prev - one before last

#[derive(Clone, Copy, Debug)]
pub struct Trackball {
    cached_xform: Option<Matrix4<f32>>,     // new
    cur_orientation: Quaternion<f32>,       // new
    prev_orientation: Quaternion<f32>,      // new
    drag_dt: Duration,                      // new
    rot_per_dt: Option<Quaternion<f32>>,    // new
    prev_orientation_time: Instant,         // new
    cur_xform: Matrix4<f32>,                // XXX deprecate
    first_xform: Matrix4<f32>,              // XXX deprecate
    axis: Vector3<f32>,                     // XXX deprecate
    velocity: Rad<f32>,                     // XXX deprecate

    mouse_state: ElementState,
    viewport_size: PhysicalSize<u32>,
    // viewport_center: ???,                // new
    physical_position: PhysicalPosition<f64>,
    prev_drag_pos: PhysicalPosition<f64>,   // new
    last_drag_point: Vector3<f32>,          // new
    last2_drag_point: Vector3<f32>,         // new
    last_drag_time: Instant,                // new
    last2_drag_time: Instant,               // new
    first_pos: Vector3<f32>,                // XXX deprecate
    last_pos: Vector3<f32>,                 // XXX deprecate
    last_instant: Instant,                  // XXX deprecate
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
                    Rad::<f32>(std::f32::consts::PI / 256.0),
                )
            ),
            drag_dt: Duration::new(0, 1_000_000_000 / 60),
            prev_orientation_time: now,
            cur_xform: Matrix4::<f32>::identity(),
            first_xform: Matrix4::<f32>::identity(),
            axis: Vector3::<f32>::unit_y(),
            // velocity for one rotation every 512 frames (8.5 seconds).
            velocity: Rad::<f32>(std::f32::consts::PI / 256.0),

            mouse_state: ElementState::Released,
            viewport_size: *viewport_size,
            physical_position: PhysicalPosition::new(0.0, 0.0),
            prev_drag_pos: PhysicalPosition::new(0.0, 0.0),
            last_drag_point: Vector3::<f32>::unit_z(),
            last2_drag_point: Vector3::<f32>::unit_z(),
            last_drag_time: now,
            last2_drag_time: now,
            first_pos: Vector3::<f32>::unit_z(),
            last_pos: Vector3::<f32>::unit_z(),
            last_instant: now,
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

    fn compose_xform(&self) -> Matrix4<f32> {   // XXX deprecate
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

    // fn rotation(
    //     &self,
    //     src: Vector3<f32>,
    //     dst: Vector3<f32>
    // ) -> Quaternion<f32> {
    //     Quaternion::<f32>::from_arc(src, dst, None)
    // }
}

impl Manipulable for Trackball {

    fn set_viewport_size(&mut self, new_size: &PhysicalSize<u32>)
    {
        self.viewport_size = *new_size;
    }

    // fn set_viewport_center(&mut self, new_center: &PhysicalPosition<u32>);

    // on mouse down, stop motion, record starting point.
    //      last_orientation = current_orientation = surface_point(pos)
    //      velocity = 0

    fn mouse_down(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        self.velocity = Rad::<f32>(0.0);
        let surface_pos = self.surface_point(pos);
        self.first_pos = surface_pos;
        self.last_pos = surface_pos;
        self.last_instant = t;
        self.first_xform = self.cur_xform;


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

    // on drag, calculate rotation; save time and current orientation.
    //      if pos != last_phys_pos
    //          prev_pos = last_pos
    //          last_pos = pos
    //          axis, angle = calc_xform(prev_pos, last_pos)

    fn mouse_drag(&mut self, pos: &PhysicalPosition<f64>, t: Instant) {
        self.last_pos = self.surface_point(pos);
        self.last_instant = t;
        self.cur_xform = self.compose_xform();


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

    // on mouse up,
    //      if mouse has recently moved,
    //          calculate axis and velocity based on last two drag events
    //      else
    //          velocity = 0
    //      last_xform = ...
    //      last_instant = ...
    fn mouse_up(&mut self, t: Instant) {
        // self.cur_xform = self.compose_xform();


        if t.duration_since(self.last_drag_time) < MOUSE_INACTIVE {
            self.cached_xform = None;
            let rotation = Quaternion::from_arc(
                self.last2_drag_point,
                self.last_drag_point,
                None
            );
            // println!("last2 = {:?} last = {:?}", self.last2_drag_point, self.last_drag_point);
            // println!("rotation = {:?}", rotation);
            let dt = self.last_drag_time - self.last2_drag_time;
            self.rot_per_dt = Some(rotation);
            self.drag_dt = dt;
        } else {
            self.rot_per_dt = None;
        }
    }

    // multiply velocity by (now - last event time).
    // update last event time.
    fn orientation(&mut self, t: Instant) -> Matrix4<f32> {
        let spot = self.prev_orientation_time;  // XXX


        if self.mouse_state == ElementState::Released &&
            self.velocity.0 != 0.0 {
            self.cur_xform =
                Matrix4::<f32>::from_axis_angle(self.axis, self.velocity) *
                self.cur_xform;
            self.prev_orientation_time = t;
        }



        if self.mouse_state == ElementState::Released {
            if let Some(vel) = self.rot_per_dt {
                // let dt = t.duration_since(self.prev_orientation_time);
                let dt = t.duration_since(spot);
                if !dt.is_zero() {
                    let dest = vel * self.cur_orientation;
                    let amount = dt.as_secs_f32() / self.drag_dt.as_secs_f32();
                    self.cur_orientation = self.cur_orientation.nlerp(
                        dest,
                        amount,
                    );
                    self.cur_orientation = vel * self.cur_orientation;
                    // println!("orient = {:?}", self.cur_orientation);
                    self.cached_xform = None;
                } 
            }

        }
        self.prev_orientation_time = t;


        if false {
            self.cur_xform
        } else {
          *self.cached_xform.get_or_insert_with(|| self.cur_orientation.into())
        }
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
