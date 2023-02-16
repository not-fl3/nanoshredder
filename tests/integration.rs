#![allow(warnings)]

const SOURCE: &'static str = r#"
    DrawQuad: {{DrawQuad}} {
        varying pos: vec2
        
        fn clip_and_transform_vertex(self)->vec4{
            let clipped: vec2 = clamp(
                self.geom_pos * self.rect_size + self.rect_pos,
                self.draw_clip.xy,
                self.draw_clip.zw
            )
            self.pos = (clipped - self.rect_pos) / self.rect_size
            // only pass the clipped position forward
            return self.camera_projection * (self.camera_view * (self.view_transform * vec4(
                clipped.x,
                clipped.y,
                self.draw_depth + self.draw_zbias,
                1.
            )))
        }

        fn transform_vertex(self)->vec4{
            let clipped: vec2 = self.geom_pos * self.rect_size + self.rect_pos;
            
            self.pos = (clipped - self.rect_pos) / self.rect_size
            // only pass the clipped position forward
            return self.camera_projection * (self.camera_view * (self.view_transform * vec4(
                clipped.x,
                clipped.y,
                self.draw_depth + self.draw_zbias,
                1.
            )))
        }
        
        fn vertex(self) -> vec4 {
            return self.clip_and_transform_vertex()
        }
        
        fn pixel(self) -> vec4 {
            return #f0f
        }
    }
"#;

const GLSL_OUTPUT: &'static str = r#"
VERTEXSHADER

uniform float draw_table[1];
float ds_draw_zbias;

uniform float pass_table[50];
mat4 ds_camera_projection;
mat4 ds_camera_view;
mat4 ds_camera_inv;
float ds_dpi_factor;
float ds_dpi_dilate;

uniform float view_table[16];
mat4 ds_view_transform;




varying vec2 packed_varying_0;

vec2 ds_pos=vec2(0.0);

void main() {
    ds_draw_zbias = draw_table[0];

    ds_camera_projection = mat4(pass_table[0], pass_table[1], pass_table[2], pass_table[3], pass_table[4], pass_table[5], pass_table[6], pass_table[7], pass_table[8], pass_table[9], pass_table[10], pass_table[11], pass_table[12], pass_table[13], pass_table[14], pass_table[15]);
    ds_camera_view = mat4(pass_table[16], pass_table[17], pass_table[18], pass_table[19], pass_table[20], pass_table[21], pass_table[22], pass_table[23], pass_table[24], pass_table[25], pass_table[26], pass_table[27], pass_table[28], pass_table[29], pass_table[30], pass_table[31]);
    ds_camera_inv = mat4(pass_table[32], pass_table[33], pass_table[34], pass_table[35], pass_table[36], pass_table[37], pass_table[38], pass_table[39], pass_table[40], pass_table[41], pass_table[42], pass_table[43], pass_table[44], pass_table[45], pass_table[46], pass_table[47]);
    ds_dpi_factor = pass_table[48];
    ds_dpi_dilate = pass_table[49];

    ds_view_transform = mat4(view_table[0], view_table[1], view_table[2], view_table[3], view_table[4], view_table[5], view_table[6], view_table[7], view_table[8], view_table[9], view_table[10], view_table[11], view_table[12], view_table[13], view_table[14], view_table[15]);


    gl_Position = fn_0_5_vertex();

    packed_varying_0.xy = ds_pos.xy;
}
PIXELSHADER

uniform float draw_table[1];
float ds_draw_zbias;

uniform float pass_table[50];
mat4 ds_camera_projection;
mat4 ds_camera_view;
mat4 ds_camera_inv;
float ds_dpi_factor;
float ds_dpi_dilate;

uniform float view_table[16];
mat4 ds_view_transform;




varying vec2 packed_varying_0;

vec2 ds_pos=vec2(0.0);

void main() {
    ds_draw_zbias = draw_table[0];

    ds_camera_projection = mat4(pass_table[0], pass_table[1], pass_table[2], pass_table[3], pass_table[4], pass_table[5], pass_table[6], pass_table[7], pass_table[8], pass_table[9], pass_table[10], pass_table[11], pass_table[12], pass_table[13], pass_table[14], pass_table[15]);
    ds_camera_view = mat4(pass_table[16], pass_table[17], pass_table[18], pass_table[19], pass_table[20], pass_table[21], pass_table[22], pass_table[23], pass_table[24], pass_table[25], pass_table[26], pass_table[27], pass_table[28], pass_table[29], pass_table[30], pass_table[31]);
    ds_camera_inv = mat4(pass_table[32], pass_table[33], pass_table[34], pass_table[35], pass_table[36], pass_table[37], pass_table[38], pass_table[39], pass_table[40], pass_table[41], pass_table[42], pass_table[43], pass_table[44], pass_table[45], pass_table[46], pass_table[47]);
    ds_dpi_factor = pass_table[48];
    ds_dpi_dilate = pass_table[49];

    ds_view_transform = mat4(view_table[0], view_table[1], view_table[2], view_table[3], view_table[4], view_table[5], view_table[6], view_table[7], view_table[8], view_table[9], view_table[10], view_table[11], view_table[12], view_table[13], view_table[14], view_table[15]);

    ds_pos.xy = packed_varying_0.xy;

    gl_FragColor = fn_0_6_pixel();
}
"#;

const METAL_OUTPUT: &'static str = r#"
    
"#;

const HLSL_OUTPUT: &'static str = r#"
"#;

use makepad_live_compiler::{
    live_parser::*, LiveFileId, LiveId, LiveModuleId, LiveRegistry, LiveType, LiveTypeInfo, TextPos,
};
use makepad_shader_compiler::generate_glsl;
use makepad_shader_compiler::generate_hlsl;
use makepad_shader_compiler::generate_metal;
use makepad_shader_compiler::DrawShaderConstTable;
use makepad_shader_compiler::DrawShaderDef;
use makepad_shader_compiler::DrawShaderPtr;
use makepad_shader_compiler::ShaderRegistry;
use makepad_shader_compiler::ShaderTy;
// lets just test most features in one go.

fn compare_no_ws(a: &str, b: &str) -> Option<String> {
    let mut b = b.to_string();
    b.retain(|c| !c.is_whitespace());
    let mut a = a.to_string();
    a.retain(|c| !c.is_whitespace());

    let b = b.as_bytes();
    let a = a.as_bytes();

    let mut start = 0;
    let mut changed = false;
    let len = b.len().min(a.len());
    for i in 0..len {
        if a[i] != b[i] {
            changed = true;
            break;
        }
        start = i;
    }
    // now go from the back to i
    let mut end = 0;
    for i in 2..len {
        end = i - 2;
        if a[a.len() - i] != b[b.len() - i] {
            changed = true;
            break;
        }
    }
    // okaay so we have to show the changed thing
    if changed {
        let range_a = if start < (a.len() - end - 1) {
            std::str::from_utf8(&a[start..(a.len() - end - 1)]).unwrap()
        } else {
            ""
        };
        let range_b = if start < (b.len() - end - 1) {
            std::str::from_utf8(&b[start..(b.len() - end - 1)]).unwrap()
        } else {
            ""
        };
        Some(format!(
            "########## OLD ########## {} to {}\n{}\n########## NEW ########## {} to {}\n{}\n########## END ##########",
            start,
            (a.len() - end - 1),
            range_a,
            start,
            (b.len() - end - 1),
            range_b
        ))
    } else {
        None
    }
}

#[test]
fn main() {
    let mut sr = ShaderRegistry::new();
    let mut live_registry = LiveRegistry::default();
    struct FakeType();

    let fake_typeof = LiveTypeInfo {
        module_id: LiveModuleId::from_str(&module_path!()).unwrap(),
        live_type: std::any::TypeId::of::<FakeType>(),
        live_ignore: false,
        fields: Vec::new(),
        type_name: LiveId::from_str("FakeType").unwrap(),
    };
    let module_path = LiveModuleId::from_str("test").unwrap();
    let file_id: LiveFileId = match live_registry.register_live_file(
        "test.live",
        module_path,
        SOURCE.to_string(),
        vec![fake_typeof],
        TextPos { line: 0, column: 0 },
    ) {
        Err(why) => panic!("Couldnt parse file {}", why),
        Ok(x) => x,
    };

    //println!("file_ids: {:?}", live_registry.file_ids);

    let mut errors = Vec::new();
    live_registry.expand_all_documents(&mut errors);

    for msg in errors {
        println!("{:?}\n", msg);
    }

    let shader_ptr = DrawShaderPtr(live_registry.file_id_index_to_live_ptr(file_id, 1));

    let (doc, class_node) = live_registry.ptr_to_doc_node(shader_ptr.0);

    //println!("doc: {:#?}", doc);
    //println!("class_node: {:?}", class_node);

    // // lets just call the shader compiler on this thing

    let analyse_res = sr.analyse_draw_shader(
        &live_registry,
        shader_ptr,
        |live_registry, shader_registry, span, draw_shader_query, live_type, draw_shader_def| {
            // if id == id!(rust_type) {
            //     draw_shader_def.add_uniform(Id::from_str("duni").unwrap(), ShaderTy::Float, span);
            //     draw_shader_def.add_instance(Id::from_str("dinst").unwrap(), ShaderTy::Float, span);
            //     draw_shader_def.add_instance(Id::from_str("dmat").unwrap(), ShaderTy::Mat3, span);
            // }
            // if id == id!(geometry) {}
        },
    );
    println!("{:?}", analyse_res);

    // /*
    // pub fn generate_glsl_shader(&mut self, shader_ptr: DrawShaderNodePtr) -> (String, String) {
    //     // lets find the FullPointer
    //     let draw_shader_decl = self.draw_shaders.get(&shader_ptr).unwrap();
    //     // TODO this env needs its const table transferred
    //     let vertex = generate_glsl::generate_vertex_shader(draw_shader_decl, self);
    //     let pixel = generate_glsl::generate_pixel_shader(draw_shader_decl, self);
    //     return (vertex, pixel)
    // }

    // pub fn generate_metal_shader(&mut self, shader_ptr: DrawShaderNodePtr) -> String{
    //     // lets find the FullPointer
    //     let draw_shader_decl = self.draw_shaders.get(&shader_ptr).unwrap();
    //     let shader = generate_metal::generate_shader(draw_shader_decl, self);
    //     return shader
    // }

    // pub fn generate_hlsl_shader(&mut self, shader_ptr: DrawShaderNodePtr) -> String{
    //     // lets find the FullPointer
    //     let draw_shader_decl = self.draw_shaders.get(&shader_ptr).unwrap();
    //     let shader = generate_hlsl::generate_shader(draw_shader_decl, self);
    //     return shader
    // }*/
    // ok the shader is analysed.
    // now we will generate the glsl shader.
    let draw_shader_def = sr.draw_shader_defs.get(&shader_ptr).unwrap();

    // // TODO this env needs its const table transferred
    let const_table = DrawShaderConstTable::default();
    let vertex = generate_glsl::generate_vertex_shader(draw_shader_def, &const_table, &sr);

    println!("{}", vertex); // LOOK ITS A SHADER!!!!

    let pixel = generate_glsl::generate_pixel_shader(draw_shader_def, &const_table, &sr);
    println!("{}", pixel);

    let compare = format!("\nVERTEXSHADER\n{}PIXELSHADER\n{}", vertex, pixel);

    if let Some(change) = compare_no_ws(GLSL_OUTPUT, &compare) {
        println!("GLSL OUTPUT CHANGED\n{}", change);
        println!(
            "########## ALL ##########\n{}\n########## END ##########",
            compare
        );
        panic!();
    }

    // let shader = generate_metal::generate_shader(draw_shader_def, &const_table, &sr);
    // let compare = format!("\n{}", shader.mtlsl);
    // if let Some(change) = compare_no_ws(METAL_OUTPUT, &compare) {
    //     println!("METAL OUTPUT CHANGED\n{}", change);
    //     println!("########## ALL ##########\n{}\n########## END ##########", compare);
    //     assert_eq!(true, false);
    // }

    // let shader = generate_hlsl::generate_shader(draw_shader_def, &const_table, &sr);
    // let compare = format!("\n{}", shader);
    // if let Some(change) = compare_no_ws(HLSL_OUTPUT, &compare) {
    //     println!("HLSL OUTPUT CHANGED\n{}", change);
    //     println!("########## ALL ##########\n{}\n########## END ##########", compare);
    //     assert_eq!(true, false);
    // }
}
