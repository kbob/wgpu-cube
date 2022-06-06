const MAX_POLYGONS: usize = 19; // unintuitive
const MAX_VERTICES: usize = 16;

struct Splitter {
    polygons: Vec<Polygon>,
    vertices: Vec<Vertex>,
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug)]
struct Line {
    closest_point: Vertex,
}

#[derive(Clone, Debug)]
struct Polygon {
    tag: [i8; 3],
    vertices: Vec<usize>,
}

impl Splitter {
    fn new() -> Self {
        Self {
            polygons: Vec::with_capacity(MAX_POLYGONS),
            vertices: Vec::with_capacity(MAX_VERTICES),
        }
    }
    fn add_rectangle(&mut self, xmin: f32, ymin: f32, xmax: f32, ymax: f32) {
        let i0 = self.vertices.len();
        self.vertices.push(Vertex { x: xmin, y: ymin });
        self.vertices.push(Vertex { x: xmax, y: ymin });
        self.vertices.push(Vertex { x: xmax, y: ymax });
        self.vertices.push(Vertex { x: xmin, y: ymax });
        self.polygons
            .push(Polygon::new(&[i0, i0 + 1, i0 + 2, i0 + 3]));
    }
    fn split(&mut self, line: &Line) {
        let replaced = self
            .polygons
            .iter()
            .enumerate()
            .filter_map(|(i, p)| self.intersects_poly(p, line).then(|| i))
            .collect::<Vec<_>>();
        for poly in replaced.iter().rev() {}
    }
    fn intersects_poly(&self, poly: &Polygon, line: &Line) -> bool {
        let left_count = poly
            .vertices
            .iter()
            .filter(|&&vi| self.vertices[vi].is_left_of(line))
            .count();

        todo!();
    }
}

impl Polygon {
    fn new(indices: &[usize]) -> Self {
        let tag = [0; 3];
        let mut vertices = Vec::with_capacity(indices.len());
        vertices.extend_from_slice(indices);
        Self { tag, vertices }
    }
}

impl Line {
    fn from_closest_point(closest_point: Vertex) -> Self {
        Self { closest_point }
    }
}

impl Vertex {
    fn is_left_of(&self, line: &Line) -> bool {
        todo!("this function name is bad")
    }
}
