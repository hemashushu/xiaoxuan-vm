// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # 控制指令
//!
//! ## end 指令

use anvm_ast::{
    instruction::BlockType,
    types::{check_value_types, ValueType, ValueTypeCheckError},
};

use crate::{
    error::{EngineError, InvalidOperation, TypeMismatch},
    vm::{INITIAL_FRAME_POINTER, VM},
    vm_stack::INFO_SEGMENT_ITEM_COUNT,
};

pub enum ControlResult {
    /// 执行下一句
    Sequence,

    /// 进入一个函数或者一个结构块
    ///
    /// 参数用于更新虚拟机的 pc 值
    PushStackFrame {
        is_call_frame: bool,
        vm_module_index: usize,
        function_index: usize,
        frame_type: BlockType,
        address: usize,
    },

    /// 从一个函数或者一个结构块跳出
    ///
    /// 参数用于更新虚拟机的 pc 值
    PopStackFrame {
        is_call_frame: bool,
        vm_module_index: usize,
        function_index: usize,
        frame_type: BlockType,
        address: usize,
    },

    /// 函数内跳转
    ///
    /// 即结构块间的跳转
    JumpWithinFunction {
        frame_type: BlockType,
        address: usize,
    },

    /// 结构块内跳转
    JumpWithinBlock(/* address */ usize),

    /// 程序已结束
    ProgramEnd,
}

pub fn process_end(
    vm: &mut VM,
    option_block_index: &Option<usize>,
) -> Result<ControlResult, EngineError> {
    let frame_type = &vm.status.frame_type;
    let vm_module_index = vm.status.vm_module_index;
    let function_index = vm.status.function_index;

    // 如果 fp 和 lp 的值相同，则说明当前是调用帧，否则则是流程控制的结构块帧
    let frame_pointer = vm.status.frame_pointer;
    let local_pointer = vm.status.local_pointer;
    let is_call_frame = frame_pointer == local_pointer;

    // 获取当前帧的返回值类型
    let result_types = {
        match frame_type {
            BlockType::ResultEmpty => vec![],
            BlockType::ResultI32 => vec![ValueType::I32],
            BlockType::ResultI64 => vec![ValueType::I64],
            BlockType::ResultF32 => vec![ValueType::F32],
            BlockType::ResultF64 => vec![ValueType::F64],
            BlockType::TypeIndex(type_index) => {
                let vm_module = &vm.resource.vm_modules[vm_module_index];
                let function_type = &vm_module.function_types[*type_index as usize];
                function_type.results.clone()
            }
        }
    };

    // 判断操作数是否足够当前函数或结构块用于返回
    let results_count = result_types.len();
    let stack_size = vm.stack.get_size();
    let operands_count = stack_size - vm.status.base_pointer - INFO_SEGMENT_ITEM_COUNT;
    if operands_count < results_count {
        if let Some(block_index) = option_block_index {
            return Err(EngineError::InvalidOperation(
                InvalidOperation::NotEnoughOperandForBlockResult {
                    vm_module_index,
                    function_index,
                    block_index: *block_index,
                    results_count,
                    operands_count,
                },
            ));
        } else {
            return Err(EngineError::InvalidOperation(
                InvalidOperation::NotEnoughOperandForFunctionResult {
                    vm_module_index,
                    function_index,
                    results_count,
                    operands_count,
                },
            ));
        }
    }

    // 判断返回值的数据类型
    let results = vm.stack.peek_values(results_count);
    match check_value_types(results, &result_types) {
        Err(ValueTypeCheckError::LengthMismatch) => unreachable!("argument count should be match"),
        Err(ValueTypeCheckError::DataTypeMismatch(result_index)) => {
            if let Some(block_index) = option_block_index {
                return Err(EngineError::TypeMismatch(
                    TypeMismatch::BlockResultTypeMismatch {
                        vm_module_index,
                        function_index,
                        block_index: *block_index,
                        result_index,
                        result_type: result_types[result_index].clone(),
                        value_type: results[result_index].get_type(),
                    },
                ));
            } else {
                return Err(EngineError::TypeMismatch(
                    TypeMismatch::FunctionResultTypeMismatch {
                        vm_module_index,
                        function_index,
                        result_index,
                        result_type: result_types[result_index].clone(),
                        value_type: results[result_index].get_type(),
                    },
                ));
            }
        }
        _ => {
            // pass
        }
    }

    let (vm_module_index, function_index, frame_type, address) = vm.pop_frame(results_count);

    // 上一句 vm.pop_frame() 调用已经更新了部分 vm.status。
    // 如果 vm.status.frame_pointer 的值等于 0，说明刚才弹出的栈帧是
    // 首个函数调用的栈帧，也就是说，当这个帧弹出之后，所有栈帧都已经弹出，
    // 意味着所有函数调用已经执行完毕，即程序已经结束。
    let is_program_end = vm.status.frame_pointer == INITIAL_FRAME_POINTER;

    if is_program_end {
        Ok(ControlResult::ProgramEnd)
    } else {
        Ok(ControlResult::PopStackFrame {
            is_call_frame: is_call_frame,
            vm_module_index,
            function_index,
            frame_type,
            address,
        })
    }
}

pub fn process_unreachable(_vm: &mut VM) -> Result<ControlResult, EngineError> {
    Err(EngineError::InvalidOperation(InvalidOperation::Unreachable))
}

pub fn process_nop(_vm: &mut VM) -> Result<ControlResult, EngineError> {
    // 无需任何操作
    Ok(ControlResult::Sequence)
}
