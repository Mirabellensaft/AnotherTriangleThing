use gl;
use std;
use std::ffi::{CString, CStr};

//Program struct
pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id());
            }
        }

        unsafe { gl::LinkProgram(program_id);

        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id());

            }
        }

        Ok(Program { id: program_id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

//shader struct
pub struct Shader {
    id: gl::types::GLuint,
}

//implement from_source function for Shader:
impl Shader {
    pub fn from_source(
        source: &CStr,
        kind: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    //two helper methods so that gl::VERTEX_SHADER can be skipped

    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    // //additional constructor methods for Shader
    // let shader = Shader::from_source(
    //     &CString::from_bytes_with_nul(b"<source code here>/0").unwrap(),
    //     gl::VERTEX_SHADER
    // ).unwrap();
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    source: &CStr,
    kind: gl::types::GLenum
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };

    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success:gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {

        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        // allocate buffer of correct size
        //let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
        //fill it with len spaces
        //ascii space byte is iterated over forever, yielding infinite number of spaces, returned items are limited to len
        //buffer.extend([b' '].iter().cycle().take(len as usize));
        // convert buffer to CString
        //let error: CString = unsafe { CString::from_vec_unchecked(buffer) };
        let error = create_whitespace_cstring_with_len(len as usize);

        //with this OpenGL writes shader info log into the error value
        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }

        return Err(error.to_string_lossy().into_owned());
        //error.to_string_lossy converts CString into Rust String replacing invalid unicode with unicode error character
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    //allocate buffer of correct usize
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }

}
