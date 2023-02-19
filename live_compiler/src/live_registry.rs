//use crate::id::Id;
use {
    crate::{
        live_component::LiveComponentRegistries,
        live_document::{LiveExpanded, LiveOriginal},
        live_error::{LiveError, LiveErrorSpan, LiveFileError},
        live_expander::LiveExpander,
        live_node::{LiveIdAsProp, LiveNode, LiveNodeOrigin, LiveType, LiveTypeInfo, LiveValue},
        live_node_vec::{LiveNodeMutReader, LiveNodeSlice, LiveNodeVec},
        live_parser::LiveParser,
        live_ptr::{LiveFileGeneration, LiveFileId, LivePtr},
        live_token::{LiveToken, LiveTokenId, TokenWithSpan},
        makepad_error_log::*,
        makepad_live_id::*,
        makepad_live_tokenizer::{
            live_error_origin, Cursor, Delim, FullToken, LiveErrorOrigin, LiveId, State, TokenPos,
            TokenRange, TokenWithLen,
        },
        span::{TextPos, TextSpan},
    },
    std::collections::{BTreeSet, HashMap},
};

#[derive(Default)]
pub struct LiveFile {
    pub(crate) reexpand: bool,

    //pub(crate) module_id: LiveModuleId,
    pub(crate) start_pos: TextPos,
    pub(crate) source: String,
    //pub(crate) deps: BTreeSet<LiveModuleId>,
    pub generation: LiveFileGeneration,

    pub original: LiveOriginal,
    pub next_original: Option<LiveOriginal>,
    pub expanded: LiveExpanded,

    pub live_type_infos: Vec<LiveTypeInfo>,
}

impl LiveFile {
    pub fn ptr_to_doc_node(&self, live_ptr: LivePtr) -> (&LiveExpanded, &LiveNode) {
        (
            &self.expanded,
            &self.expanded.resolve_ptr(live_ptr.index as usize),
        )
    }

    // this looks at the 'id' before the live token id
    pub fn get_node_prefix(&self, origin: LiveNodeOrigin) -> Option<LiveId> {
        if !origin.node_has_prefix() {
            return None;
        }
        let first_def = origin.first_def().unwrap();
        let token_index = first_def.token_index();
        if token_index == 0 {
            return None;
        }
        let doc = &self.original;
        let token = doc.tokens[token_index - 1];
        if let LiveToken::Ident(id) = token.token {
            return Some(id);
        }
        None
    }

    pub fn find_scope_target_via_start(
        &self,
        item: LiveId,
        index: usize,
        nodes: &[LiveNode],
    ) -> Option<LiveScopeTarget> {
        if let Some(index) = nodes.scope_up_down_by_name(index, item.as_field()) {
            match &nodes[index].value {
                // LiveValue::Import(module_id) => {
                //     // if let Some(ret) = self.find_module_id_name(item, *module_id) {
                //     //     return Some(ret);
                //     // }
                //     unimplemented!()
                // }
                LiveValue::Registry(component_type) => {
                    // if let Some(info) = self.components.find_component(*component_type, item) {
                    //     if let Some(ret) = self.find_module_id_name(item, info.module_id) {
                    //         return Some(ret);
                    //     }
                    // };
                    unimplemented!()
                }
                _ => return Some(LiveScopeTarget::LocalPtr(index)),
            }
        }
        // ok now look at the glob use * things
        let mut node_iter = Some(1);
        while let Some(index) = node_iter {
            if nodes[index].id == LiveId::empty() {
                match &nodes[index].value {
                    LiveValue::Registry(component_type) => {
                        // if let Some(info) = self.components.find_component(*component_type, item) {
                        //     if let Some(ret) = self.find_module_id_name(item, info.module_id) {
                        //         return Some(ret);
                        //     }
                        // };
                        unimplemented!()
                    }
                    _ => (),
                }
            }
            node_iter = nodes.next_child(index);
        }
        None
    }

    pub fn find_scope_ptr_via_expand_index(&self, index: usize, item: LiveId) -> Option<LivePtr> {
        // ok lets start
        // let token_id = origin.token_id().unwrap();
        //let index = origin.node_index().unwrap();
        //let file_id = token_id.file_id();
        match self.find_scope_target_via_start(item, index, &self.expanded.nodes) {
            Some(LiveScopeTarget::LocalPtr(index)) => Some(LivePtr {
                index: index as u32,
                generation: self.generation,
            }),
            Some(LiveScopeTarget::LivePtr(ptr)) => Some(ptr),
            None => None,
        }
    }

    pub fn expand_all_documents(&mut self) {
        let mut errors = vec![];
        let mut out_doc = LiveExpanded::new();
        std::mem::swap(&mut out_doc, &mut self.expanded);

        out_doc.nodes.clear();

        let in_doc = &self.original;

        let mut live_document_expander = LiveExpander {
            errors: &mut errors,
        };

        live_document_expander.expand(
            in_doc,
            &mut out_doc,
            self.generation,
        );
        std::mem::swap(&mut out_doc, &mut self.expanded);
    }

    pub fn live_node_as_string(&self, node: &LiveNode) -> Option<String> {
        match &node.value {
            LiveValue::Str(v) => Some(v.to_string()),
            LiveValue::FittedString(v) => Some(v.as_str().to_string()),
            LiveValue::InlineString(v) => Some(v.as_str().to_string()),
            LiveValue::DocumentString {
                string_start,
                string_count,
            } => {
                let origin_doc = self.original();
                let mut out = String::new();
                origin_doc.get_string(*string_start, *string_count, &mut out);
                Some(out)
            }
            LiveValue::Dependency {
                string_start,
                string_count,
            } => {
                let origin_doc = self.original();
                let mut out = String::new();
                origin_doc.get_string(*string_start, *string_count, &mut out);
                Some(out)
            }
            _ => None,
        }
    }

    pub fn original(&self) -> &LiveOriginal {
        &self.original
    }

    pub fn ptr_to_nodes_index(&self, live_ptr: LivePtr) -> (&[LiveNode], usize) {
        if self.generation != live_ptr.generation {
            panic!();
        }
        (&self.expanded.nodes, live_ptr.index as usize)
    }

    pub fn ptr_to_doc(&self) -> &LiveExpanded {
        &self.expanded
    }

    pub fn ptr_to_node(&self, live_ptr: LivePtr) -> &LiveNode {
        &self.expanded.resolve_ptr(live_ptr.index as usize)
    }
}

pub struct LiveDocNodes<'a> {
    pub nodes: &'a [LiveNode],
    pub file_id: LiveFileId,
    pub index: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum LiveScopeTarget {
    LocalPtr(usize),
    LivePtr(LivePtr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiveEditEvent {
    ReparseDocument,
    Mutation {
        tokens: Vec<LiveTokenId>,
        apply: Vec<LiveNode>,
        live_ptrs: Vec<LivePtr>,
    },
}

pub fn tokenize_from_str(
    source: &str,
    start_pos: TextPos,
) -> Result<(Vec<TokenWithSpan>, Vec<char>), LiveError> {
    let mut line_chars = Vec::new();
    let mut state = State::default();
    let mut scratch = String::new();
    let mut strings = Vec::new();
    let mut tokens = Vec::new();
    let mut pos = start_pos;
    for line_str in source.lines() {
        line_chars.clear();
        line_chars.extend(line_str.chars());
        let mut cursor = Cursor::new(&line_chars, &mut scratch);
        loop {
            let (next_state, full_token) = state.next(&mut cursor);
            if let Some(full_token) = full_token {
                let span = TextSpan {
                    start: pos,
                    end: TextPos {
                        column: pos.column + full_token.len as u32,
                        line: pos.line,
                    },
                };
                match full_token.token {
                    FullToken::Unknown | FullToken::OtherNumber | FullToken::Lifetime => {
                        return Err(LiveError {
                            origin: live_error_origin!(),
                            span: span.into(),
                            message: format!("Error tokenizing"),
                        })
                    }
                    FullToken::String => {
                        let len = full_token.len - 2;
                        tokens.push(TokenWithSpan {
                            span: span,
                            token: LiveToken::String {
                                index: strings.len() as u32,
                                len: len as u32,
                            },
                        });
                        let col = pos.column as usize + 1;
                        strings.extend(&line_chars[col..col + len]);
                    }
                    FullToken::Dependency => {
                        let len = full_token.len - 3;
                        tokens.push(TokenWithSpan {
                            span: span,
                            token: LiveToken::Dependency {
                                index: strings.len() as u32,
                                len: len as u32,
                            },
                        });
                        let col = pos.column as usize + 2;
                        strings.extend(&line_chars[col..col + len]);
                    }
                    _ => match LiveToken::from_full_token(full_token.token) {
                        Some(live_token) => {
                            // lets build up the span info
                            tokens.push(TokenWithSpan {
                                span: span,
                                token: live_token,
                            })
                        }
                        _ => (),
                    },
                }
                pos.column += full_token.len as u32;
            } else {
                break;
            }
            state = next_state;
        }
        pos.line += 1;
        pos.column = 0;
    }
    tokens.push(TokenWithSpan {
        span: TextSpan::default(),
        token: LiveToken::Eof,
    });
    Ok((tokens, strings))
}

pub fn load_file(
    //own_module_id: LiveModuleId,
    source: String,
) -> Result<LiveFile, LiveFileError> {
    let start_pos = TextPos { line: 0, column: 0 };
    let (tokens, strings) = match tokenize_from_str(&source, start_pos) {
        Err(msg) => return Err(msg.into_live_file_error()), //panic!("Lex error {}", msg),
        Ok(lex_result) => lex_result,
    };

    let mut parser = LiveParser::new(&tokens, &[]);

    let mut original = match parser.parse_live_document() {
        Err(msg) => return Err(msg.into_live_file_error()), //panic!("Parse error {}", msg.to_live_file_error(file, &source)),
        Ok(ld) => ld,
    };

    original.strings = strings;
    original.tokens = tokens;

    for node in &mut original.nodes {
        match &mut node.value {
            // LiveValue::Import(module_id) => {
            //     if module_id.0 == id!(crate) {
            //         // patch up crate refs
            //         module_id.0 = own_module_id.0
            //     };
            //     deps.insert(*module_id);
            // } // import
            // LiveValue::Registry(component_id) => {
            //     let reg = self.components.0.borrow();
            //     if let Some(entry) = reg
            //         .values()
            //         .find(|entry| entry.component_type() == *component_id)
            //     {
            //         entry.get_module_set(&mut deps);
            //     }
            // }
            LiveValue::Class { .. } => {
                // hold up. this is always own_module_path
                // let infos = self.live_type_infos.get(&live_type).unwrap();
                // for sub_type in infos.fields.clone() {
                //     let sub_module_id = sub_type.live_type_info.module_id;
                //     if sub_module_id != own_module_id {
                //         deps.insert(sub_module_id);
                //     }
                // }
            }
            _ => {}
        }
    }

    let mut live_file = LiveFile {
        reexpand: true,
        //module_id: own_module_id,
        start_pos,
        source,
        generation: LiveFileGeneration::default(),
        live_type_infos: vec![],
        original,
        next_original: None,
        expanded: LiveExpanded::new(),
    };

    live_file.expand_all_documents();
    return Ok(live_file);
}
