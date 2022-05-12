use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub xmin: f32,
    pub ymin: f32,
    pub zmin: f32,
    pub xmax: f32,
    pub ymax: f32,
    pub zmax: f32,
}

impl Bounds {
    pub fn new() -> Self {
        Self {
            xmin: f32::MAX,
            ymin: f32::MAX,
            zmin: f32::MAX,
            xmax: f32::MIN,
            ymax: f32::MIN,
            zmax: f32::MIN,
        }
    }

    pub fn add(&mut self, p: Point3) {
        self.xmin = self.xmin.min(p.x);
        self.ymin = self.ymin.min(p.y);
        self.zmin = self.zmin.min(p.z);
        self.xmax = self.xmax.max(p.x);
        self.ymax = self.ymax.max(p.y);
        self.zmax = self.zmax.max(p.z);
    }
}

impl FromIterator<Point3> for Bounds {
    fn from_iter<I: IntoIterator<Item = Point3>>(iter: I) -> Self {
        let mut bounds = Bounds::new();

        for point in iter {
            bounds.add(point);
        }
        bounds
    }
}
