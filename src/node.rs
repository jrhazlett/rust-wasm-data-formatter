//
// Libraries - native
//
use rustc_hash::FxHashMap;
//
// Libraries - js
//
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::JsValue;
//
// Public
//
#[derive(Debug)]
pub enum EnumJsTypes {
    TypeArray,
    TypeObject,
    TypeNotIterableOrIsString,
}

pub enum EnumStorageTypes {
    TypeHashMapWithKeyInts(FxHashMap<u32, JsValue>),
    TypeHashMapWithKeyStrings(FxHashMap<String, JsValue>),
    TypeValue(JsValue),
}

pub enum EnumKeyTypes {
    TypeInt(u32),
    TypeString(String),
}

pub struct Node {
    pub field_bool_children_in_stack: bool,
    pub field_parent_key: Option<EnumKeyTypes>,
    pub field_type: Option<EnumJsTypes>,
    pub field_usize_layer: usize,
    pub field_value: Option<EnumStorageTypes>,
}

impl Node {
    pub fn get_enum_type_iterable_or_not_iterable(arg_value: &JsValue) -> EnumJsTypes {
        if arg_value.is_array() {
            EnumJsTypes::TypeArray
        } else if arg_value.is_object() {
            EnumJsTypes::TypeObject
        } else {
            EnumJsTypes::TypeNotIterableOrIsString
        }
    }

    fn get_string_printable_from_array(arg_slice: Vec<String>) -> String {
        return format!("[ {} ]", arg_slice.join(", "));
    }

    pub fn get_value(&self) -> Result<JsValue, JsValue> {
        if let Some(type_from_field) = &self.field_type {
            match type_from_field {
                EnumJsTypes::TypeArray => {
                    match &self.field_value {
                        Some(EnumStorageTypes::TypeHashMapWithKeyInts(hash_map_stored)) => {
                            let array = Array::new();
                            for (item_key_int, item_value) in hash_map_stored.iter() {
                                array.set(*item_key_int, item_value.clone())
                            }
                            Ok(array.into())
                        }
                        _ => return Err(JsValue::from_str(
                            "Error: Trying to perform non-Array op on Array in Node.get_value().",
                        )),
                    }
                }
                EnumJsTypes::TypeObject => {
                    match &self.field_value {
                        Some(EnumStorageTypes::TypeHashMapWithKeyStrings(hash_map_stored)) => {
                            let object_to_return = Object::new();
                            for (item_key, item_value) in hash_map_stored.iter() {
                                if let Ok(_bool_returned) = Reflect::set(
                                    &object_to_return,
                                    &JsValue::from(item_key),
                                    &item_value,
                                ) {
                                } else {
                                    return Err(JsValue::from_str(
                                        "Error: Failed to set value at key in Node.get_value().",
                                    ));
                                }
                            }
                            Ok(object_to_return.into())
                        }
                        _ => return Err(JsValue::from_str(
                            "Error: Trying to perform non-Object op on Object in Node.get_value().",
                        )),
                    }
                }
                _ => {
                    match &self.field_value {
                        Some(EnumStorageTypes::TypeValue(value_stored)) => Ok(value_stored.clone()),
                        _ => return Err(JsValue::from_str(
                            "Error: Trying to perform non-Value op on Value in Node.get_value()",
                        )),
                    }
                }
            }
        } else {
            return Err(JsValue::from_str(
                "Error: Failed to identify value's type is None while executing Node.get_value().",
            ));
        }
    }

    pub fn get_vec_of_node_children(&self) -> Result<Vec<Node>, JsValue> {
        match &self.field_value {
            Some(EnumStorageTypes::TypeHashMapWithKeyInts(hash_map)) => {
                let mut vec_to_return = Vec::new();
                for (item_key, item_value) in hash_map {
                    let mut item_node_child = Node::new();
                    item_node_child.setup(
                        item_value.clone(),
                        self.field_usize_layer + 1,
                        Some(EnumKeyTypes::TypeInt(*item_key)),
                    )?;
                    vec_to_return.push(item_node_child);
                }
                return Ok(vec_to_return);
            }
            Some(EnumStorageTypes::TypeHashMapWithKeyStrings(hash_map)) => {
                let mut vec_to_return = Vec::new();
                for (item_key, item_value) in hash_map {
                    let mut item_node_child = Node::new();
                    item_node_child.setup(
                        item_value.clone(),
                        self.field_usize_layer + 1,
                        Some(EnumKeyTypes::TypeString(item_key.clone())),
                    )?;
                    vec_to_return.push(item_node_child);
                }
                return Ok(vec_to_return);
            }
            Some(EnumStorageTypes::TypeValue(_value)) => Ok(Vec::new()),
            None => Ok(Vec::new()),
        }
    }

    pub fn logic_has_children(&self) -> bool {
        match &self.field_value {
            Some(EnumStorageTypes::TypeHashMapWithKeyInts(_hash_map)) => true,
            Some(EnumStorageTypes::TypeHashMapWithKeyStrings(_hash_map)) => true,
            Some(EnumStorageTypes::TypeValue(_value)) => false,
            None => false,
        }
    }

    pub fn set_value_at_key(
        &mut self,
        arg_key: &EnumKeyTypes,
        arg_value: &JsValue,
    ) -> Result<JsValue, JsValue> {
        match &mut self.field_value {
            Some(EnumStorageTypes::TypeHashMapWithKeyInts(ref mut hash_map)) => {
                let key = match arg_key {
                    EnumKeyTypes::TypeInt(result) => result,
                    _ => {
                        return Err(JsValue::from_str(
                            "Error: Attempted non-int key op on int key in set_value_at_key().",
                        ))
                    }
                };
                if !hash_map.contains_key(key) {
                    return Err(JsValue::from_str(
                        [
                            "Error: Attempting to set value for non-existent key.".to_string(),
                            format!("key = {}", key),
                            format!(
                                "array_valid_keys = {}",
                                Node::get_string_printable_from_array(
                                    hash_map
                                        .keys()
                                        .map(|item| { format!("{}", item) })
                                        .collect::<Vec<String>>()
                                )
                            ),
                        ]
                        .join("\n")
                        .as_str(),
                    ));
                }
                hash_map.insert(*key, arg_value.clone());
            }
            Some(EnumStorageTypes::TypeHashMapWithKeyStrings(ref mut hash_map)) => {
                let key = match arg_key {
                    EnumKeyTypes::TypeString(result) => result,
                    _ => return Err(JsValue::from_str(
                        "Error: Attempting non-string op on string key in Node.set_value_at_key().",
                    )),
                };
                if !hash_map.contains_key(key) {
                    return Err(JsValue::from_str(
                        [
                            "Error: Attempting to set value for non-existent key.".to_string(),
                            format!("key = {}", key),
                            format!(
                                "array_valid_keys = {}",
                                Node::get_string_printable_from_array(
                                    hash_map
                                        .keys()
                                        .map(|item| { item.clone() })
                                        .collect::<Vec<String>>()
                                )
                            ),
                        ]
                        .join("\n")
                        .as_str(),
                    ));
                }
                hash_map.insert(key.clone(), arg_value.clone());
            }
            Some(EnumStorageTypes::TypeValue(_value)) => {
                return Err(JsValue::from_str(
                    "Error: Attempting to set key while Node contains raw JsValue.",
                ));
            }
            None => {
                return Err(JsValue::from_str(
                    "Error: Attempt to set key when Node's value is None.",
                ))
            }
        }
        Ok(JsValue::from_str(""))
    }

    pub fn setup(
        &mut self,
        arg_value: JsValue,
        arg_usize_layer: usize,
        arg_parent_key: Option<EnumKeyTypes>,
    ) -> Result<JsValue, JsValue> {
        self.field_parent_key = arg_parent_key;
        self.field_usize_layer = arg_usize_layer;
        match Node::get_enum_type_iterable_or_not_iterable(&arg_value) {
            EnumJsTypes::TypeArray => {
                self.field_type = Some(EnumJsTypes::TypeArray);
                self.field_value = Some(EnumStorageTypes::TypeHashMapWithKeyInts({
                    let mut hash_map = FxHashMap::default();
                    for item_entry_result in Array::from(&arg_value).entries() {
                        if let Ok(item_entry) = item_entry_result {
                            let item_entry_array = js_sys::Array::from(&item_entry);
                            if let Some(item_key_f64) = item_entry_array.get(0).as_f64() {
                                hash_map.insert(item_key_f64 as u32, item_entry_array.get(1));
                            } else {
                                return Err(JsValue::from_str(
                                    "Error: Failed to convert JsValue key to u32 in Node.setup().",
                                ));
                            }
                        } else {
                            return Err( JsValue::from_str("Error: Failed to unpack item_entry_result during iteration in Node.setup()." ) );
                        }
                    }
                    hash_map
                }))
            }
            EnumJsTypes::TypeObject => {
                self.field_type = Some(EnumJsTypes::TypeObject);
                if let Some(object) = Object::try_from(&arg_value) {
                    self.field_value = Some(EnumStorageTypes::TypeHashMapWithKeyStrings({
                        let mut hash_map_to_return = FxHashMap::default();
                        for item_pair_result in Object::entries(&object).values() {
                            let item_pair_array = Array::from(&item_pair_result?);
                            if let Some(item_string_key) = item_pair_array.get(0).as_string() {
                                hash_map_to_return.insert(item_string_key, item_pair_array.get(1));
                            } else {
                                return Err( JsValue::from_str( "Error: Failed to convert JsValue key to String in Node.setup()." ) );
                            }
                        }
                        hash_map_to_return
                    }))
                } else {
                    return Err(JsValue::from_str(
                        "Error: Failed to convert JsValue to Object in Node.setup().",
                    ));
                }
            }
            _ => {
                self.field_bool_children_in_stack = true;
                self.field_type = Some(EnumJsTypes::TypeNotIterableOrIsString);
                self.field_value = Some(EnumStorageTypes::TypeValue(arg_value.clone()));
            }
        }
        Ok(JsValue::from_str(""))
    }

    pub fn new() -> Self {
        Self {
            field_bool_children_in_stack: false,
            field_parent_key: None,
            field_type: None,
            field_usize_layer: 0,
            field_value: None,
        }
    }
}
