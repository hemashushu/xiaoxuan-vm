// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anvm_ast::{
    ast::FunctionType,
    types::{Value, ValueType},
};

use crate::error::NativeError;

pub type NativeFunction = fn(&[Value]) -> Result<Vec<Value>, NativeError>;

pub struct NativeFunctionItem {
    pub name: String,
    pub type_index: usize,
    pub param_names: Vec<String>,
    pub native_function: NativeFunction,
}

pub struct NativeModule {
    pub name: String,
    pub function_types: Vec<FunctionType>,
    pub function_items: Vec<NativeFunctionItem>,
}

impl NativeModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            function_types: vec![],
            function_items: vec![],
        }
    }

    /// 添加 FunctionType 到内部列表，
    /// 如果相同的 FunctionType 已经存在，则返回已存在项目的索引值。
    fn add_function_type(&mut self, params: Vec<ValueType>, results: Vec<ValueType>) -> usize {
        let function_type = FunctionType { params, results };

        let option_function_type_index = self
            .function_types
            .iter()
            .enumerate()
            .find(|item| item.1 == &function_type)
            .map(|item| item.0);

        if let Some(function_type_index) = option_function_type_index {
            function_type_index
        } else {
            let count = self.function_types.len();
            self.function_types.push(function_type);
            count
        }
    }

    pub fn add_function(
        &mut self,
        name: &str,
        params: Vec<ValueType>,
        param_names: Vec<String>,
        results: Vec<ValueType>,
        native_function: NativeFunction,
    ) {
        let type_index = self.add_function_type(params, results);
        let function_item = NativeFunctionItem {
            name: name.to_string(),
            type_index,
            param_names,
            native_function,
        };

        self.function_items.push(function_item);
    }

    pub fn find_function_index_by_name(&self, name: &str) -> Option<usize> {
        self.function_items
            .iter()
            .enumerate()
            .find(|item| item.1.name == name)
            .map(|item| item.0)
    }
}
