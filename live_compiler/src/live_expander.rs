use crate::{
    live_document::{LiveExpanded, LiveOriginal},
    live_error::LiveError,
    live_node::{LiveIdAsProp, LiveNode, LiveValue},
    live_node_vec::{LiveNodeSlice, LiveNodeVec},
    live_ptr::{LiveFileId, LivePtr},
    makepad_live_id::*,
    makepad_live_tokenizer::{live_error_origin, LiveErrorOrigin},
    LiveTypeInfo,
};

pub struct LiveExpander<'a> {
    pub errors: &'a mut Vec<LiveError>,
}

impl<'a> LiveExpander<'a> {
    pub fn is_baseclass(id: LiveId) -> bool {
        id == id!(Struct)
    }

    pub fn shift_parent_stack(
        &self,
        parents: &mut Vec<(LiveId, usize)>,
        nodes: &[LiveNode],
        after_point: usize,
        old_size: usize,
        new_size: usize,
    ) {
        for (live_id, item) in parents {
            if *item > after_point {
                if old_size > new_size {
                    *item -= old_size - new_size;
                } else if old_size < new_size {
                    *item += new_size - old_size;
                }
                if nodes[*item].id != *live_id {
                    panic!()
                }
                if !nodes[*item].is_open() {
                    panic!()
                }
            }
        }
    }

    pub fn expand(
        &mut self,
        in_doc: &LiveOriginal,
        out_doc: &mut LiveExpanded,
    ) {
        out_doc.nodes.push(in_doc.nodes[0].clone());
        let mut current_parent = vec![(LiveId(0), 0usize)];
        let mut in_index = 1;

        loop {
            if in_index >= in_doc.nodes.len() - 1 {
                break;
            }

            let in_node = &in_doc.nodes[in_index];
            let in_value = &in_node.value;

            match in_value {
                LiveValue::Close => {
                    current_parent.pop();
                    in_index += 1;
                    continue;
                }
                // LiveValue::Registry(component_type) => {
                //     let registries = self.live_registry.components.0.borrow();
                //     if let Some(registry) = registries
                //         .values()
                //         .find(|v| v.component_type() == *component_type)
                //     {
                //         if in_node.id != LiveId(0)
                //             && registry.get_component_info(in_node.id).is_none()
                //         {
                //             self.errors.push(LiveError {
                //                 origin: live_error_origin!(),
                //                 span: in_node.origin.token_id().unwrap().into(),
                //                 message: format!(
                //                     "Use statement component not found {}::{}",
                //                     component_type, in_node.id
                //                 ),
                //             });
                //         }
                //     } else {
                //         self.errors.push(LiveError {
                //             origin: live_error_origin!(),
                //             span: in_node.origin.token_id().unwrap().into(),
                //             message: format!(
                //                 "Use statement invalid component type {}::{}",
                //                 component_type, in_node.id
                //             ),
                //         });
                //     }
                //     let index = out_doc
                //         .nodes
                //         .append_child_index(current_parent.last().unwrap().1);
                //     let old_len = out_doc.nodes.len();
                //     out_doc.nodes.insert(index, in_node.clone());
                //     self.shift_parent_stack(
                //         &mut current_parent,
                //         &out_doc.nodes,
                //         index,
                //         old_len,
                //         out_doc.nodes.len(),
                //     );
                //     in_index += 1;
                //     continue;
                // }
                // LiveValue::Import(module_id) => {
                //     // lets verify it points anywhere
                //     if self
                //         .live_registry
                //         .module_id_and_name_to_doc(*module_id, in_node.id)
                //         .is_none()
                //     {
                //         self.errors.push(LiveError {
                //             origin: live_error_origin!(),
                //             span: in_node.origin.token_id().unwrap().into(),
                //             message: format!(
                //                 "Use statement invalid target {}::{}",
                //                 module_id, in_node.id
                //             ),
                //         });
                //     }
                //     let index = out_doc
                //         .nodes
                //         .append_child_index(current_parent.last().unwrap().1);
                //     let old_len = out_doc.nodes.len();
                //     out_doc.nodes.insert(index, in_node.clone());
                //     self.shift_parent_stack(
                //         &mut current_parent,
                //         &out_doc.nodes,
                //         index,
                //         old_len,
                //         out_doc.nodes.len(),
                //     );
                //     in_index += 1;
                //     continue;
                // }
                x => {}
            }

            //// determine node overwrite rules

            let out_index = match out_doc
                .nodes
                .child_or_append_index_by_name(current_parent.last().unwrap().1, in_node.prop())
            {
                Ok(overwrite) => {
                    let out_value = &out_doc.nodes[overwrite].value;
                    let out_origin = out_doc.nodes[overwrite].origin;

                    if in_node.origin.edit_info().is_some() {
                        self.errors.push(LiveError {
                            origin: live_error_origin!(),
                            span: in_doc
                                .token_id_to_span(in_node.origin.token_id().unwrap())
                                .into(),
                            message: format!(
                                "Cannot define edit info after first prop def of {}",
                                in_node.id
                            ),
                        });
                    }
                    // object override
                    if in_value.is_object()
                        && (out_value.is_clone() || out_value.is_class() || out_value.is_object())
                    { // lets set the target ptr
                         // do nothing
                    }
                    // replacing object types
                    else if out_value.is_expr() || in_value.is_expr() && out_value.is_value_type()
                    {
                        // replace range
                        let next_index = out_doc.nodes.skip_node(overwrite);

                        // POTENTIAL SHIFT
                        let old_len = out_doc.nodes.len();
                        out_doc.nodes.splice(
                            overwrite..next_index,
                            in_doc.nodes.node_slice(in_index).iter().cloned(),
                        );
                        self.shift_parent_stack(
                            &mut current_parent,
                            &out_doc.nodes,
                            overwrite,
                            old_len,
                            out_doc.nodes.len(),
                        );

                        in_index = in_doc.nodes.skip_node(in_index);
                        out_doc.nodes[overwrite].origin.inherit_origin(out_origin);
                        continue;
                    } else if out_value.is_open() && in_value.is_open() {
                        // just replace the whole thing
                        let next_index = out_doc.nodes.skip_node(overwrite);
                        let old_len = out_doc.nodes.len();
                        out_doc.nodes.drain(overwrite + 1..next_index - 1);
                        self.shift_parent_stack(
                            &mut current_parent,
                            &out_doc.nodes,
                            overwrite,
                            old_len,
                            out_doc.nodes.len(),
                        );
                        out_doc.nodes[overwrite] = in_node.clone();
                    }
                    // replace object with single value
                    else if out_value.is_open() {
                        let next_index = out_doc.nodes.skip_node(overwrite);
                        let old_len = out_doc.nodes.len();
                        out_doc.nodes.drain(overwrite + 1..next_index);
                        self.shift_parent_stack(
                            &mut current_parent,
                            &out_doc.nodes,
                            overwrite,
                            old_len,
                            out_doc.nodes.len(),
                        );
                        out_doc.nodes[overwrite] = in_node.clone();
                    }
                    // replace single value with object
                    else if in_value.is_open() {
                        let old_len = out_doc.nodes.len();
                        out_doc.nodes[overwrite] = in_node.clone();
                        out_doc.nodes.insert(overwrite + 1, in_node.clone());
                        out_doc.nodes[overwrite + 1].value = LiveValue::Close;
                        self.shift_parent_stack(
                            &mut current_parent,
                            &out_doc.nodes,
                            overwrite,
                            old_len,
                            out_doc.nodes.len(),
                        );
                    } else {
                        out_doc.nodes[overwrite] = in_node.clone();
                    };
                    out_doc.nodes[overwrite].origin.inherit_origin(out_origin);
                    overwrite
                }
                Err(insert_point) => {
                    // ok so. if we are inserting an expression, just do the whole thing in one go.
                    if in_node.is_expr() {
                        // splice it in
                        let old_len = out_doc.nodes.len();
                        out_doc.nodes.splice(
                            insert_point..insert_point,
                            in_doc.nodes.node_slice(in_index).iter().cloned(),
                        );
                        self.shift_parent_stack(
                            &mut current_parent,
                            &out_doc.nodes,
                            insert_point - 1,
                            old_len,
                            out_doc.nodes.len(),
                        );

                        in_index = in_doc.nodes.skip_node(in_index);
                        continue;
                    }

                    let old_len = out_doc.nodes.len();
                    out_doc.nodes.insert(insert_point, in_node.clone());
                    if in_node.is_open() {
                        out_doc.nodes.insert(insert_point + 1, in_node.clone());
                        out_doc.nodes[insert_point + 1].value = LiveValue::Close;
                    }
                    self.shift_parent_stack(
                        &mut current_parent,
                        &out_doc.nodes,
                        insert_point - 1,
                        old_len,
                        out_doc.nodes.len(),
                    );

                    insert_point
                }
            };

            // process stacks
            match in_value {
                LiveValue::Clone(clone) => {
                    // if let Some(target) = self.live_registry.find_scope_target_via_start(
                    //     *clone,
                    //     out_index,
                    //     &out_doc.nodes,
                    // ) {
                    //     match target {
                    //         LiveScopeTarget::LocalPtr(local_ptr) => {
                    //             let old_len = out_doc.nodes.len();

                    //             out_doc
                    //                 .nodes
                    //                 .insert_children_from_self(local_ptr, out_index + 1);
                    //             self.shift_parent_stack(
                    //                 &mut current_parent,
                    //                 &out_doc.nodes,
                    //                 out_index,
                    //                 old_len,
                    //                 out_doc.nodes.len(),
                    //             );

                    //             // clone the value and store a parent pointer
                    //             if let LiveValue::Class {
                    //                 live_type: old_live_type,
                    //                 ..
                    //             } = &out_doc.nodes[out_index].value
                    //             {
                    //                 if let LiveValue::Class { live_type, .. } =
                    //                     &out_doc.nodes[local_ptr].value
                    //                 {
                    //                     if live_type != old_live_type {
                    //                         self.errors.push(LiveError {
                    //                             origin: live_error_origin!(),
                    //                             span: in_doc
                    //                                 .token_id_to_span(
                    //                                     in_node.origin.token_id().unwrap(),
                    //                                 )
                    //                                 .into(),
                    //                             message: format!(
                    //                                 "Class override with wrong type {}",
                    //                                 in_node.id
                    //                             ),
                    //                         });
                    //                     }
                    //                 }
                    //             }

                    //             out_doc.nodes[out_index].value =
                    //                 out_doc.nodes[local_ptr].value.clone();
                    //             if let LiveValue::Class { class_parent, .. } =
                    //                 &mut out_doc.nodes[out_index].value
                    //             {
                    //                 *class_parent = Some(LivePtr {
                    //                     index: out_index as u32,
                    //                     generation,
                    //                 });
                    //             }
                    //         }
                    //         LiveScopeTarget::LivePtr(live_ptr) => {
                    //             let doc = &self.live_registry.live_files[0].expanded;

                    //             let old_len = out_doc.nodes.len();
                    //             out_doc.nodes.insert_children_from_other(
                    //                 live_ptr.node_index(),
                    //                 out_index + 1,
                    //                 &doc.nodes,
                    //             );
                    //             self.shift_parent_stack(
                    //                 &mut current_parent,
                    //                 &out_doc.nodes,
                    //                 out_index,
                    //                 old_len,
                    //                 out_doc.nodes.len(),
                    //             );

                    //             out_doc.nodes[out_index].value =
                    //                 doc.nodes[live_ptr.node_index()].value.clone();
                    //             if let LiveValue::Class { class_parent, .. } =
                    //                 &mut out_doc.nodes[out_index].value
                    //             {
                    //                 *class_parent = Some(LivePtr {
                    //                     index: out_index as u32,
                    //                     generation,
                    //                 });
                    //             }
                    //         }
                    //     };
                    //     //overwrite value, this copies the Class
                    // } else if !Self::is_baseclass(*clone) {
                    //     //if !self.live_registry.ignore_no_dsl.contains(clone) {
                    //     self.errors.push(LiveError {
                    //                 origin: live_error_origin!(),
                    //                 span: in_doc.token_id_to_span(in_node.origin.token_id().unwrap()).into(),
                    //                 message: format!("Can't find live definition of {} did you forget to call live_register for it?", clone)
                    //             });
                    // }
                    // current_parent.push((out_doc.nodes[out_index].id, out_index));
                    unimplemented!()
                }
                LiveValue::Class { .. } => {
                    // store the class context
                    if let LiveValue::Class { class_parent, .. } =
                        &mut out_doc.nodes[out_index].value
                    {
                        *class_parent = Some(LivePtr {
                            index: out_index as u32,
                        });
                    }

                    let mut insert_point = out_index + 1;

                    current_parent.push((out_doc.nodes[out_index].id, out_index));
                }
                LiveValue::Expr { .. } => {
                    panic!()
                }
                LiveValue::Array
                | LiveValue::TupleEnum { .. }
                | LiveValue::NamedEnum { .. }
                | LiveValue::Object => {
                    // lets check what we are overwriting
                    current_parent.push((out_doc.nodes[out_index].id, out_index));
                }
                LiveValue::DSL { .. } => {}
                x => {}
            }

            in_index += 1;
        }
        out_doc.nodes.push(in_doc.nodes.last().unwrap().clone());

        // this stores the node index on nodes that don't have a node index
        for i in 1..out_doc.nodes.len() {
            if out_doc.nodes[i].value.is_dsl() {
                out_doc.nodes[i].value.set_dsl_expand_index_if_none(i);
            }
            if out_doc.nodes[i].value.is_expr() {
                out_doc.nodes[i].value.set_expr_expand_index_if_none(i);
            }
        }
    }
}
