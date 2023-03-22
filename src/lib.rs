use js_sys::Function;
use wasm_bindgen::prelude::*;

pub mod node;
use node::EnumKeyTypes;
use node::Node;
//
// Exports
//
#[wasm_bindgen]
pub fn get_tree_modified(arg_input: &JsValue, arg_callback: &Function) -> Result<JsValue, JsValue> {
    let mut node_root = Node::new();
    node_root.setup(arg_input.clone(), 0, None)?;
    let mut stack = vec![node_root];
    loop {
        if let Some(mut item_node) = stack.pop() {
            if !item_node.field_bool_children_in_stack {
                item_node.field_bool_children_in_stack = true;
                let vec_of_children = item_node.get_vec_of_node_children()?;
                stack.push(item_node);
                stack.extend(vec_of_children);
            } else {
                if let Some(parent_key_type) = &item_node.field_parent_key {
                    let key = match parent_key_type {
                        EnumKeyTypes::TypeInt(result) => EnumKeyTypes::TypeInt(*result),
                        EnumKeyTypes::TypeString(result) => {
                            EnumKeyTypes::TypeString(result.clone())
                        }
                    };
                    let mut bool_node_to_update_never_found = true;
                    let usize_layer_for_node_to_update = item_node.field_usize_layer - 1;
                    for item_index in (0..stack.len()).rev() {
                        if let Some(item_node_to_update) = stack.get_mut(item_index) {
                            if item_node_to_update.field_usize_layer
                                == usize_layer_for_node_to_update
                                && item_node_to_update.field_bool_children_in_stack
                            {
                                bool_node_to_update_never_found = false;
                                item_node_to_update.set_value_at_key(&key, &{
                                    if item_node.logic_has_children() {
                                        item_node.get_value()?
                                    } else {
                                        get_value_returned_from_callback(
                                            &item_node.get_value()?,
                                            arg_callback,
                                        )?
                                    }
                                })?;
                            }
                        }
                    }
                    if bool_node_to_update_never_found {
                        return Err(JsValue::from_str("Error: item_node_to_update never found."));
                    }
                } else {
                    return item_node.get_value();
                }
            }
        } else {
            return Err(JsValue::from_str(
                "Error: Stack is empty and function still hasn't exited.",
            ));
        }
    }
}

pub fn get_value_returned_from_callback(
    arg_value: &JsValue,
    arg_callback: &Function,
) -> Result<JsValue, JsValue> {
    if let Ok(value_returned) = arg_callback.call1(&arg_value, &arg_value) {
        Ok(value_returned)
    } else {
        Err(JsValue::from_str(
            "Error: Failed to get value returned from callback.",
        ))
    }
}
