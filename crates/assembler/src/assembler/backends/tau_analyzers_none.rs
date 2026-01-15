use crate::{
    assembler::{AssemblerError, backends::Backend},
    lexer::token::Span,
    parser::{DefineMap, LabelMap, operations::OperationWithArgs},
};

fn assemble_2reg(register: u8, register1: u8) -> u16 {
    (register as u16) << 8 | register1 as u16
}

pub fn assemble_operation(
    defines: &DefineMap,
    labels: &LabelMap,
    operation: OperationWithArgs,
    span: Span,
) -> Result<Vec<u8>, AssemblerError> {
    let backend = Backend::TauAnalyzersNone;

    // match operation {
    //     OperationWithArgs::Add2(register, register1) => todo!(),
    // }

    Ok(vec![])
}
