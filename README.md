# NanoShredder

This code is [makepad](https://github.com/makepad/makepad)'s [shader compiler](https://github.com/makepad/makepad/tree/master/platform/shader_compiler), but _modified_.

During the modification shader-compiler became *worse*, not better. Most of live editing capabilities got lost, error reporting became worse, new bugs were introduced and maintanability/possibility to merge from upstream was lost.

Like the code was runned through a paper hredder and purely assembled back together, thus the project name. 

![shadertoy](https://user-images.githubusercontent.com/910977/220827364-a39e005f-bbf3-4658-875a-7b2ec65f7ad0.gif)


*macroquad's [shadertoy](https://github.com/not-fl3/macroquad/blob/master/examples/shadertoy.rs) example*

`NanoShredder` can take a rust-like dsl and produce `glsl`, `metal` and `hlsl` shaders. Usage example: [basic.rs](/examples/basic.rs). It may(or may not) be evantually used as an optional cli/runtime tool to help miniquad based projects with shaders cross compilation. 



