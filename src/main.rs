use melior::ir::attribute::{ArrayAttribute, IntegerAttribute, StringAttribute, TypeAttribute};
use melior::ir::operation::{OperationLike, OperationPrintingFlags};
use melior::ir::r#type::IntegerType;
use melior::ir::{Attribute, AttributeLike, Block, BlockLike, Location, Region, RegionLike, Type, TypeLike};
use melior::Context;
use melior::dialect::ods::{builtin, hw, sv};

use circt_sv_attrs::sv::svMacroIdentAttrGetAlt2;

macro_rules! here {
    ($c:ident) => {
        Location::new(&$c, file!(), line!() as usize, column!() as usize)
    }
}

fn create_hw_module() -> String 
{
    let ctx = Context::new();
    let hw_handle = melior::dialect::DialectHandle::hw();
    hw_handle.load_dialect(&ctx);    
    let sv_handle = melior::dialect::DialectHandle::sv();
    sv_handle.load_dialect(&ctx);

    // Build top block
    let top_block = Block::new(&[]);

    /*
    sv.macro.decl @RANDOM
    sv.macro.decl @PRINTF_COND_
    sv.macro.decl @SYNTHESIS
     */
    let macro_decl = sv::macro_decl(&ctx, StringAttribute::new(&ctx, "RANDOM"), here!(ctx));
    top_block.append_operation(macro_decl.as_operation().clone());
    let macro_decl = sv::macro_decl(&ctx, StringAttribute::new(&ctx, "PRINTF_COND_"), here!(ctx));
    top_block.append_operation(macro_decl.as_operation().clone());
    let macro_decl = sv::macro_decl(&ctx, StringAttribute::new(&ctx, "SYNTHESIS"), here!(ctx));
    top_block.append_operation(macro_decl.as_operation().clone());

    // Now the body block
    let i1_type = IntegerType::new(&ctx, 1);
    let i8_type = IntegerType::new(&ctx, 8);

    let body_block = Block::new(&[]);
    // Body blocks have the same args as the module's ports
    let arg0 = body_block.add_argument(i1_type.clone().into(), here!(ctx));    
    let _arg1 = body_block.add_argument(i1_type.clone().into(), here!(ctx));    
    let _arg8 = body_block.add_argument(i8_type.clone().into(), here!(ctx));    
    
    /* %fd = hw.constant 0x80000002 : i32 */
    let i32_type = IntegerType::new(&ctx,32);
    let arith_constant = hw::constant(&ctx,
                                i32_type.clone().into(),
                                IntegerAttribute::new(i32_type.clone().into(), 0x80000002).into(), 
                                here!(ctx)); 
    /* Equivalent low level code:
    let arith_constant = melior::ir::operation::OperationBuilder::new("hw.constant", here!(ctx))
        .add_attributes(&[(melior::ir::Identifier::new(&ctx, "value"),
                            IntegerAttribute::new(i32_type.clone().into(), 0x80000002).into())])
        .add_results(&[i32_type.into()])
        .build()
        .expect("valid operation");*/
    body_block.append_operation(arith_constant.into());

    /* %param_x = sv.localparam {value = 11 : i42} : i42 */
    let i42_type = IntegerType::new(&ctx, 42);
    let param = sv::localparam(&ctx, i42_type.into(),
                                IntegerAttribute::new(i42_type.into(), 11).into(), 
                                StringAttribute::new(&ctx, "x"), here!(ctx));
    /* Equivalent low level code:
    let param = melior::ir::operation::OperationBuilder::new("sv.localparam", here!(ctx))
        .add_attributes(&[(melior::ir::Identifier::new(&ctx, "value"),
                            IntegerAttribute::new(i42_type.clone().into(), 11).into()),
                            (melior::ir::Identifier::new(&ctx, "name"),
                            StringAttribute::new(&ctx, "param_x").into())])
        .add_results(&[i42_type.into()])
        .build()
        .expect("valid operation");*/

    body_block.append_operation(param.into());

    let always_region = Region::new();
    let always_block = Block::new(&[]);
    let if_block = Block::new(&[]);    
    let if_region = Region::new(); // Block::new(&[]);
    if_region.append_block(if_block);
    let else_block = Block::new(&[]);
    let else_region = Region::new();
    else_region.append_block(else_block);

    let macro_ident = StringAttribute::new(&ctx, "SYNTHESIS");
    let macro_ref = unsafe { Attribute::from_raw(svMacroIdentAttrGetAlt2(macro_ident.to_raw())) };
    let ifdef_op = sv::ifdef_procedural(&ctx, if_region, else_region, macro_ref.into(), here!(ctx));

    always_block.append_operation(ifdef_op.into());

    // sv.always posedge %arg0
    always_region.append_block(always_block);
    // posedge = 0
    let posedge = IntegerAttribute::new(IntegerType::new(&ctx, 32).into(), 0 as i64);
    let events = ArrayAttribute::new(&ctx, &[posedge.into()]);
    let sv_always = sv::always(&ctx, &[arg0], always_region, events, here!(ctx));
    body_block.append_operation(sv_always.into());

    let hw_output = hw::output(&ctx, &[], here!(ctx));
    body_block.append_operation(hw_output.into());

    let body_region = Region::new();
    body_region.append_block(body_block);

    // Create the module
    let sym_name = StringAttribute::new(&ctx, "test1");
    let mod_ports = [
        mlir_sys::HWModulePort {
            name: StringAttribute::new(&ctx, "arg0").to_raw(),
            type_: i1_type.clone().to_raw(),
            dir: mlir_sys::HWModulePortDirection_Input
        },
        mlir_sys::HWModulePort {
            name: StringAttribute::new(&ctx, "arg1").to_raw(),
            type_: i1_type.to_raw(),
            dir: mlir_sys::HWModulePortDirection_Input
        },        
        mlir_sys::HWModulePort {
            name: StringAttribute::new(&ctx, "arg8").to_raw(),
            type_: i8_type.to_raw(),
            dir: mlir_sys::HWModulePortDirection_Input
        }
    ];
    let module_type = TypeAttribute::new(unsafe { 
        Type::from_raw(mlir_sys::hwModuleTypeGet(ctx.to_raw(), 
                                                    mod_ports.len() as isize, 
                                                    std::mem::transmute(&mod_ports))) 
    });
    let parameters = ArrayAttribute::new(&ctx, &[]); 

    let module = hw::module(&ctx,
                            body_region,
                            sym_name,
                            module_type,
                            parameters,
                            here!(ctx));

    top_block.append_operation(module.into());

    let top_region = Region::new();
    top_region.append_block(top_block);
    let top = builtin::module(&ctx, top_region, here!(ctx));

    unsafe {
        if mlir_sys::mlirOperationVerify(top.as_operation().to_raw()) {
                eprintln!("Verification passed!");
            } else {
                eprintln!("Verification failed :-(");
            }
    }
    let flags = OperationPrintingFlags::default();
    let text = top.as_operation().to_string_with_flags(flags).unwrap();
    text    
}

fn main() {
    println!("{}", create_hw_module());
}
