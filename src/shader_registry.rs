use {
    crate::{
        analyse::*,
        builtin::{generate_builtins, Builtin},
        makepad_error_log::*,
        makepad_live_compiler::*,
        makepad_live_id::*,
        shader_ast::*,
        shader_parser::{ShaderParser, ShaderParserDep},
    },
    std::{
        cell::{Cell, RefCell},
        collections::{BTreeMap, HashMap, HashSet},
    },
};

pub struct Shader {
    pub all_fns: HashMap<FnPtr, FnDef>,
    pub draw_shader_def: DrawShaderDef,
    pub structs: HashMap<StructPtr, StructDef>,
    pub builtins: HashMap<Ident, Builtin>,
    pub enums: HashMap<LiveType, ShaderEnum>,
}

pub struct ShaderEnum {
    pub enum_name: LiveId,
    pub variants: Vec<LiveId>,
}

#[derive(Debug)]
pub enum LiveNodeFindResult {
    NotFound,
    Component(LivePtr),
    Struct(StructPtr),
    Function(FnPtr),
    PossibleStatic(StructPtr, FnPtr),
    LiveValue(ValuePtr, TyLit),
    Error(LiveError),
}

pub enum DrawShaderQuery {
    DrawShader,
    Geometry,
}

impl Shader {
    pub fn fn_ident_from_ptr(&self, shader_file: &LiveFile, fn_node_ptr: FnPtr) -> Ident {
        let node = shader_file.ptr_to_node(fn_node_ptr.0);
        Ident(node.id)
    }

    pub fn draw_shader_method_ptr_from_ident(
        &self,
        draw_shader_def: &DrawShaderDef,
        ident: Ident,
    ) -> Option<FnPtr> {
        for fn_node_ptr in &draw_shader_def.methods {
            let fn_decl = self.all_fns.get(fn_node_ptr).unwrap();
            if fn_decl.ident == ident {
                return Some(*fn_node_ptr);
            }
        }
        None
    }

    pub fn struct_method_ptr_from_ident(
        &self,
        struct_def: &StructDef,
        ident: Ident,
    ) -> Option<FnPtr> {
        for fn_node_ptr in &struct_def.methods {
            let fn_decl = self.all_fns.get(fn_node_ptr).unwrap();
            if fn_decl.ident == ident {
                return Some(*fn_node_ptr);
            }
        }
        None
    }

    pub fn draw_shader_method_decl_from_ident(
        &self,
        draw_shader_def: &DrawShaderDef,
        ident: Ident,
    ) -> Option<&FnDef> {
        for fn_node_ptr in &draw_shader_def.methods {
            let fn_decl = self.all_fns.get(fn_node_ptr).unwrap();
            if fn_decl.ident == ident {
                return Some(fn_decl);
            }
        }
        None
    }

    pub fn struct_method_decl_from_ident(
        &self,
        struct_def: &StructDef,
        ident: Ident,
    ) -> Option<&FnDef> {
        for fn_node_ptr in &struct_def.methods {
            let fn_decl = self.all_fns.get(fn_node_ptr).unwrap();
            if fn_decl.ident == ident {
                return Some(fn_decl);
            }
        }
        None
    }

    pub fn find_live_node_by_path(
        shader_file: &LiveFile,
        base_ptr: LivePtr,
        ids: &[LiveId],
    ) -> LiveNodeFindResult {
        let doc = &shader_file.ptr_to_doc();

        let ret = walk_recur(
            shader_file,
            None,
            base_ptr.generation,
            base_ptr.index as usize,
            &doc.nodes,
            ids,
        );
        return ret;
        // ok so we got a node. great. now what
        fn walk_recur(
            shader_file: &LiveFile,
            struct_ptr: Option<LivePtr>,
            generation: LiveFileGeneration,
            index: usize,
            nodes: &[LiveNode],
            ids: &[LiveId],
        ) -> LiveNodeFindResult {
            let node = &nodes[index];

            if ids.len() != 0
                && !node.value.is_class()
                && !node.value.is_clone()
                && !node.value.is_object()
            {
                return LiveNodeFindResult::NotFound;
            }

            let now_ptr = LivePtr {
                index: index as u32,
                generation,
            };
            //let first_def = node.origin.first_def().unwrap();
            match node.value {
                LiveValue::Bool(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Bool)
                }
                LiveValue::Int(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Int)
                }
                LiveValue::Float(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Float)
                }
                LiveValue::Color(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Vec4)
                }
                LiveValue::Vec2(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Vec2)
                }
                LiveValue::Vec3(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Vec3)
                }
                LiveValue::Vec4(_)
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    return LiveNodeFindResult::LiveValue(ValuePtr(now_ptr), TyLit::Vec4)
                }
                LiveValue::Expr { .. }
                    if shader_file.get_node_prefix(node.origin) == Some(id!(const)) =>
                {
                    // ok lets eval the expr to get a type
                    match live_eval(shader_file, index, &mut (index + 1), nodes) {
                        Ok(value) => {
                            if let Some(ty) = Ty::from_live_eval(value) {
                                if let Some(ty_lit) = ty.maybe_ty_lit() {
                                    return LiveNodeFindResult::LiveValue(
                                        ValuePtr(now_ptr),
                                        ty_lit,
                                    );
                                }
                            }
                            return LiveNodeFindResult::Error(LiveError {
                                origin: live_error_origin!(),
                                message: format!("Type of eval result not valid for shader"),
                                span: nodes[index].origin.token_id().unwrap().into(),
                            });
                        }
                        Err(err) => {
                            error!("Cannot find node in expression");
                            return LiveNodeFindResult::Error(err);
                        }
                    }
                }
                LiveValue::DSL { token_start, .. } => {
                    // lets get the first token
                    let origin_doc = shader_file.original();
                    match origin_doc.tokens[token_start as usize].token {
                        LiveToken::Ident(id!(fn)) => {
                            if let Some(struct_ptr) = struct_ptr {
                                return LiveNodeFindResult::PossibleStatic(
                                    StructPtr(struct_ptr),
                                    FnPtr(now_ptr),
                                );
                            }
                            return LiveNodeFindResult::Function(FnPtr(now_ptr));
                        }
                        _ => LiveNodeFindResult::NotFound,
                    }
                }
                LiveValue::Class { .. } => {
                    if ids.len() == 0 {
                        return LiveNodeFindResult::Component(now_ptr);
                    }
                    match nodes.child_by_name(index, ids[0].as_field()) {
                        Some(child_index) => {
                            return walk_recur(
                                shader_file,
                                None,
                                generation,
                                child_index,
                                nodes,
                                &ids[1..],
                            )
                        }
                        None => {
                            return LiveNodeFindResult::NotFound;
                        }
                    }
                }
                LiveValue::Clone(clone) => {
                    if ids.len() == 0 {
                        if clone == id!(Struct) {
                            return LiveNodeFindResult::Struct(StructPtr(now_ptr));
                        }
                        return LiveNodeFindResult::Component(now_ptr);
                    }
                    match nodes.child_by_name(index, ids[0].as_field()) {
                        Some(child_index) => {
                            let struct_ptr = if clone == id!(Struct) {
                                Some(now_ptr)
                            } else {
                                None
                            };
                            return walk_recur(
                                shader_file,
                                struct_ptr,
                                generation,
                                child_index,
                                nodes,
                                &ids[1..],
                            );
                        }
                        None => {
                            return LiveNodeFindResult::NotFound;
                        }
                    }
                }
                LiveValue::Object => {
                    if ids.len() == 0 {
                        return LiveNodeFindResult::NotFound;
                    }
                    match nodes.child_by_name(index, ids[0].as_field()) {
                        Some(child_index) => {
                            return walk_recur(
                                shader_file,
                                None,
                                generation,
                                child_index,
                                nodes,
                                &ids[1..],
                            )
                        }
                        None => {
                            return LiveNodeFindResult::NotFound;
                        }
                    }
                }
                _ => {
                    return LiveNodeFindResult::NotFound;
                }
            }
        }
    }

    // lets compile the thing
    pub fn new(shader_file: &LiveFile) -> Result<Shader, LiveError> {
        let mut draw_shader_def = DrawShaderDef::default();

        // lets insert the 2D drawshader uniforms
        // draw_shader_def.add_uniform(
        //     id_from_str!(camera_projection).unwrap(),
        //     id_from_str!(pass).unwrap(),
        //     Ty::Mat4,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(camera_view).unwrap(),
        //     id_from_str!(pass).unwrap(),
        //     Ty::Mat4,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(camera_inv).unwrap(),
        //     id_from_str!(pass).unwrap(),
        //     Ty::Mat4,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(dpi_factor).unwrap(),
        //     id_from_str!(pass).unwrap(),
        //     Ty::Float,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(dpi_dilate).unwrap(),
        //     id_from_str!(pass).unwrap(),
        //     Ty::Float,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(view_transform).unwrap(),
        //     id_from_str!(view).unwrap(),
        //     Ty::Mat4,
        //     TokenSpan::default(),
        // );
        // draw_shader_def.add_uniform(
        //     id_from_str!(draw_zbias).unwrap(),
        //     id_from_str!(draw).unwrap(),
        //     Ty::Float,
        //     TokenSpan::default(),
        // );

        let doc = &shader_file.expanded;

        // ext_self(
        //     self,
        //     class_node.origin.token_id().unwrap().into(),
        //     DrawShaderQuery::DrawShader,
        //     &mut draw_shader_def,
        // );

        let mut parser_deps = Vec::new();
        let mut all_fns = HashMap::new();
        let mut node_iter = doc.nodes.first_child(0);
        let mut method_set = HashSet::new();
        while let Some(node_index) = node_iter {
            let prop = &doc.nodes[node_index];
            let prop_ptr = LivePtr {
                index: node_index as _,
                generation: LiveFileGeneration(0),
            };
            if prop.id == id!(debug_id) {
                node_iter = doc.nodes.next_child(node_index);
                continue;
            }
            match prop.value {
                LiveValue::Bool(_)
                | LiveValue::Id(_)
                | LiveValue::Int(_)
                | LiveValue::Float(_)
                | LiveValue::Color(_)
                | LiveValue::Vec2(_)
                | LiveValue::Vec3(_)
                | LiveValue::Vec4(_)
                | LiveValue::Expr { .. } => {
                    if prop.origin.prop_type() != LivePropType::Field {
                        return Err(LiveError {
                            origin: live_error_origin!(),
                            span: prop.origin.token_id().unwrap().into(),
                            message: format!("Can only support field colon : values don't use ="),
                        });
                    }
                    if prop.id == id!(size) {}
                    let first_def = prop.origin.first_def().unwrap();
                    let before = shader_file.get_node_prefix(prop.origin);

                    let ty = match ShaderTy::from_live_node(shader_file, node_index, &doc.nodes) {
                        Ok(ty) => ty,
                        Err(_) => {
                            // just ignore it
                            node_iter = doc.nodes.next_child(node_index);
                            continue;
                            //return Err(err)
                        }
                    };
                    let ty_expr = ty.to_ty_expr();
                    match before {
                        Some(id!(geometry)) => {
                            draw_shader_def.fields.push(DrawShaderFieldDef {
                                kind: DrawShaderFieldKind::Geometry {
                                    is_used_in_pixel_shader: Cell::new(false),
                                    var_def_ptr: Some(VarDefPtr(prop_ptr)),
                                },
                                span: first_def.into(),
                                ident: Ident(prop.id),
                                ty_expr,
                            });
                        }
                        Some(id!(instance)) => {
                            let decl = DrawShaderFieldDef {
                                kind: DrawShaderFieldKind::Instance {
                                    is_used_in_pixel_shader: Cell::new(false),
                                    live_field_kind: LiveFieldKind::Live,
                                    var_def_ptr: Some(VarDefPtr(prop_ptr)),
                                },
                                span: first_def.into(),
                                ident: Ident(prop.id),
                                ty_expr,
                            };
                            // find from the start the first instancefield
                            // without a var_def_node_prt
                            if let Some(index) = draw_shader_def.fields.iter().position(|field| {
                                if let DrawShaderFieldKind::Instance { var_def_ptr, .. } =
                                    field.kind
                                {
                                    if var_def_ptr.is_none() {
                                        return true;
                                    }
                                }
                                false
                            }) {
                                draw_shader_def.fields.insert(index, decl);
                            } else {
                                draw_shader_def.fields.push(decl);
                            }
                        }
                        Some(id!(uniform)) => {
                            draw_shader_def.fields.push(DrawShaderFieldDef {
                                kind: DrawShaderFieldKind::Uniform {
                                    var_def_ptr: Some(VarDefPtr(prop_ptr)),
                                    block_ident: Ident(id!(user)),
                                },
                                span: first_def.into(),
                                ident: Ident(prop.id),
                                ty_expr,
                            });
                        }
                        Some(id!(varying)) => {
                            draw_shader_def.fields.push(DrawShaderFieldDef {
                                kind: DrawShaderFieldKind::Varying {
                                    var_def_ptr: VarDefPtr(prop_ptr),
                                },
                                span: first_def.into(),
                                ident: Ident(prop.id),
                                ty_expr,
                            });
                        }
                        Some(id!(texture)) => {
                            draw_shader_def.fields.push(DrawShaderFieldDef {
                                kind: DrawShaderFieldKind::Texture {
                                    var_def_ptr: Some(VarDefPtr(prop_ptr)),
                                },
                                span: first_def.into(),
                                ident: Ident(prop.id),
                                ty_expr,
                            });
                        }
                        Some(id!(const)) => {}
                        None => {
                            if let LiveValue::Bool(val) = prop.value {
                                match prop.id {
                                    id!(debug) => {
                                        draw_shader_def.flags.debug = val;
                                    }
                                    id!(draw_call_compare) => {
                                        draw_shader_def.flags.draw_call_nocompare = val;
                                    }
                                    id!(draw_call_always) => {
                                        draw_shader_def.flags.draw_call_always = val;
                                    }
                                    _ => {} // could be input value
                                }
                            }
                        }
                        _ => {
                            return Err(LiveError {
                                origin: live_error_origin!(),
                                span: first_def.into(),
                                message: format!("Unexpected variable prefix {:?}", before),
                            })
                        }
                    };
                }
                LiveValue::Class { .. } => {
                    if prop.id == id!(geometry) {
                        // ext_self(
                        //     self,
                        //     prop.origin.token_id().unwrap().into(),
                        //     DrawShaderQuery::Geometry,
                        //     &mut draw_shader_def,
                        // );
                    }
                }
                LiveValue::DSL {
                    token_start,
                    token_count,
                    expand_index,
                } => {
                    let origin_doc = shader_file.original();

                    let parser = ShaderParser::new(
                        shader_file,
                        origin_doc.get_tokens(token_start as usize, token_count as usize),
                        &mut parser_deps,
                        Some(FnSelfKind::DrawShader),
                        expand_index.unwrap() as usize,
                        prop.origin.token_id().unwrap().file_id().unwrap(),
                        token_start as usize, //None
                    );

                    let token = &origin_doc.tokens[token_start as usize];
                    match token.token {
                        LiveToken::Ident(id!(fn)) => {
                            let fn_def =
                                parser.expect_method_def(FnPtr(prop_ptr), Ident(prop.id))?;
                            if let Some(fn_def) = fn_def {
                                method_set.insert(prop.id);
                                draw_shader_def.methods.push(fn_def.fn_ptr);
                                all_fns.insert(fn_def.fn_ptr, fn_def);
                            }
                        }
                        _ => {
                            return Err(LiveError {
                                origin: live_error_origin!(),
                                span: token.span.into(),
                                message: format!("Unexpected in shader body {}", token),
                            });
                            /*
                            let decl = parser.expect_self_decl(Ident(prop.id), prop_ptr) ?;
                            if let Some(decl) = decl {
                                // lets see where to inject this.
                                // if its an instance var it needs to
                                // go above the var_def_node_ptr one
                                if let DrawShaderFieldKind::Instance {..} = decl.kind {
                                    // find from the start the first instancefield
                                    // without a var_def_node_prt
                                    if let Some(index) = draw_shader_def.fields.iter().position( | field | {
                                        if let DrawShaderFieldKind::Instance {var_def_ptr, ..} = field.kind {
                                            if var_def_ptr.is_none() {
                                                return true
                                            }
                                        }
                                        false
                                    }) {
                                        draw_shader_def.fields.insert(index, decl);
                                    }
                                    else {
                                        draw_shader_def.fields.push(decl);
                                    }
                                }
                                else {
                                    draw_shader_def.fields.push(decl);
                                }
                            }*/
                        }
                    }
                }
                _ => (),
            }
            node_iter = doc.nodes.next_child(node_index);
        }
        // lets check for duplicate fields
        for i in 0..draw_shader_def.fields.len() {
            for j in (i + 1)..draw_shader_def.fields.len() {
                let field_a = &draw_shader_def.fields[i];
                let field_b = &draw_shader_def.fields[j];
                if field_a.ident == field_b.ident && !field_a.ident.0.is_empty() {
                    return Err(LiveError {
                        origin: live_error_origin!(),
                        span: field_a.span.into(),
                        message: format!("Field double declaration  {}", field_b.ident),
                    });
                }
            }
        }

        if !method_set.contains(&id!(vertex)) {
            return Err(LiveError {
                origin: live_error_origin!(),
                span: TokenSpan::default().into(),
                message: format!("analyse_draw_shader missing vertex method"),
            });
        }

        if !method_set.contains(&id!(pixel)) {
            return Err(LiveError {
                origin: live_error_origin!(),
                span: TokenSpan::default().into(),
                message: format!("analyse_draw_shader missing pixel method"),
            });
        }

        assert!(parser_deps.len() == 0);
        //self.analyse_deps(shader_file, &parser_deps) ?;

        let mut shader = Shader {
            structs: HashMap::new(),
            enums: HashMap::new(),
            all_fns,
            draw_shader_def,
            builtins: generate_builtins(),
        };

        let mut sa = DrawShaderAnalyser {
            file: shader_file,
            shader_registry: &mut shader,
            scopes: &mut Scopes::new(),
            options: ShaderAnalyseOptions {
                no_const_collapse: true,
            },
        };
        sa.analyse_shader()?;

        // ok we have all structs
        Ok(shader)
    }
}
