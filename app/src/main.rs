extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate raytracer;
extern crate image;

use raytracer::scene::*;
use image::{ImageBuffer, Rgb, DynamicImage};//, ImageFormat

use std::env;
use std::fs::{File};//, OpenOptions};
use std::thread;
use std::path::Path;
use std::time::Instant;
use std::sync::mpsc::{channel, Sender};

type ImageSection = (usize, DynamicImage);

fn render_chunk(i: usize, total_sections: usize, sender: Sender<ImageSection>){
    let scene_path = Path::new("scenes/test.json");
    let scene_file =   File::open(scene_path).expect("File not found");
    let scene: Scene = serde_json::from_reader(scene_file).unwrap();
    let image_0 = raytracer::render(&scene, i, total_sections);
    let _ = sender.send((i, image_0));
}

fn save_image(mut render_frames: Vec<ImageSection>, image_width: u32, image_height: u32, mut final_image_path: &Path) {
    
    render_frames.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut final_image_buffer : Vec<u8> = vec![];
    
    for ele in &render_frames {
        let mut pixel_array = ele.1.raw_pixels();
        final_image_buffer.append(&mut pixel_array);
    }
    let image_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(image_width, image_height, final_image_buffer).unwrap();
    
    image_buf.save(&mut final_image_path).unwrap();
}

fn render_image(scene_file_path: String, number_of_render_threads: usize, output_file: String) {
    let now = Instant::now();

    let mut thread_handles = vec![];
    let (tx, rx) = channel();

    for i in 0..number_of_render_threads {
        let tx_clone = tx.clone();
        thread_handles.push(
            thread::Builder::new().name(i.to_string()).spawn(move || 
                render_chunk(i, number_of_render_threads, tx_clone)
            ).unwrap()
        );
    }

    for th in thread_handles {
        let _ = th.join();
    }
    
    let mut finished_count = 0;
    let mut rendered_sections = vec![];


    while finished_count != number_of_render_threads {
        let result = rx.recv();
        match result {
            Ok(ele) => {
                rendered_sections.push(ele);
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
        finished_count+=1;
    }
    
    let elasped = now.elapsed();
    println!("{:?}", elasped);


    let scene_path = Path::new(&scene_file_path);
    let scene_file =   File::open(scene_path).expect("File not found");
    let scene: Scene = serde_json::from_reader(scene_file).unwrap();

    save_image(rendered_sections, scene.width, scene.height, Path::new(&output_file));
    let save_timer = now.elapsed() - elasped;
    println!("{:?}", save_timer);

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Please rerun with a number of threads stated as program arguments");
        return;
    }
    
    let try_parse = args[1].parse::<usize>();
    let num_threads;

    match try_parse {
        Ok(val) => {
            num_threads = val;
        },
        Err(e) => {
            panic!("Error parsing thread count value: {}", e);
        }
    }
    render_image(String::from("scenes/test.json"), num_threads, String::from("./out.png"));
    return;
}
