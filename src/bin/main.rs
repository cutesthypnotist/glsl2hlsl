extern crate glsl2hlsl;
use glsl2hlsl::*;
use glsl2hlsl::ShaderType;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: glsl2hlsl <filename>");
        return;
    }

    let path = std::path::Path::new(args[1].as_str());
    let glsl = std::fs::read_to_string(path).expect("Error reading file");
    let shader = download_shader("flfSzl").unwrap();
    
    //https://www.shadertoy.com/view/flfSzl
    // let common = get_common(&shader);
    // let buffers = get_buffers(&shader);
    // reset_buffer_num();
    // let mut shader_files = get_shader_file(&shader, false, false, common, buffers);
    
    let mut shader_files = get_files(&shader, false, false);
    
    
    let towrite = shader_files.iter().map(|f| f.contents.clone()).collect::<Vec<_>>().concat();
    //let towrite = buffers.iter().map(|f| f.1.clone()).collect::<Vec<_>>().concat();
    //let compiled = transpile(glsl, false, false, ShaderType::MainImage(format!("Converted"), None, vec![]));

    let mut arg = args[1].clone();
    arg.push_str(".shader");
    let path = std::path::Path::new(arg.as_str());
    std::fs::write(path, towrite.clone()).expect("Error writing file");
}
