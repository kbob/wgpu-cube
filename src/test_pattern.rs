const TEST_PATTERN_TYPE: i32 = 1;

const FACE_COUNT: usize = 6;
const SIDE: usize = 64;
const CHANNEL_COUNT: usize = 4;
const BYTES: usize = FACE_COUNT * SIDE * SIDE * CHANNEL_COUNT;
pub type PixelArray = [u8; BYTES];

pub struct TestPattern {
    frame_number: usize,
    data: PixelArray,
}

impl TestPattern {
    pub fn new() -> Self {
        let mut new = Self {
            frame_number: 0,
            // data: [255u8; BYTES],
            data: [0; BYTES],
        };
        for i in (0..BYTES).step_by(CHANNEL_COUNT) {
            new.data[i + CHANNEL_COUNT - 1] = 255;
        }
        new
    }

    pub fn next_frame(&mut self) -> &PixelArray {
        match TEST_PATTERN_TYPE {
            0 => {
                self.frame_number += 1;
                &self.data
            }
            1 => self.next_frame_1(),
            2 => self.next_frame_2(),
            3 => self.next_frame_3(),
            _ => panic!(),
        }
    }

    fn next_frame_1(&mut self) -> &PixelArray {
        // self.write_row_column(self.frame_number, 0u8);
        self.write_row_column(self.frame_number + SIDE - 7, 0u8);
        self.write_row_column(self.frame_number + SIDE - 6, 255u8);
        self.write_row_column(self.frame_number + SIDE - 5, 63u8);
        self.write_row_column(self.frame_number + SIDE - 4, 15u8);
        self.write_row_column(self.frame_number + SIDE - 3, 3u8);
        self.write_row_column(self.frame_number + SIDE - 2, 3u8);
        self.write_row_column(self.frame_number + SIDE - 1, 15u8);
        self.write_row_column(self.frame_number + SIDE - 0, 63u8);

        if self.frame_number == usize::MAX / 2 {
            self.frame_number = 0;
        } else {
            self.frame_number += 1;
        }
        self.write_row_column(self.frame_number, 255u8);
        &self.data
    }

    pub fn current_frame(&self) -> &PixelArray {
        &self.data
    }

    fn write_row_column(&mut self, frame_number: usize, value: u8) {
        const HORIZ_CHANNEL: [usize; FACE_COUNT] = [2, 0, 2, 2, 1, 2];
        const VERT_CHANNEL: [usize; FACE_COUNT] = [1, 1, 1, 0, 0, 0];
        const HALF_SIDE: usize = SIDE / 2;

        let pos = frame_number % HALF_SIDE;
        let dir = frame_number / HALF_SIDE % 2 != 0;
        let (rc0, rc1) = match dir {
            false => (pos, SIDE - pos - 1),
            true => (HALF_SIDE - pos - 1, HALF_SIDE + pos),
        };
        // BPP:  4 bytes per pixel
        // BPFR: 64 pixels per face row
        // BPCR: 6 face rows per row
        const BPP: usize = CHANNEL_COUNT;
        const BPFR: usize = SIDE * BPP;
        const BPCR: usize = FACE_COUNT * BPFR;
        fn index_4d(face: usize, row: usize, col: usize, chan: usize) -> usize {
            BPCR * row + BPFR * (FACE_COUNT - face - 1) + BPP * col + chan
        }
        for face in 0..FACE_COUNT {
            for i in 0..SIDE {
                // Horizontal stripes
                self.data[index_4d(face, i, rc0, HORIZ_CHANNEL[face])] = value;
                self.data[index_4d(face, i, rc1, HORIZ_CHANNEL[face])] = value;

                // Vertical stripes
                self.data[index_4d(face, rc0, i, VERT_CHANNEL[face])] = value;
                self.data[index_4d(face, rc1, i, VERT_CHANNEL[face])] = value;
            }
        }
    }

    fn next_frame_2(&mut self) -> &PixelArray {
        const FACE_COLORS: [[u8; 3]; FACE_COUNT] = [
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [0, 255, 255],
            [255, 0, 255],
            [255, 255, 0],
        ];
        const HALF_SIDE: usize = SIDE / 2;
        const REP: usize = 4 * SIDE;
        if self.frame_number == 4 * SIDE {
            self.frame_number = 0;
        } else {
            self.frame_number += 1;
        }

        // BPP:  4 bytes per pixel
        // BPFR: 64 pixels per face row
        // BPCR: 6 face rows per row
        const BPP: usize = CHANNEL_COUNT;
        const BPFR: usize = SIDE * BPP;
        const BPCR: usize = FACE_COUNT * BPFR;
        fn index_4d(face: usize, row: usize, col: usize, chan: usize) -> usize {
            BPCR * row + BPFR * (FACE_COUNT - face - 1) + BPP * col + chan
        }

        let angle = cgmath::Rad(
            std::f32::consts::TAU * self.frame_number as f32 / REP as f32,
        );
        let rot = cgmath::Matrix2::<f32>::from_angle(-angle);
        for face in 0..FACE_COUNT {
            for i in 0..SIDE {
                for j in 0..SIDE {
                    let x = (j as f32 - HALF_SIDE as f32) + 0.5;
                    let y = (HALF_SIDE as f32 - i as f32) + 0.5;
                    let v = cgmath::Vector2::new(x, y);
                    let v = rot * v;
                    let color = if v.y > 0.0 {
                        FACE_COLORS[face]
                    } else {
                        [0, 0, 0]
                    };
                    self.data[index_4d(face, i, j, 0)] = color[0];
                    self.data[index_4d(face, i, j, 1)] = color[1];
                    self.data[index_4d(face, i, j, 2)] = color[2];
                }
            }
        }
        &self.data
    }

    fn next_frame_3(&mut self) -> &PixelArray {
        const FACE_COLORS: [[u8; 3]; FACE_COUNT] = [
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [0, 255, 255],
            [255, 0, 255],
            [255, 255, 0],
        ];
        if self.frame_number >= usize::MAX / 2 {
            self.frame_number = 0;
        } else {
            self.frame_number += 1;
        }
        let lit_face = self.frame_number / 16 % FACE_COUNT;
        let radius = (self.frame_number % 16 * 2) as i32;
        const BPP: usize = CHANNEL_COUNT;
        const BPFR: usize = SIDE * BPP;
        const BPCR: usize = FACE_COUNT * BPFR;
        fn index_4d(face: usize, row: usize, col: usize, chan: usize) -> usize {
            BPCR * row + BPFR * (FACE_COUNT - face - 1) + BPP * col + chan
        }

        for face in 0..FACE_COUNT {
            let color = if face == lit_face {
                FACE_COLORS[face]
            } else {
                [0, 0, 0]
            };
            for i in 0..SIDE {
                for j in 0..SIDE {
                    let r = (i as i32 - 32).abs().max((j as i32 - 32).abs());
                    let c = if r <= radius && 2 * r > radius {
                        color
                    } else {
                        [0, 0, 0]
                    };
                    self.data[index_4d(face, i, j, 0)] = c[0];
                    self.data[index_4d(face, i, j, 1)] = c[1];
                    self.data[index_4d(face, i, j, 2)] = c[2];
                }
            }
        }
        &self.data
    }
}
