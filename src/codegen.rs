use thiserror::Error;
use inkwell::{builder::{Builder, BuilderError}, context::Context, module::Module, passes::PassManager, support::LLVMString, types::BasicMetadataTypeEnum, values::{FunctionValue, PointerValue}, IntPredicate};

use crate::parser::Instruction;

pub const TAPE_LEN: u16 = u16::MAX; // Note: Tape len may not be simply changed as type of head is i16 and we rely on default unsigned wrap-around.

#[derive(Error, PartialEq, Eq, Debug)]
pub enum CompileError {
    #[error("LLVM builder error: {0}")]
    BuilderError(#[from] BuilderError),
    #[error("Generated function failed LLVM verification.")]
    FunctionVerifyError,
    #[error("Invalid loop.")]
    UnbalancedLoopError,
    #[error("Failed to find function.")]
    LibraryLinkageError
}

fn compile_instructions<'ctx, 'i>(context: &'ctx Context, module: &Module<'ctx>, builder: &Builder<'ctx>, main: FunctionValue<'ctx>, head: PointerValue<'ctx>, tape: PointerValue<'ctx>, instructions: impl IntoIterator<Item = &'i Instruction>) -> Result<(), CompileError> {
    for instruction in instructions {
        match instruction {
            Instruction::MoveLeft => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();

                let new_head = builder.build_int_sub(
                    head_value, 
                    context.i16_type().const_int(1, false), 
                    "new_head"
                )?;

                // update head
                builder.build_store(head, new_head)?;
            },
            Instruction::MoveRight => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();

                let new_head = builder.build_int_add(
                    head_value, 
                    context.i16_type().const_int(1, false), 
                    "new_head"
                )?;

                // update head
                builder.build_store(head, new_head)?;
            },
            Instruction::Increment => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                
                let cell_value = builder.build_load(context.i8_type(), cell, "cell")?.into_int_value();
                let incremented = builder.build_int_add(cell_value, context.i8_type().const_int(1, false), "cell_inc")?;

                builder.build_store(cell, incremented)?;
            },
            Instruction::Decrement => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                
                let cell_value = builder.build_load(context.i8_type(), cell, "cell")?.into_int_value();
                let decremented = builder.build_int_sub(cell_value, context.i8_type().const_int(1, false), "cell_dec")?;
                
                builder.build_store(cell, decremented)?;
            },
            Instruction::Output => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                let cell_value = builder.build_load(context.i8_type(), cell, "cell")?.into_int_value();
                
                builder.build_call(
                    module.get_function("putchar").ok_or(CompileError::LibraryLinkageError)?,
                    &[cell_value.into()],
                    "putchar",
                )?;
            },
            Instruction::Input => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                
                let read = builder.build_call(
                    module.get_function("getchar").ok_or(CompileError::LibraryLinkageError)?,
                    &[],
                    "getchar",
                )?.try_as_basic_value().left().ok_or(CompileError::LibraryLinkageError)?;

                builder.build_store(cell, read)?;
            },
            Instruction::Loop(content) => {
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                let cell_value = builder.build_load(context.i8_type(), cell, "cell")?.into_int_value();
                
                let loop_block = context.append_basic_block(main, "loop");
                let remain_block = context.append_basic_block(main, "loop_remain");

                builder.build_conditional_branch(
                    builder.build_int_compare(IntPredicate::EQ, cell_value, context.i8_type().const_int(0, false), "cell_eq_zero")?,
                    remain_block,
                    loop_block,
                )?;

                builder.position_at_end(loop_block);

                compile_instructions(context, module, builder, main, head, tape, &**content)?;

                // re-load in case of update
                let head_value = builder.build_load(context.i16_type(), head, "load_head")?.into_int_value();
                let cell = unsafe { builder.build_gep(context.i16_type(), tape, &[head_value], "cell_ptr")? };
                let cell_value = builder.build_load(context.i8_type(), cell, "cell")?.into_int_value();

                builder.build_conditional_branch(
                    builder.build_int_compare(IntPredicate::NE, cell_value, context.i8_type().const_int(0, false), "cell_eq_zero")?,
                    loop_block,
                    remain_block,
                )?;

                builder.position_at_end(remain_block);
            }
        }
    }

    Ok(())
}

/// Compiles the given brainf*ck program.
pub fn compile<'i>(program: impl IntoIterator<Item = &'i Instruction>) -> Result<LLVMString, CompileError> {
    let context = Context::create();
    let module = context.create_module("bf");
    let builder = context.create_builder();
    
    // declare i/o functions.
    module.add_function("putchar", context.i16_type().fn_type(&[BasicMetadataTypeEnum::IntType(context.i8_type())], false), None); // libc putchar
    module.add_function("getchar", context.i8_type().fn_type(&[], false), None); // libc getchar
    
    // wrapper function for code.
    let main = module.add_function("main", context.void_type().fn_type(&[], false), None);

    let entry = context.append_basic_block(main, "entry");
    builder.position_at_end(entry);

    // stack-allocate the tape.
    let head = builder.build_alloca(context.i16_type(), "head")?; // TODO: currently an index value but potentially use pointer + offsetting on move instead of look-up on cell access.
    //let tape = builder.build_array_alloca(context.i8_type(), context.i8_type().const_int(TAPE_LEN, false), "tape")?;
    let tape = builder.build_alloca(context.i8_type().array_type(TAPE_LEN as u32), "tape")?;
    
    builder.build_store(head, context.i16_type().const_int(0, false))?;

    compile_instructions(&context, &module, &builder, main, head, tape, program)?;

    builder.build_return(None)?;

    if main.verify(false) {
        let fpm = PassManager::create(&module);
        
        // only available up to llvm14 (see inkwell doc)
        // fpm.add_instruction_combining_pass();
        // fpm.add_reassociate_pass();
        // fpm.add_gvn_pass();
        // fpm.add_cfg_simplification_pass();
        // fpm.add_basic_alias_analysis_pass();
        // fpm.add_promote_memory_to_register_pass();
        // fpm.add_instruction_combining_pass();
        // fpm.add_reassociate_pass();

        fpm.initialize();
        
        fpm.run_on(&main);
    } else {
        return Err(CompileError::FunctionVerifyError)
    }

    Ok(module.print_to_string())
}