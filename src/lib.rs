#[macro_use]
extern crate serde_derive;
extern crate image;
extern crate serde;

pub mod scene;
pub mod vector;
pub mod point;
mod rendering;
mod matrix;



use scene::{Scene};//, Color};
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba};

use rendering::{Ray, cast_ray};

pub fn render(scene: &Scene, i: usize, sections: usize) -> DynamicImage {
    let mut image = DynamicImage::new_rgb8(scene.width , scene.height / sections as u32);
   
    let mut ray_array : Vec<Vec<Ray>> = Vec::with_capacity((scene.height / sections as u32) as usize * scene.width as usize);
    let shift_unit = scene.height as usize / sections;
    
    //each render thread is asked to render some chunk of the image
    //which we treat as slices of the overal frame
    let mut array_count = 0;
    for x in 0..scene.width {
        ray_array.push(vec![]);
        for y in (shift_unit * i)..(shift_unit * i + shift_unit){//0..scene.height {
            ray_array[array_count].push(Ray::create_prime(x, y as u32, scene));
        }
        array_count += 1;
    }

    for x in 0..scene.width {
        for y in (shift_unit * i)..(shift_unit * i + shift_unit) {
            let ray = &ray_array[x as usize][y as usize - (shift_unit * i) as usize];
            image.put_pixel(x, y as u32 - (shift_unit * i) as u32, cast_ray(scene, ray, 0).to_rgba());
        }
    }
    image
}

pub fn render_into(scene: &Scene, image: &mut ImageBuffer<Rgba<u8>, &mut [u8]>) {
    for y in 0..scene.height {
        for x in 0..scene.width {
            let ray = Ray::create_prime(x, y, scene);
            image.put_pixel(x, y, cast_ray(scene, &ray, 0).to_rgba());
        }
    }
}