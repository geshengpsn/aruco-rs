mod dict;
use dict::DICT4X4;
use imageproc::{
    contours::find_contours,
    contrast::adaptive_threshold,
    geometric_transformations::{warp, Projection},
    geometry::approximate_polygon_dp,
    image::{
        self,
        imageops::{crop_imm, grayscale, resize, rotate180_in, rotate270_in, rotate90_in},
        GenericImageView, ImageBuffer, Luma, Pixel,
    },
    point::Point,
};

fn extract_data(bits_img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> u16 {
    let mut data = 0u16;
    let mut a = 15;
    for y in 1..5 {
        for x in 1..5 {
            data += if bits_img.get_pixel(x, y).0[0] > 128 {
                1 << a
            } else {
                0
            };
            a -= 1;
        }
    }
    data
}

fn edge_size(contour: &[Point<u32>]) -> f32 {
    let p1 = contour[0];
    let p2 = contour[1];
    // length
    let dx = p1.x as f32 - p2.x as f32;
    let dy = p1.y as f32 - p2.y as f32;
    (dx * dx + dy * dy).sqrt()
}

fn is_contour_convex(contour: &[Point<u32>]) -> bool {
    let p1 = contour[0];
    let p2 = contour[1];
    let p3 = contour[2];
    let dx1 = p2.x as f32 - p1.x as f32;
    let dy1 = p2.y as f32 - p1.y as f32;
    let dx2 = p3.x as f32 - p2.x as f32;
    let dy2 = p3.y as f32 - p2.y as f32;
    let is_pos = (dx1 * dy2 - dy1 * dx2).is_sign_positive();
    for i in 0..contour.len() {
        let p1 = contour[i];
        let p2 = contour[(i + 1) % contour.len()];
        let p3 = contour[(i + 2) % contour.len()];
        let dx1 = p2.x as f32 - p1.x as f32;
        let dy1 = p2.y as f32 - p1.y as f32;
        let dx2 = p3.x as f32 - p2.x as f32;
        let dy2 = p3.y as f32 - p2.y as f32;
        let sign = (dx1 * dy2 - dy1 * dx2).is_sign_positive();
        if sign != is_pos {
            return false;
        }
    }
    true
}

#[derive(Debug)]
pub struct Aruco {
    pub id: usize,
    pub corners: [Point<u32>; 4],
}

pub struct Config {
    pub block_radius: u32,
    pub approximate_polygon_epsilon: f64,
    pub min_edge_size: f32,
}

pub fn detect_aruco<I: GenericImageView>(image: &I, config: Config) -> Vec<Aruco>
where
    I::Pixel: Pixel<Subpixel = u8>,
{
    let gray_image = grayscale(image);
    let bin_img = adaptive_threshold(&gray_image, config.block_radius);
    let contours = find_contours::<u32>(&bin_img);
    contours
        .iter()
        .filter(|contour| contour.border_type == imageproc::contours::BorderType::Hole)
        .map(|contour| approximate_polygon_dp(&contour.points, config.approximate_polygon_epsilon, true))
        .filter(|polygon| {
            // 4 corners
            polygon.len() == 4 && 
            // convex
            is_contour_convex(polygon) && 
            // min edge size
            edge_size(polygon) > config.min_edge_size
            // max edge size
            // max_edge_size(polygon) < config.max_edge_size
        })
        .filter_map(|marker| {
            let proj = Projection::from_control_points(
                [
                    (marker[0].x as f32, marker[0].y as f32),
                    (marker[1].x as f32, marker[1].y as f32),
                    (marker[2].x as f32, marker[2].y as f32),
                    (marker[3].x as f32, marker[3].y as f32),
                ],
                [(0., 0.), (100., 0.), (100., 100.), (0., 100.)],
            )
            .unwrap();
            let img = warp(
                &bin_img,
                &proj,
                imageproc::geometric_transformations::Interpolation::Bilinear,
                Luma([0u8]),
            );
            let sub_img = crop_imm(&img, 0, 0, 100, 100);
            let bits_img = resize(
                &sub_img.to_image(),
                6,
                6,
                image::imageops::FilterType::Nearest,
            );
            

            // rotat
            let data = extract_data(&bits_img);
            if let Some(index) = DICT4X4.get(&data) {
                return Some(Aruco {
                    id: *index,
                    corners: [marker[0], marker[1], marker[2], marker[3]],
                });
            }
            
            let mut rotate_bits_img = ImageBuffer::new(6, 6);
            let _ = rotate90_in(&bits_img, &mut rotate_bits_img);
            if let Some(index) = DICT4X4.get(&extract_data(&rotate_bits_img)) {
                return Some(Aruco {
                    id: *index,
                    corners: [marker[1], marker[2], marker[3], marker[0]],
                });
            }
            
            let _ = rotate180_in(&bits_img, &mut rotate_bits_img);
            if let Some(index) = DICT4X4.get(&extract_data(&rotate_bits_img)) {
                return Some(Aruco {
                    id: *index,
                    corners: [marker[2], marker[3], marker[0], marker[1]],
                });
            }

            let _ = rotate270_in(&bits_img, &mut rotate_bits_img);
            if let Some(index) = DICT4X4.get(&extract_data(&rotate_bits_img)) {
                return Some(Aruco {
                    id: *index,
                    corners: [marker[3], marker[0], marker[1], marker[2]],
                });
            }
            None
        })
        .collect()
}
