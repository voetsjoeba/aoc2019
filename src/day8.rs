// vim: set ai et ts=4 sts=4 sw=4:
use crate::util;
use std::ops::{Index, IndexMut};

pub fn main() {
    let line: String = util::file_read_lines("input/day8.txt").into_iter().next().unwrap();
    let data: Vec<u32> = line.chars().map(|c| c.to_string().parse().unwrap()).collect();
    part1(&data);
    part2(&data);
}

#[allow(unused)]
struct Layer {
    order: u32,
    width: usize,
    height: usize,
    data: Vec<u32>,
}
impl Layer {
    pub fn new(order: u32, w: usize, h: usize, data: &[u32]) -> Self {
        Self {
            order,
            width: w,
            height: h,
            data: data.to_vec(),
        }
    }
}
impl Index<(usize,usize)> for Layer {
    type Output = u32;
    fn index(&self, idx: (usize,usize)) -> &Self::Output { // x,y
        &self.data[idx.1*self.width + idx.0]
    }
}
impl IndexMut<(usize,usize)> for Layer {
    fn index_mut(&mut self, idx: (usize,usize)) -> &mut Self::Output {
        &mut self.data[idx.1*self.width + idx.0]
    }
}
struct Image {
    pub width: usize,
    pub height: usize,
    pub layers: Vec<Layer>,
}
impl Image {
    pub fn new(w: usize, h: usize, data: &Vec<u32>) -> Self {
        Self {
            width: w,
            height: h,
            layers: data.chunks((w * h) as usize)
                        .enumerate()
                        .map(|(n,chunk)| Layer::new(n as u32, w, h, &chunk))
                        .collect()
        }
    }
    pub fn flatten_layers(&mut self) {
        let mut output_layer = Layer::new(0, self.width, self.height, &vec![2; (self.width*self.height) as usize]);
        for y in 0..self.height {
            for x in 0..self.width {
                // look through the pixels from the top layer to the bottom one, and return the first one that's
                // either black or white.
                for layer in &self.layers {
                    let pixel = layer[(x,y)];
                    if pixel == 2 { continue; }
                    output_layer[(x,y)] = pixel;
                    break;
                }
            }
        }
        self.layers = vec![output_layer];
    }
}

fn part1(data: &Vec<u32>) {
    let mut img = Image::new(25, 6, data);

    // sort by amount of 0 digits in the layers
    img.layers.sort_by_key(|ly| ly.data.iter().filter(|&&d| d == 0).count());
    let layer = &img.layers[0];
    let count1 = layer.data.iter().filter(|&&d| d==1).count();
    let count2 = layer.data.iter().filter(|&&d| d==2).count();
    println!("{}", count1*count2);
}
fn part2(data: &Vec<u32>) {
    let mut img = Image::new(25, 6, data);
    img.flatten_layers();

    for y in 0..img.height {
        for x in 0..img.width {
            print!("{}", match img.layers[0][(x,y)] {
                0 => " ",
                1 => "x",
                2 => "?",
                _ => panic!(""),
            });
        }
        println!("");
    }
}
