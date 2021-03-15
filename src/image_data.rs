use rand::prelude::*;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::io::Read;
use std::{fs::File, io::Write};

#[derive(Debug)]
pub enum GifSource {
    File(File),
    Vec(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    Empty,
    Rgb(u8, u8, u8),
}

impl Default for Pixel {
    fn default() -> Self {
        Self::Empty
    }
}
impl Pixel {
    #[inline]
    pub fn combine(self, other: Self, similarity: u32) -> Self {
        if other == Self::Empty {
            self
        } else {
            if self.similarity(other) > similarity {
                other
            } else {
                self
            }
        }
    }
    #[inline]
    pub fn mut_combine(&mut self, other: Self, similarity: u32) {
        *self = self.combine(other, similarity);
    }
    /// Calculates the similarity of 2 pixels
    pub fn similarity(self, other: Self) -> u32 {
        if self == other {
            return 0;
        }
        if let Self::Rgb(r1, g1, b1) = self {
            if let Self::Rgb(r2, g2, b2) = other {
                return (abs_diff(r1, r2) as u32 * 76
                    + abs_diff(g1, g2) as u32 * 150
                    + abs_diff(b1, b2) as u32 * 29)
                    / 255;
            }
        }
        return u32::MAX;

        #[inline]
        fn abs_diff(a: u8, b: u8) -> u8 {
            return (a > b) as u8 * (a.overflowing_sub(b).0)
                + (a < b) as u8 * (b.overflowing_sub(a).0);
        }
    }
    pub fn rgb_to_hex(rgb: (u8, u8, u8)) -> [u8; 6] {
        let combined: [[u8; 2]; 3] = [hex_str(rgb.0), hex_str(rgb.1), hex_str(rgb.2)];
        return unsafe { std::mem::transmute(combined) };

        #[inline]
        fn hex_str(a: u8) -> [u8; 2] {
            let lookup = b"0123456789abcdef";
            [lookup[(a / 16) as usize], lookup[(a & 0xF) as usize]]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    image: Vec<Pixel>,
    offset: (u32, u32),
    size: (u32, u32),
    /// Frame delay in 10ms
    delay: u16,
}

#[derive(Debug, Clone)]
pub struct OptimizedImage {
    pub start: Frame,
    pub frames: Vec<Frame>,
    pub corrections: Vec<Vec<(u32, u32, (u8, u8, u8))>>,
}

#[derive(Debug, Clone)]
pub struct FlutInstructions {
    /// Start frame instructions
    pub start: Vec<u8>,
    /// Frame, correction instructions and delay in 10ms
    pub frames: Vec<(Vec<u8>, Vec<SmallVec<[u8; 18]>>, u16)>,
}

impl Frame {
    pub fn combine(&self, other: &Self, similarity: u32) -> Self {
        let new_offset = (
            self.offset.0.min(other.offset.0),
            self.offset.1.min(other.offset.1),
        );
        let self_offset = (self.offset.0 - new_offset.0, self.offset.1 - new_offset.1);
        let other_offset = (other.offset.0 - new_offset.0, other.offset.1 - new_offset.1);
        let new_size = (
            (self.size.0 + self_offset.0).max(other.size.0 + other_offset.0),
            (self.size.1 + self_offset.1).max(other.size.1 + other_offset.1),
        );

        /*println!(
            "Size: {:?} {:?} -> {:?} Offset: {:?} {:?} -> {:?}",
            self.size, other.size, new_size, self.offset, other.offset, new_offset
        );*/

        //println!("Rel. Offset: {:?} {:?}", self_offset, other_offset);

        let mut new_data = vec![Pixel::Empty; (new_size.0 * new_size.1) as usize];
        self.image
            .par_iter()
            .enumerate()
            .map(|(i, self_pixel)| {
                let i = i as u32;
                let x = (i % self.size.0) + self_offset.0;
                let y = (i / self.size.0) + self_offset.1;
                ((x + new_size.0 * y) as usize, self_pixel)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(i, &self_pixel)| {
                new_data[i] = self_pixel;
            });

        other
            .image
            .par_iter()
            .enumerate()
            .map(|(i, other_pixel)| {
                let i = i as u32;
                let x = (i % other.size.0) + other_offset.0;
                let y = (i / other.size.0) + other_offset.1;
                ((x + new_size.0 * y) as usize, other_pixel)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(i, &other_pixel)| {
                new_data[i].mut_combine(other_pixel, similarity);
            });

        Frame {
            image: new_data,
            size: new_size,
            offset: new_offset,
            delay: other.delay,
        }
    }
    pub fn to_instructions<R: Rng + ?Sized>(
        &self,
        off_x: u32,
        off_y: u32,
        rng_option: &mut Option<&mut R>,
    ) -> Vec<u8> {
        let off_x = self.offset.0 + off_x;
        let off_y = self.offset.1 + off_y;
        if let Some(rng) = rng_option {
            let mut buffer = Vec::with_capacity(18 * self.image.len() / 2);
            let mut pixels: Vec<_> = self.image.par_iter().enumerate().collect();
            pixels.shuffle(*rng);
            for (i, &pixel) in pixels {
                if let Pixel::Rgb(r, g, b) = pixel {
                    let i = i as u32;
                    let x = (i % self.size.0) + off_x;
                    let y = (i / self.size.0) + off_y;
                    write_instruction(&mut buffer, x, y, (r, g, b)).unwrap();
                }
            }
            buffer
        } else {
            let mut buffer = Vec::with_capacity(18 * self.image.len() / 2);
            for (i, &pixel) in self.image.iter().enumerate() {
                if let Pixel::Rgb(r, g, b) = pixel {
                    let i = i as u32;
                    let x = (i % self.size.0) + off_x;
                    let y = (i / self.size.0) + off_y;
                    write_instruction(&mut buffer, x, y, (r, g, b)).unwrap();
                }
            }
            buffer
        }
    }
}

pub fn load_image<R: Read>(src: R) -> Vec<Frame> {
    let decode_options = {
        let mut opt = gif::DecodeOptions::new();
        opt.set_color_output(gif::ColorOutput::Indexed);
        opt
    };

    let mut gif_decoder = decode_options.read_info(src).unwrap();
    let mut frames = Vec::new();

    let g_palette = gif_decoder.global_palette().map(|p| p.to_owned());

    while let Some(frame) = gif_decoder.read_next_frame().unwrap() {
        let mut pixels = Vec::with_capacity(frame.buffer.len());
        let palette = frame
            .palette
            .as_ref()
            .unwrap_or_else(|| g_palette.as_ref().unwrap());
        for i in 0..pixels.capacity() {
            let idx = frame.buffer[i] as usize;
            pixels.push(if frame.transparent.map_or(false, |t| t == idx as u8) {
                Pixel::Empty
            } else {
                let rgb = &palette[(3 * idx)..(3 * idx + 3)];
                Pixel::Rgb(rgb[0], rgb[1], rgb[2])
            });
        }
        frames.push(Frame {
            image: pixels,
            offset: (frame.left as u32, frame.top as u32),
            size: (frame.width as u32, frame.height as u32),
            delay: frame.delay,
        });
    }

    frames
}

/// Removes unchanged pixels from frames
pub fn optimize_image(frames: Vec<Frame>, similarity: u32) -> OptimizedImage {
    let start = frames[0].clone();
    let mut intermediate = start.clone();
    let mut optimized_frames = Vec::with_capacity(frames.len());
    let mut corrections = Vec::with_capacity(frames.len());
    for _ in 0..corrections.capacity() {
        corrections.push(Vec::with_capacity(0));
    }
    optimized_frames.push(start.clone());

    for i in 1..frames.len() {
        intermediate = intermediate.combine(&frames[i], similarity);
    }
    intermediate = intermediate.combine(&frames[0], similarity);

    for i in 1..=frames.len() {
        let i = i % frames.len();
        let cmp = &frames[i];
        let combined_offset = (
            cmp.offset.0 - intermediate.offset.0,
            cmp.offset.1 - intermediate.offset.1,
        );
        let optimized_data = cmp
            .image
            .iter()
            .enumerate()
            .map(|(i, &pixel)| {
                let i = i as u32;
                let x = (i % cmp.size.0) + combined_offset.0;
                let y = (i / cmp.size.0) + combined_offset.1;
                let idx = (x + intermediate.size.0 * y) as usize;

                if intermediate.image[idx] == pixel {
                    Pixel::Empty
                } else {
                    pixel
                }
            })
            .collect();
        optimized_frames.push(Frame {
            image: optimized_data,
            ..cmp.clone()
        });
        let correction = intermediate
            .image
            .iter()
            .enumerate()
            .map(|(i, &pixel)| {
                let i = i as u32;
                let lx = i % intermediate.size.0;
                let ly = i / intermediate.size.0;
                if lx < combined_offset.0
                    || ly < combined_offset.1
                    || lx >= cmp.size.0 + combined_offset.0
                    || ly >= cmp.size.1 + combined_offset.1
                {
                    if let Pixel::Rgb(r, g, b) = pixel {
                        Some((
                            lx + intermediate.offset.0,
                            ly + intermediate.offset.1,
                            (r, g, b),
                        ))
                    } else {
                        None
                    }
                } else {
                    let x = lx - combined_offset.0;
                    let y = ly - combined_offset.1;
                    let idx = (x + cmp.size.0 * y) as usize;

                    if cmp.image[idx] == pixel {
                        if let Pixel::Rgb(r, g, b) = pixel {
                            Some((
                                lx + intermediate.offset.0,
                                ly + intermediate.offset.1,
                                (r, g, b),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();
        corrections[i] = correction;
        intermediate = intermediate.combine(cmp, similarity);
    }
    OptimizedImage {
        start,
        frames: optimized_frames,
        corrections,
    }
}

pub fn optimized_image_to_instructions<R: Rng + ?Sized>(
    image: OptimizedImage,
    off_x: u32,
    off_y: u32,
    rng_option: &mut Option<&mut R>,
) -> FlutInstructions {
    FlutInstructions {
        start: image.start.to_instructions(off_x, off_y, rng_option),
        frames: image
            .frames
            .iter()
            .zip(image.corrections)
            .map(|(frame, corrections)| {
                (
                    frame.to_instructions(off_x, off_y, rng_option),
                    {
                        let mut cr: Vec<_> = corrections
                            .par_iter()
                            .map(|&(x, y, rgb)| {
                                let mut b: SmallVec<[u8; 18]> = SmallVec::new();
                                write_instruction_smallvec(&mut b, x + off_x, y + off_y, rgb);
                                b
                            })
                            .chain(
                                frame
                                    .image
                                    .par_iter()
                                    .enumerate()
                                    .map(|(i, pixel)| {
                                        let i = i as u32;
                                        let x = (i % frame.size.0) + frame.offset.0 + off_x;
                                        let y = (i / frame.size.0) + frame.offset.1 + off_y;
                                        (x, y, pixel)
                                    })
                                    .filter(|(_x, _y, &pixel)| match pixel {
                                        Pixel::Empty => false,
                                        _ => true,
                                    })
                                    .map(|(x, y, &pixel)| {
                                        if let Pixel::Rgb(r, g, b) = pixel {
                                            let mut bytes: SmallVec<[u8; 18]> = SmallVec::new();
                                            write_instruction_smallvec(&mut bytes, x, y, (r, g, b));
                                            bytes
                                        } else {
                                            unreachable!();
                                        }
                                    }),
                            )
                            .collect();
                        if let Some(rng) = rng_option {
                            cr.shuffle(*rng);
                        }
                        cr
                    },
                    frame.delay,
                )
            })
            .collect(),
    }
}

fn write_instruction<W: Write>(
    buffer: &mut W,
    x: u32,
    y: u32,
    rgb: (u8, u8, u8),
) -> std::io::Result<usize> {
    let mut c = buffer.write(format!("PX {} {} ", x, y).as_bytes())?;
    c += buffer.write(&Pixel::rgb_to_hex(rgb))?;
    c += buffer.write(&[b'\n'])?;
    Ok(c)
}

fn write_instruction_smallvec<const N: usize>(
    buffer: &mut SmallVec<[u8; N]>,
    x: u32,
    y: u32,
    rgb: (u8, u8, u8),
) {
    buffer.extend_from_slice(format!("PX {} {} ", x, y).as_bytes());
    buffer.extend_from_slice(&Pixel::rgb_to_hex(rgb));
    buffer.push(b'\n');
}
