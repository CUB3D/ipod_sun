//! A fuzzer for partially manual fuzzing of the iPod nano 5G video parser
//! Generates a set of random inputs, which are manually copied to the iPod

use std::error::Error;

//TODO: Can we fuzz the dat file format, so we don't need to mess about with loading the image files??
//TODO: Can we start with a smaller (1-frame) base image to make mutations more impactful
//TODO: can we save what mutations we have already done so we don't waste time (hash db?)

/// The base .MP4 file we use for mutation
/// This is essentially a single file corpus
const BASE_MP4: &[u8] = include_bytes!("../../original/IMG_0000.mp4");

/// The base .DAT file we use to make our mp4s show up in the video player
/// We don't know the format of this
const BASE_DAT: &[u8] = include_bytes!("../../original/IMG_0000.DAT");

/// Represents a fuzz input
struct FileData {
    /// Contents of the dat file
    dat_data: Vec<u8>,

    /// Contents of the mp4 file
    mp4_data: Vec<u8>,

    /// Name for the files, without the extension
    file_name: String,
}

impl FileData {
    /// Mutate the mp4 data
    pub fn mutate_mp4(&mut self, rng: &mut rng::XorShift) {
            // Mutate the mp4 file
            for _ in 0..rng.gen_range(0..100) {
                let idx = rng.gen_range(0..self.mp4_data.len());
                let val = rng.gen_range(0..0xff) as u8;
                self.mp4_data[idx] = val;
            } 
    }

    /// Mutate the DAT data
    pub fn mutate_dat(&mut self, rng: &mut rng::XorShift) {
            // Mutate the dat file
            for _ in 0..1000/*rng.gen_range(19..1000)*/ {

                let idx = rng.gen_range(0..self.dat_data.len());
                let val = rng.gen_range(0..0xff) as u8;
                self.dat_data[idx] = val;
            } 
    }

    /// Update the file name in the DAT file to point to the linked MP4 file
    pub fn write_file_index(&mut self, file_index: usize) {
            self.dat_data[0x20..0x24].copy_from_slice(&format!("{:04}", file_index).as_bytes());
    }

    /// Save this data to disk
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        std::fs::write(format!("./out/{}.mp4", self.file_name), &self.mp4_data)?;
        std::fs::write(format!("./out/{}.DAT", self.file_name), &self.dat_data)?;
        Ok(())
    }
}

/// Create new fuzz input
fn create_mutated_files(file_index: usize, rng: &mut rng::XorShift) -> FileData {
    // Create the data
    let mut data =  FileData {
        dat_data: BASE_DAT.to_vec(),
        mp4_data: BASE_MP4.to_vec(),
        file_name: format!("IMG_{:04}", file_index),
    };
    //data.mutate_mp4(rng);
    data.mutate_dat(rng);

    // Update the file name in the DAT
    data.write_file_index(file_index);

    data
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = rng::XorShift::new(0x22222222);

    for i in 0..400 {
        let file = create_mutated_files(i, &mut rng);
            file.save()?;
    }
    Ok(())
}

pub mod rng {
    use core::ops::{BitXor, Range};

    /// An implementation of XorShift
    pub struct XorShift {
        /// The seed for the rng
        seed: usize,
    }

    impl XorShift {
        /// Create a new instance of XorShift, with the given seed
        pub fn new(seed: usize) -> Self {
            Self { seed }
        }

        /// Generate a number in the given range
        pub fn gen_range(&mut self, rng: Range<usize>) -> usize {
            (self.gen().wrapping_add(rng.start).max(rng.start)) % rng.end
        }

        /// Generate the next number in the sequence, advancing the seed
        pub fn gen(&mut self) -> usize {
            let x = self.seed;
            let x = x.bitxor(x << 13);
            let x = x.bitxor(x >> 7);
            let x = x.bitxor(x << 17);
            assert_ne!(self.seed, x);
            self.seed = x;
            x
        }
    }
}
