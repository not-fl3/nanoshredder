#![allow(warnings)]

use makepad_live_compiler::load_file;
use makepad_shader_compiler::{
    generate_glsl, generate_hlsl, generate_metal, DrawShaderConstTable, DrawShaderDef, Shader,
};

#[test]
fn main() {
    let mut file = load_file(SOURCE.to_string()).unwrap();

    // lets just call the shader compiler on this thing
    let sr = Shader::new(&file).unwrap();

    // ok the shader is analysed.
    // now we will generate the glsl/metal/hlsl shader.
    let draw_shader_def = &sr.draw_shader_def;

    // TODO this env needs its const table transferred
    let const_table = DrawShaderConstTable::default();
    let vertex = generate_glsl::generate_vertex_shader(draw_shader_def, &const_table, &sr);
    let pixel = generate_glsl::generate_pixel_shader(draw_shader_def, &const_table, &sr);
    let compare = format!("\nVERTEXSHADER\n{}PIXELSHADER\n{}", vertex, pixel);
    compare_no_ws(GLSL_OUTPUT, &compare);

    let shader = generate_metal::generate_shader(draw_shader_def, &const_table, &sr);
    compare_no_ws(METAL_OUTPUT, &shader.mtlsl);

    let shader = generate_hlsl::generate_shader(draw_shader_def, &const_table, &sr);
    compare_no_ws(HLSL_OUTPUT, &shader);
}

const SOURCE: &'static str = r#"
        varying pos: vec2
        

        fn vertex(self) -> vec4 {
            self.pos = vec2(0, 1);
            return vec4(1, 0, 0, 1);//self.clip_and_transform_vertex()
        }
        
        fn pixel(self) -> vec4 {
            return #f0f
        }
"#;

const GLSL_OUTPUT: &'static str = r#"
VERTEXSHADER




varying vec2 packed_varying_0;

vec2 ds_pos=vec2(0.0);

vec4 fn_2_vertex() {
    (ds_pos = vec2(0.0f, 1.0f));
    return vec4(1.0f, 0.0f, 0.0f, 1.0f);
}

void main() {

    gl_Position = fn_2_vertex();

    packed_varying_0.xy = ds_pos.xy;
}
PIXELSHADER




varying vec2 packed_varying_0;

vec2 ds_pos=vec2(0.0);

vec4 fn_3_pixel() {
    return vec4(1.0, 0.0, 1.0, 1.0);
}

void main() {
    ds_pos.xy = packed_varying_0.xy;

    gl_FragColor = fn_3_pixel();
}
"#;

const METAL_OUTPUT: &'static str = r#"
#include <metal_stdlib>
using namespace metal;
struct LiveUniforms {
};
struct Textures {
};
struct Geometries {
};
struct Instances {
};
struct Varyings {
    float4 position [[position]];
    float2 ds_pos;
};
float4 fn_3_pixel() {
    return float4(1.0, 0.0, 1.0, 1.0);
}
float4 fn_2_vertex() {
    (varyings.ds_pos = float2(0.0f, 1.0f));
    return float4(1.0f, 0.0f, 0.0f, 1.0f);
}
vertex Varyings vertex_main(Textures textures
, const device Geometries *in_geometries [[buffer(0)]]
, const device Instances *in_instances [[buffer(1)]]
, constant LiveUniforms &live_uniforms [[buffer(2)]]
, constant const float *const_table [[buffer(3)]]
, uint vtx_id [[vertex_id]]
, uint inst_id [[instance_id]]
) {
    Geometries geometries = in_geometries[vtx_id];
    Instances instances = in_instances[inst_id];
    Varyings varyings;
    varyings.position = fn_2_vertex();
    return varyings;
}
fragment float4 fragment_main(Varyings varyings[[stage_in]]
, Textures textures
, constant LiveUniforms &live_uniforms [[buffer(2)]]
, constant const float *const_table [[buffer(3)]]
) {
    return     fn_3_pixel();
}    
"#;

const HLSL_OUTPUT: &'static str = r#"
cbuffer LiveUniforms : register(b0) {
};
cbuffer ConstTable : register(b1){float4 const_table[0];};
struct Geometries {
};
struct Instances {
};
struct Varyings {
    float4 position: SV_POSITION;
    float2 ds_pos: VARYA;
};
float2 consfn_vec2_float_float(float x0, , float x1) {    return float2(x0, x1);}float4 consfn_vec4_float_float_float_float(float x0, , float x1, , float x2, , float x3) {    return float4(x0, x1, x2, x3);}float4 fn_3_pixel() {
    return float4(1.0, 0.0, 1.0, 1.0);
}
float4 fn_2_vertex() {
    (varyings.ds_pos = consfn_vec2_float_float(0.0f, 1.0f));
    return consfn_vec4_float_float_float_float(1.0f, 0.0f, 0.0f, 1.0f);
}
Varyings vertex_main(Geometries geometries, Instances instances, uint inst_id: SV_InstanceID) {
    Varyings varyings = {float4(0.0,0.0,0.0,0.0), float2(0.0,0.0)};
    varyings.position = fn_2_vertex();
    return varyings;
}
float4 pixel_main(Varyings varyings) : SV_TARGET{
    return     fn_3_pixel();
}
"#;

fn compare_no_ws(a: &str, b: &str) {
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
        panic!(
            "########## OLD ########## {} to {}\n{}\n########## NEW ########## {} to {}\n{}\n########## END ##########",
            start,
            (a.len() - end - 1),
            range_a,
            start,
            (b.len() - end - 1),
            range_b
        );
    }
}
