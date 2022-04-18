const FACES: usize = 6;
const SIDE: usize = 64;
const CHANNELS: usize = 4;
const BYTES: usize = FACES * SIDE * SIDE * CHANNELS;
// pub type PixelArray = [[[u8; CHANNELS]; FACES * SIDE]; SIDE];
pub type PixelArray = [u8; BYTES];

pub struct TestPattern {
    frame_number: usize,
    data: PixelArray,
}

impl TestPattern {
    pub fn new() -> Self {
        let mut new = Self {
            frame_number: 0,
            data: [0; BYTES],
        };
        for i in (0..BYTES).step_by(CHANNELS) {
            new.data[i + CHANNELS - 1] = 255;
        }
        new
    }

    pub fn next_frame(&mut self) -> &PixelArray {
        self.write_row_column(self.frame_number, 0u8);
        if self.frame_number == usize::MAX {
            self.frame_number = 0;
        } else {
            self.frame_number += 1;
        }
        self.write_row_column(self.frame_number, 255u8);
        &self.data
    }

    pub fn _current_frame(&self) -> &PixelArray {
        &self.data
    }

    fn write_row_column(&mut self, frame_number: usize, value: u8) {
        const HORIZ_CHANNEL: [usize; FACES] = [2, 0, 2, 2, 1, 2];
        const VERT_CHANNEL: [usize; FACES] = [1, 1, 1, 0, 0, 0];
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
        const BPP: usize = CHANNELS;
        const BPFR: usize = SIDE * BPP;
        const BPCR: usize = FACES * BPFR;
        fn index_4d(face: usize, row: usize, col: usize, chan: usize) -> usize {
            BPCR * row + BPFR * (FACES - face - 1) + BPP * col + chan
        }
        for face in 0..FACES {
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
}
