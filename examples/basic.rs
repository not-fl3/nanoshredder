use nanoshredder::{Shader, ShaderTy};

fn main() {
    let mut shader = Shader::new(SOURCE).unwrap();

    shader.add_attribute("position", ShaderTy::Vec3);
    shader.add_attribute("texcoord", ShaderTy::Vec2);

    shader.add_uniform("Projection", ShaderTy::Mat4);
    shader.add_uniform("Model", ShaderTy::Mat4);

    shader.compile().unwrap();

    let (glsl_vertex, glsl_pixel) = shader.generate_glsl();
    println!("glsl, vertex: {}", glsl_vertex);
    println!("glsl, pixel: {}", glsl_pixel);

    let metal = shader.generate_metal();
    println!("metal: {}", metal);

    let hlsl = shader.generate_hlsl();
    println!("hlsl: {}", hlsl);
}

const SOURCE: &'static str = r#"
varying uv: vec2
        
fn vertex(self) -> vec4 {
    self.uv = vec2(0, 1);
    return self.Projection * self.Model * vec4(self.position, 1);
}
        
fn pixel(self) -> vec4 {
    return #f0f
}
"#;

