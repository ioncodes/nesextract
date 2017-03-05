extern crate bytereader;
extern crate bmp;

use bytereader::byte::ByteReader;
use bytereader::hex::Hex;
use bmp::Image;
use std::env;

#[allow(unused_assignments)]
#[allow(unused_must_use)]
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        panic!("Usage: nesextract <rom-file>");
    }
    let mut reader = ByteReader::new(args[1].to_string());
    let mut header = vec![0; 4];
    reader.read_next(&mut header);
    if header.to_hex() != "4E45531A" {
        panic!("NES header invalid");
    }
    let mut prg_count: usize = 0;
    let mut chr_count: usize = 0;
    let mut counts = vec![0; 2];
    reader.read_next(&mut counts);
    prg_count = counts[0] as usize;
    chr_count = counts[1] as usize;
    println!("PRGs: {:?}", prg_count);
    println!("CHRs: {:?}", chr_count);
    reader.jump(10); // skip the other 10 bytes
    let bytes_to_chr = prg_count as i64 * 16384;
    println!("Jumping {:?} bytes", bytes_to_chr);
    reader.jump(bytes_to_chr); // jump to the beginning of the first chr bank
    for b in 0..chr_count {
        println!("Processing CHR Bank #{:?}...", b);
        let mut set = vec![vec![0u8; 256]; 128];
        for y in 0..32 {
            for x in 0..16 {
                let mut channel_a = vec![0; 8];
                let mut channel_b = vec![0; 8];
                reader.read_next(&mut channel_a);
                reader.read_next(&mut channel_b);
                let img = decode_sprite(channel_a, channel_b);
                append_to_bitmap(&mut set, img, x * 8, y * 8);
            }
        }
        let bmp = create_image(set);
        bmp.save(&format!("set{}.bmp", b));
    }
}

fn decode_sprite(channel_a: Vec<u8>, channel_b: Vec<u8>) -> Vec<Vec<u8>> {
    let mut img = vec![vec![0u8; 8]; 8];
    for y in 0..8 {
        let mut mask = 0x80;
        let mut shift = 7;
        for x in 0..8 {
            let color = ((channel_a[y] & mask) >> shift) + ((channel_b[y] & mask) >> shift) * 2;
            mask >>= 1;
            shift -= 1;
            img[x as usize][y as usize] = color;
        }
    }
    return img;
}

fn get_color(num: u8) -> bmp::Pixel {
    match num {
        num if num == 0 => return bmp::Pixel::new(0, 0, 0),
        num if num == 1 => return bmp::Pixel::new(211, 211, 211),
        num if num == 2 => return bmp::Pixel::new(128, 128, 128),
        num if num == 3 => return bmp::Pixel::new(169, 169, 169),
        _ => return bmp::Pixel::new(0, 0, 0),
    }
}

fn append_to_bitmap(bmp: &mut Vec<Vec<u8>>, img: Vec<Vec<u8>>, x: u32, y: u32) {
    for img_y in 0..8 {
        for img_x in 0..8 {
            bmp[(x + img_x) as usize][(y + img_y) as usize] = img[img_x as usize][img_y as usize];
        }
    }
}

fn create_image(set: Vec<Vec<u8>>) -> Image {
    let mut bmp = Image::new(128, 256);
    for y in 0..256 {
        for x in 0..128 {
            bmp.set_pixel(x, y, get_color(set[x as usize][y as usize]));
        }
    }
    return bmp;
}
