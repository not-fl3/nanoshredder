#![allow(warnings)]

mod shader;
mod shader_ast;
mod shader_parser;
//mod env;
mod analyse;
mod builtin;
mod const_eval;
mod const_gather;
mod dep_analyse;
mod generate;
mod lhs_check;
mod swizzle;
mod ty_check;
mod util;

//#[cfg(any(target_os = "linux", target_arch = "wasm32", test))]
pub(crate) mod generate_glsl;
//#[cfg(any(target_os = "macos", test))]
pub(crate) mod generate_metal;
//#[cfg(any(target_os = "windows", test))]
pub(crate) mod generate_hlsl;

pub(crate) use crate::{
    shader::{DrawShaderQuery, ShaderEnum},
    shader_ast::{DrawShaderFieldKind, DrawShaderFlags, DrawShaderPtr, ValuePtr},
};
pub(crate) use makepad_live_compiler::{
    self, makepad_live_tokenizer, makepad_math,
};
pub(crate) use makepad_live_tokenizer::makepad_live_id;

pub(crate) use crate::shader_ast::{DrawShaderConstTable, DrawShaderDef};

pub use crate::shader_ast::ShaderTy;
pub use shader::Shader;
