#[macro_use] extern crate failure;

extern crate sdl2;
extern crate gl;
//use std::ffi::{CString, CStr};



pub mod render_gl;
pub mod resources;

use resources::Resources;
use std::path::Path;
use failure::err_msg;

use render_gl::data;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32,
}

impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        unsafe {
            gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl.VertexAttribPointer(
                0, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null() // offset of the first component
            );
            gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
            gl.VertexAttribPointer(
                1, // index of the generic vertex attribute ("layout (location = 0)")
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
            );
        }
    }
}

fn main() {

    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }

    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()
        .unwrap();


    let gl_context = window.gl_create_context().unwrap();
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    //set up shader program

    let shader_program = render_gl::Program::from_res(
        &gl, &res, "shaders/triangle"
    ).unwrap();

    let vertices: Vec<Vertex> = vec![
        Vertex { pos: (0.5, -0.5, 0.0).into(),  clr: (1.0, 0.0, 0.0).into() }, // bottom right
        Vertex { pos: (-0.5, -0.5, 0.0).into(), clr: (0.0, 1.0, 0.0).into() }, // bottom left
        Vertex { pos: (0.0,  0.5, 0.0).into(),  clr: (0.0, 0.0, 1.0).into() }  // top
    ];

    //request OpenGL to give us one buffer name as int, and write it into vertex buffer obj
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl.GenBuffers(1, &mut vbo);
    }

    // for GenBuffers we have to provide a pointer to array which it will overwrite with a new value.
    // &mut and & are pointers, so we can simply pass them along. The number of buffers must be
    // limited to 1, so it does not overwrite unknown memory nearby. Yet the unsafe block!!

    unsafe {
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER, //target
            (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr, //size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, //pointer to data
            gl::STATIC_DRAW, // usage
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    //set up vertex array object

    //Vertex Array object describes how to interpret the data in the vertices, and convert it into input
    //for the vertex shader. The shader has a single input vec3, meaning 3 f32 values in a sequence.

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
    }



    unsafe {
        gl.BindVertexArray(vao);

        //to configure the relation between VAO and VBO, VBO also needs to be bound.

        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);


        Vertex::vertex_attrib_pointers(&gl);


        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }

    //set uo shared state for window

    unsafe {
        gl.Viewport(0, 0, 900, 700);
        gl.ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // main loop

    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }
        //render window contents here
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        // draw triangle
        shader_program.set_used();
        unsafe {
            gl.BindVertexArray(vao);
            gl.DrawArrays(
                gl::TRIANGLES, //mode
                0, //starting index in the enabled arrays
                3 // number od indices to be rendered
            );
        }

        window.gl_swap_window();
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_exe_path()?;

    let sdl = sdl2::init().map_err(err_msg)?;

    Ok(())
}

pub fn failure_to_string (e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.iter_chain().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "     Which caused the following issue:");
        }
        let _ = writeln!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
