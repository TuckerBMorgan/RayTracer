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

type ImageSection = (usize, usize, DynamicImage);

fn render_chunk(thread_index: usize, total_sections:usize, sender: Sender<ImageSection>){
  
        let scene_path = Path::new("scenes/test.json");
        let scene_file = File::open(scene_path).expect("File not found");
        let mut scene: Scene = serde_json::from_reader(scene_file).unwrap();

        loop {
            let image_0 = raytracer::render(&scene, thread_index, total_sections);
            let _ = sender.send((thread_index, scene.scene_count, image_0));
            let should_end = scene.update();
            if should_end != 0 {
                break;
            }
        }
}

fn save_film_to_disk(mut render_frames: Vec<ImageSection>,
                           image_width: u32, 
                           image_height: u32, 
                           number_of_frames: usize, //we don't need number_of_frames, it is a result of number_of_threads image_height image_width
                           //number_of_threads: usize,
                           mut final_image_path: &Path) {
    
    render_frames.sort_unstable_by(|a, b| 
        (a.0 + a.1 * 10000).cmp(
        &(b.0 + b.1 * 10000)
    ));

    let mut final_image_buffer : Vec<u8> = vec![];
    for ele in &render_frames {
        let mut pixel_array = ele.2.raw_pixels();
    // println!("Number of pixels added to buffer {}", pixel_array.len());
        final_image_buffer.append(&mut pixel_array);
    }

    //println!("number of pixels submitted to image saving {}", final_image_buffer.len());
    //println!("single chunk size {}", number_of_frames as u32 * image_height * image_width * 3);
    let image_as_chunks = final_image_buffer.chunks((image_height * image_width) as usize * 3);//.chunk_to_vec();
   // let mut file_name = "out0.png";
   // println!("{}", image_as_chunks.len());
    let mut count = 0;
    for chunk in image_as_chunks {
        let image_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(image_width, image_height, chunk.to_vec()).unwrap();
        let file_path = String::from("out") + &count.to_string()[..] + &String::from(".png");
        image_buf.save(&mut final_image_path.join(file_path)).unwrap();
        count+=1;
    }
}

fn render_image(scene_file_path: String, number_of_render_threads: usize, output_folder: String) {
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

    let scene_path = Path::new(&scene_file_path);
    let scene_file =   File::open(scene_path).expect("File not found");
    let scene: Scene = serde_json::from_reader(scene_file).unwrap();

    while finished_count != number_of_render_threads * (scene.deltas.len() + 1){
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
     //   println!("{}", finished_count);
    }

    let elasped = now.elapsed();
    println!("{:?}", elasped);



    save_film_to_disk(rendered_sections, scene.width, scene.height, scene.deltas.len() + 1, Path::new(&output_folder));
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
    render_image(String::from("scenes/test.json"), num_threads, String::from("./out"));
    return;
}
