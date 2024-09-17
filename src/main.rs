use ab_glyph::FontRef;
use dict::DICT4X4;
use imageproc::{
    contours::find_contours,
    contrast::adaptive_threshold,
    drawing::{draw_polygon_mut, draw_text_mut, Canvas},
    geometric_transformations::{warp, Projection},
    geometry::approximate_polygon_dp,
    image::{
        self,
        imageops::{crop_imm, grayscale, resize, rotate180, rotate180_in, rotate270, rotate270_in, rotate90, rotate90_in},
        GenericImageView, ImageBuffer, Luma, Pixel, Rgba,
    },
    point::Point,
};

mod dict;
fn main() {
    let img = image::open("./img.png").unwrap();
    // let img = img.resize(480, 640, image::imageops::FilterType::CatmullRom);
    let start = std::time::Instant::now();
    let arucos = detect_aruco(&img, Config {
        block_radius: 200,
        approximate_polygon_epsilon: 20.,
        min_edge_size: 200.,
    });
    println!("time: {:?}", start.elapsed());
    println!("{:?}", arucos);
    // let gray = img.grayscale();
    let image_buffer = grayscale(&img);
    // let image::DynamicImage::ImageLuma8(ref image_buffer) = gray else {
    //     panic!("not gray");
    // };
    let bin_img = adaptive_threshold(&image_buffer, 200);

    bin_img.save("./img_bin.png").unwrap();
    let contours = find_contours::<u32>(&bin_img);

    let mut contours_img = img.clone();
    for contour in &contours {
        for p in &contour.points {
            contours_img.draw_pixel(
                p.x,
                p.y,
                Rgba([
                    255,
                    contour.points.len() as u8,
                    contour.points.len() as u8,
                    1,
                ]),
            );
        }
    }

    contours_img.save("./img_contours.png").unwrap();

    let mut polygon_img = img.clone();
    let font = FontRef::try_from_slice(include_bytes!("../FiraCode-Light.ttf")).unwrap();
    for (contour_index, contour) in contours.iter().enumerate() {
        let points = approximate_polygon_dp(&contour.points, 20., true);
            points.iter().for_each(|p| {
                polygon_img.draw_pixel(p.x, p.y, Rgba([
                            255,
                            0,
                            0,
                            1,
                        ]));
            });
        if  points.len() == 4 &&
            is_contour_convex(&points) && 
            // edge_size(&points) > 10. && 
            contour.border_type == imageproc::contours::BorderType::Hole
            // false
        {
            let poly = points
                .iter()
                .map(|p| Point::new(p.x as i32, p.y as i32))
                .collect::<Vec<_>>();
            draw_polygon_mut(&mut polygon_img, &poly, Rgba([255, 0, 0, 255]));
            // points.iter().for_each(|p| {
            //     polygon_img.draw_pixel(p.x, p.y, Rgba([
            //                 255,
            //                 0,
            //                 0,
            //                 1,
            //             ]));
            // });
            // contours_img.draw_pixel(
            //     p.x,
            //     p.y,
            //     Rgba([
            //         255,
            //         contour.points.len() as u8,
            //         contour.points.len() as u8,
            //         1,
            //     ]),
            // );

            // for (i, p) in points.iter().enumerate() {
            //     draw_text_mut(
            //         &mut polygon_img,
            //         Rgba([0, 0, 0, 255]),
            //         p.x as i32,
            //         p.y as i32,
            //         20.,
            //         &font,
            //         &format!("{i}"),
            //     );
            // }

            // homography
            // let proj = Projection::from_control_points(
            //     [
            //         (points[0].x as f32, points[0].y as f32),
            //         (points[1].x as f32, points[1].y as f32),
            //         (points[2].x as f32, points[2].y as f32),
            //         (points[3].x as f32, points[3].y as f32),
            //     ],
            //     [(0., 0.), (100., 0.), (100., 100.), (0., 100.)],
            // )
            // .unwrap();
            // let img = warp(
            //     &bin_img,
            //     &proj,
            //     imageproc::geometric_transformations::Interpolation::Bilinear,
            //     Luma([0u8]),
            // );
            // let sub_img = crop_imm(&img, 0, 0, 100, 100);
            // // sub_img
            // //     .to_image()
            // //     .save(&format!("./sub_img_{}.png", contour_index))
            // //     .unwrap();
            // let bits_img = resize(
            //     &sub_img.to_image(),
            //     6,
            //     6,
            //     image::imageops::FilterType::Nearest,
            // );

            // // rotat
            // let data = extract_data(&bits_img);
            // if let Some(index) = DICT4X4.get(&data) {
            //     println!(
            //         "rotate270 contour_index: {contour_index} data:{:016b} index: {index}",
            //         data
            //     );
            // }
            // let bits_img = rotate90(&bits_img);
            // let data = extract_data(&bits_img);
            // if let Some(index) = DICT4X4.get(&data) {
            //     println!(
            //         "rotate270 contour_index: {contour_index} data:{:016b} index: {index}",
            //         data
            //     );
            // }
            // let bits_img = rotate180(&bits_img);
            // let data = extract_data(&bits_img);
            // if let Some(index) = DICT4X4.get(&data) {
            //     println!(
            //         "rotate270 contour_index: {contour_index} data:{:016b} index: {index}",
            //         data
            //     );
            // }
            // let bits_img = rotate270(&bits_img);
            // let data = extract_data(&bits_img);
            // if let Some(index) = DICT4X4.get(&data) {
            //     println!(
            //         "rotate270 contour_index: {contour_index} data:{:016b} index: {index}",
            //         data
            //     );
            // }
        }
    }
    polygon_img.save("./img_polygon.png").unwrap();
}

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

pub struct Config {
    pub block_radius: u32,
    pub approximate_polygon_epsilon: f64,
    pub min_edge_size: f32,
}

#[derive(Debug)]
pub struct Aruco {
    pub id: usize,
    pub corners: [Point<u32>; 4],
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
