extern crate gl;
extern crate sdl2;
extern crate ocl;
extern crate ocl_interop;

use ocl::{util, ProQue, Buffer, MemFlags, Context};
use ocl_interop::get_properties_list;
use gl::types::*;


use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}



// Number of results to print out:
const RESULTS_TO_PRINT: usize = 20;

// Our arbitrary data set size (about a million) and coefficent:
const DATA_SET_SIZE: usize = 1 << 20;
const COEFF: f32 = 5432.1;

static KERNEL_SRC: &'static str = r#"
    __kernel void multiply_by_scalar(
                __private float coeff,
                __global float * src,
                __global float*  res)
    {
        uint const idx = get_global_id(0);
        res[idx] = src[idx] * coeff;
    }
"#;

fn main() {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 + ocl?", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .expect("AHHHH");
    let glContext = canvas.window().gl_create_context().unwrap();

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    canvas.window().gl_set_context_to_current();

    unsafe {
        gl::ClearColor(0.0, 0.5, 1.0, 1.0);
    }
    //Create an OpenCL context with the GL interop enabled
    let mut context = Context::builder()
        .properties(get_properties_list())
        .build()
        .unwrap();
    // Create a big ball of OpenCL-ness (see ProQue and ProQueBuilder docs for info):
    let ocl_pq = ProQue::builder()
        .context(context)
        .src(KERNEL_SRC)
        .dims(DATA_SET_SIZE)
        .build()
        .expect("Build ProQue");

    // Create a temporary init vector and the source buffer. Initialize them
    // with random floats between 0.0 and 20.0:
    let vec_source = util::scrambled_vec((0.0, 20.0), ocl_pq.dims().to_len());
    let source_buffer = Buffer::builder()
        .queue(ocl_pq.queue().clone())
        .flags(MemFlags::new().read_write().copy_host_ptr())
        .dims(ocl_pq.dims().clone())
        .host_data(&vec_source)
        .build()
        .unwrap();


    // Create an empty vec and buffer (the quick way) for results. Note that
    // there is no need to initialize the buffer as we did above because we
    // will be writing to the entire buffer first thing, overwriting any junk
    // data that may be there.
    let mut vec_result = vec![0.0f32; DATA_SET_SIZE];
    let result_buffer: Buffer<f32> = ocl_pq.create_buffer().unwrap();

    // Create a kernel with arguments corresponding to those in the kernel:
    let kern = ocl_pq
        .create_kernel("multiply_by_scalar")
        .unwrap()
        .arg_scl(COEFF)
        .arg_buf(&source_buffer)
        .arg_buf(&result_buffer);

    println!("Kernel global work size: {:?}", kern.get_gws());

    // Enqueue kernel:
    kern.enq().unwrap();

    // Read results from the device into result_buffer's local vector:
    result_buffer.read(&mut vec_result).enq().unwrap();

    // Check results and print the first 20:
    for idx in 0..DATA_SET_SIZE {
        if idx < RESULTS_TO_PRINT {
            println!("source[{idx}]: {:.03}, \t coeff: {}, \tresult[{idx}]: {}",
                     vec_source[idx],
                     COEFF,
                     vec_result[idx],
                     idx = idx);
        }
        assert_eq!(vec_source[idx] * COEFF, vec_result[idx]);
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    let frameLength = Duration::new(0, 1_000_000_000u32 / 60);
    let mut nextFrameTime = Instant::now() + frameLength;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            canvas.present();

            if nextFrameTime > Instant::now() {
                std::thread::sleep(nextFrameTime - Instant::now());
            }
            nextFrameTime = Instant::now() + frameLength;
        }
    }
}
