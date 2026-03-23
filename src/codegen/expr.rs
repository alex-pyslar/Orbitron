use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, FloatValue, IntValue};

use crate::parser::ast::{BinOp, Expr, UnaryOp};
use super::{CodeGen, ConstVal, Val, Var, VarKind};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Val<'ctx> {
        match expr {
            Expr::Number(n)  => Val::Int(self.i64_ty.const_int(*n as u64, true)),
            Expr::Float(f)   => Val::Float(self.f64_ty.const_float(*f)),
            Expr::Str(_)     => panic!("String literals are only allowed inside println()"),
            Expr::Interpolated(_) => panic!(
                "Interpolated strings are only allowed inside println()"
            ),

            Expr::Ident(name) => {
                // Check compile-time constants first  (from Rust / C++)
                if let Some(cv) = self.consts.get(name).cloned() {
                    return match cv {
                        ConstVal::Int(n)   => Val::Int(self.i64_ty.const_int(n as u64, true)),
                        ConstVal::Float(f) => Val::Float(self.f64_ty.const_float(f)),
                    };
                }
                let var = self.vars.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name));
                match var.kind {
                    VarKind::Float =>
                        Val::Float(
                            self.builder.build_load(self.f64_ty, var.ptr, name)
                                .unwrap().into_float_value(),
                        ),
                    VarKind::Int =>
                        Val::Int(
                            self.builder.build_load(self.i64_ty, var.ptr, name)
                                .unwrap().into_int_value(),
                        ),
                    VarKind::Struct(type_name) =>
                        Val::Struct(var.ptr, type_name),
                    VarKind::Array | VarKind::Tuple =>
                        Val::Array(var.ptr),
                    VarKind::FnPtr =>
                        Val::Int(self.builder.build_load(self.i64_ty, var.ptr, name)
                            .unwrap().into_int_value()),
                }
            }

            // cond ? then : els  — ternary operator  (from C / Java)
            Expr::Ternary { cond, then, els } => {
                let cond_i1  = self.bool_cond(cond);
                let func     = self.cur_fn();
                let then_bb  = self.ctx.append_basic_block(func, "tern.then");
                let else_bb  = self.ctx.append_basic_block(func, "tern.else");
                let merge_bb = self.ctx.append_basic_block(func, "tern.merge");

                self.builder.build_conditional_branch(cond_i1, then_bb, else_bb).unwrap();

                // then branch
                self.builder.position_at_end(then_bb);
                let then_val = self.gen_expr(then);
                let then_int = self.as_int(then_val);
                let then_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // else branch
                self.builder.position_at_end(else_bb);
                let else_val = self.gen_expr(els);
                let else_int = self.as_int(else_val);
                let else_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // merge — phi node selects the value
                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(self.i64_ty, "tern.val").unwrap();
                phi.add_incoming(&[(&then_int, then_end), (&else_int, else_end)]);
                Val::Int(phi.as_basic_value().into_int_value())
            }

            // [expr, ...]  — array literal  (from Python / JS)
            Expr::ArrayLit(exprs) => {
                let n    = exprs.len() as u64;
                let size = self.i64_ty.const_int(n, false);
                let alloca = self.builder
                    .build_array_alloca(self.i64_ty, size, "arr")
                    .unwrap();
                for (i, e) in exprs.iter().enumerate() {
                    let v   = self.gen_expr(e);
                    let vi  = self.as_int(v);
                    let idx = self.i64_ty.const_int(i as u64, false);
                    let ptr = unsafe {
                        self.builder.build_gep(self.i64_ty, alloca, &[idx], "arr.init").unwrap()
                    };
                    self.builder.build_store(ptr, vi).unwrap();
                }
                Val::Array(alloca)
            }

            // expr[idx]  — array element access  (from Python / JS)
            Expr::Index { arr, idx } => {
                let arr_ptr = self.extract_array_ptr(arr);
                let idx_v   = { let v = self.gen_expr(idx); self.as_int(v) };
                let elem    = unsafe {
                    self.builder.build_gep(self.i64_ty, arr_ptr, &[idx_v], "idx.get").unwrap()
                };
                Val::Int(
                    self.builder.build_load(self.i64_ty, elem, "idx.val")
                        .unwrap().into_int_value()
                )
            }

            // obj.field
            Expr::FieldAccess { obj, field } => {
                // Check if this is an enum variant access: EnumName.Variant  (from Rust/Swift)
                if let Expr::Ident(enum_name) = obj.as_ref() {
                    if let Some(variants) = self.enums.get(enum_name).cloned() {
                        if let Some(&val) = variants.get(field) {
                            return Val::Int(self.i64_ty.const_int(val as u64, true));
                        }
                    }
                }
                // Ordinary struct field access
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let type_name = type_name.clone();
                    let field_info = self.struct_fields.get(&type_name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Unknown struct '{}'", type_name));
                    let idx = field_info.iter().position(|(n, _)| n == field)
                        .unwrap_or_else(|| panic!("Unknown field '{}' in '{}'", field, type_name));
                    let (_, is_float) = field_info[idx];
                    let st  = *self.struct_types.get(&type_name)
                        .unwrap_or_else(|| panic!("Unknown struct type '{}'", type_name));
                    let gep = self.builder
                        .build_struct_gep(st, ptr, idx as u32, "fget")
                        .unwrap();
                    if is_float {
                        Val::Float(
                            self.builder.build_load(self.f64_ty, gep, field)
                                .unwrap().into_float_value(),
                        )
                    } else {
                        Val::Int(
                            self.builder.build_load(self.i64_ty, gep, field)
                                .unwrap().into_int_value(),
                        )
                    }
                } else {
                    panic!("Field access on a non-struct value");
                }
            }

            // obj.method(args)
            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let func_name = format!("{}_{}", type_name, method);
                    let callee = self.module.get_function(&func_name)
                        .unwrap_or_else(|| panic!("Unknown method '{}'", func_name));
                    let mut argv: Vec<BasicMetadataValueEnum> = vec![ptr.into()];
                    argv.extend(args.iter().map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    }));
                    let result = self.builder
                        .build_call(callee, &argv, "mcall")
                        .unwrap()
                        .try_as_basic_value()
                        .expect_basic("method must return a value")
                        .into_int_value();
                    Val::Int(result)
                } else {
                    panic!("Method call on a non-struct value");
                }
            }

            // StructName { field: expr, ... }  — struct literal (no `new`)
            Expr::StructLit { name, fields } => {
                let field_info = self.struct_fields.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Unknown struct '{}'", name));
                let st = *self.struct_types.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type '{}'", name));
                let alloca = self.builder
                    .build_alloca(st, &format!("{}.new", name))
                    .unwrap();
                for (fname, fexpr) in fields {
                    let idx = field_info.iter().position(|(n, _)| n == fname)
                        .unwrap_or_else(|| panic!("Unknown field '{}' in '{}'", fname, name));
                    let (_, is_float) = field_info[idx];
                    let gep = self.builder
                        .build_struct_gep(st, alloca, idx as u32, "finit")
                        .unwrap();
                    let v = self.gen_expr(fexpr);
                    if is_float {
                        self.builder.build_store(gep, self.as_float(v)).unwrap();
                    } else {
                        self.builder.build_store(gep, self.as_int(v)).unwrap();
                    }
                }
                Val::Struct(alloca, name.clone())
            }

            // new ClassName(args)  — allocate struct then call ClassName_new(ptr, args)
            Expr::ConstructorCall { class, args } => {
                let st = *self.struct_types.get(class)
                    .unwrap_or_else(|| panic!("Unknown class '{}' in constructor", class));
                let ptr = self.builder
                    .build_alloca(st, &format!("{}.ctor", class))
                    .unwrap();

                let ctor_name = format!("{}_new", class);
                if let Some(ctor) = self.module.get_function(&ctor_name) {
                    let mut argv: Vec<BasicMetadataValueEnum> = vec![ptr.into()];
                    argv.extend(args.iter().map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    }));
                    self.builder.build_call(ctor, &argv, "ctor_call").unwrap();
                } else if !args.is_empty() {
                    panic!(
                        "Class '{}' has no 'init' constructor but was called with {} argument(s). \
                         Define 'init(...)' inside the class.",
                        class,
                        args.len()
                    );
                }
                Val::Struct(ptr, class.clone())
            }

            // readInt()  — read one i64 from stdin via scanf("%lld", &tmp)
            Expr::Input => {
                let alloca = self.builder.build_alloca(self.i64_ty, "input_tmp").unwrap();
                let fmt    = self.fmt_ptr("%lld", "fmt.scan.int");
                let scanf  = self.module.get_function("scanf").unwrap();
                self.builder
                    .build_call(scanf, &[fmt.into(), alloca.into()], "scanf.int")
                    .unwrap();
                Val::Int(
                    self.builder
                        .build_load(self.i64_ty, alloca, "input_val")
                        .unwrap()
                        .into_int_value(),
                )
            }

            // readFloat()  — read one f64 from stdin via scanf("%lf", &tmp)
            Expr::InputFloat => {
                let alloca = self.builder.build_alloca(self.f64_ty, "inputf_tmp").unwrap();
                let fmt    = self.fmt_ptr("%lf", "fmt.scan.flt");
                let scanf  = self.module.get_function("scanf").unwrap();
                self.builder
                    .build_call(scanf, &[fmt.into(), alloca.into()], "scanf.flt")
                    .unwrap();
                Val::Float(
                    self.builder
                        .build_load(self.f64_ty, alloca, "inputf_val")
                        .unwrap()
                        .into_float_value(),
                )
            }

            // &expr — address-of: returns the alloca pointer as an i64
            Expr::AddrOf(inner) => {
                let ptr_ty = self.ctx.ptr_type(inkwell::AddressSpace::default());
                let raw_ptr = match inner.as_ref() {
                    Expr::Ident(name) => {
                        let var = self.vars.get(name).cloned()
                            .unwrap_or_else(|| panic!("&: undefined variable '{}'", name));
                        var.ptr
                    }
                    _ => panic!("& (address-of) only works with a variable name"),
                };
                let _ = ptr_ty; // suppress unused warning
                Val::Int(
                    self.builder.build_ptr_to_int(raw_ptr, self.i64_ty, "addr").unwrap()
                )
            }

            // *expr — dereference: interpret i64 as a pointer and load i64 from it
            Expr::Deref(inner) => {
                let ptr_ty = self.ctx.ptr_type(inkwell::AddressSpace::default());
                let v   = self.gen_expr(inner);
                let addr = self.as_int(v);
                let ptr = self.builder.build_int_to_ptr(addr, ptr_ty, "deref.ptr").unwrap();
                Val::Int(
                    self.builder.build_load(self.i64_ty, ptr, "deref.val")
                        .unwrap().into_int_value()
                )
            }

            // cstr("literal") — address of a null-terminated C string stored as a global
            Expr::CStr(s) => {
                let key = format!("cstr.{:x}", super::fxhash(s));
                let ptr = match self.module.get_global(&key) {
                    Some(g) => g.as_pointer_value(),
                    None    => self.builder
                        .build_global_string_ptr(s, &key)
                        .unwrap()
                        .as_pointer_value(),
                };
                Val::Int(
                    self.builder.build_ptr_to_int(ptr, self.i64_ty, "cstr.addr").unwrap()
                )
            }

            Expr::Unary(op, inner) => {
                let v = self.gen_expr(inner);
                match op {
                    UnaryOp::Neg => match v {
                        Val::Int(i)       => Val::Int(self.builder.build_int_neg(i, "neg").unwrap()),
                        Val::Float(f)     => Val::Float(self.builder.build_float_neg(f, "fneg").unwrap()),
                        Val::Struct(_, n) => panic!("Cannot negate struct '{}'", n),
                        Val::Array(_)     => panic!("Cannot negate an array"),
                    },
                    UnaryOp::Not => {
                        let i   = self.as_int(v);
                        let cmp = self.builder.build_int_compare(
                            inkwell::IntPredicate::EQ, i, self.i64_ty.const_zero(), "not.cmp",
                        ).unwrap();
                        Val::Int(self.builder.build_int_z_extend(cmp, self.i64_ty, "not.ext").unwrap())
                    }
                    // ~x — bitwise NOT  (from C / Java)
                    UnaryOp::BitNot => {
                        let i        = self.as_int(v);
                        let all_ones = self.i64_ty.const_all_ones();
                        Val::Int(self.builder.build_xor(i, all_ones, "bitnot").unwrap())
                    }
                }
            }

            Expr::Binary(lhs, op, rhs) => {
                let l = self.gen_expr(lhs);
                let r = self.gen_expr(rhs);
                self.gen_binop(l, r, op)
            }

            // Type::method(args)  — static method call  (from C++ / Rust)
            Expr::StaticCall { type_name, method, args } => {
                let func_name = format!("{}_{}", type_name, method);
                let callee = self.module.get_function(&func_name)
                    .unwrap_or_else(|| panic!("Unknown static method '{}'", func_name));
                let argv: Vec<BasicMetadataValueEnum> = args.iter()
                    .map(|a| { let v = self.gen_expr(a); BasicMetadataValueEnum::IntValue(self.as_int(v)) })
                    .collect();
                let result = self.builder
                    .build_call(callee, &argv, "scall")
                    .unwrap()
                    .try_as_basic_value()
                    .expect_basic("static method must return a value")
                    .into_int_value();
                Val::Int(result)
            }

            // (a, b, ...)  — tuple literal  (from Python / Rust)
            // Stored as a flat i64 array on the stack.
            Expr::Tuple(exprs) => {
                let n    = exprs.len() as u64;
                let size = self.i64_ty.const_int(n, false);
                let alloca = self.builder
                    .build_array_alloca(self.i64_ty, size, "tuple")
                    .unwrap();
                for (i, e) in exprs.iter().enumerate() {
                    let v   = self.gen_expr(e);
                    let vi  = self.as_int(v);
                    let idx = self.i64_ty.const_int(i as u64, false);
                    let ptr = unsafe {
                        self.builder.build_gep(self.i64_ty, alloca, &[idx], "tuple.init").unwrap()
                    };
                    self.builder.build_store(ptr, vi).unwrap();
                }
                Val::Array(alloca)
            }

            // |params| expr  — lambda / closure  (from Rust / Python)
            Expr::Lambda { params, body } => {
                let name = format!("__lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;

                let ptys: Vec<BasicMetadataTypeEnum> = params.iter()
                    .map(|_| BasicMetadataTypeEnum::from(self.i64_ty))
                    .collect();
                let fn_ty = self.i64_ty.fn_type(&ptys, false);
                let func  = self.module.add_function(&name, fn_ty, None);

                // Save caller context
                let outer_block    = self.builder.get_insert_block().unwrap();
                let outer_vars     = std::mem::take(&mut self.vars);
                let outer_deferred = std::mem::take(&mut self.deferred);
                let outer_arrays   = std::mem::take(&mut self.array_lens);

                // Generate lambda body
                let entry = self.ctx.append_basic_block(func, "entry");
                self.builder.position_at_end(entry);
                for (i, pname) in params.iter().enumerate() {
                    let alloca = self.builder.build_alloca(self.i64_ty, pname).unwrap();
                    let pval   = func.get_nth_param(i as u32).unwrap().into_int_value();
                    self.builder.build_store(alloca, pval).unwrap();
                    self.vars.insert(pname.clone(), Var { ptr: alloca, kind: VarKind::Int });
                }
                let ret_val = self.gen_expr(body);
                let ret_int = self.as_int(ret_val);
                self.builder.build_return(Some(&ret_int)).unwrap();

                // Restore caller context
                self.vars       = outer_vars;
                self.deferred   = outer_deferred;
                self.array_lens = outer_arrays;
                self.builder.position_at_end(outer_block);

                // Return function pointer as i64
                Val::Int(
                    self.builder
                        .build_ptr_to_int(func.as_global_value().as_pointer_value(), self.i64_ty, "lambda.ptr")
                        .unwrap()
                )
            }

            // match expr { pat => val, ... }  — match as expression  (from Rust)
            Expr::MatchExpr { expr, arms } => {
                let val = self.gen_expr(expr);
                let v   = self.as_int(val);
                let func = self.cur_fn();
                let merge_bb = self.ctx.append_basic_block(func, "mexpr.end");

                let arm_bbs: Vec<BasicBlock> = (0..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("mexpr.arm.{}", i)))
                    .collect();
                let check_bbs: Vec<BasicBlock> = (1..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("mexpr.chk.{}", i)))
                    .collect();

                // Dispatch to first arm
                if let Some(arm) = arms.first() {
                    match self.resolve_match_pat(&arm.pat) {
                        Some(pv) => {
                            let cmp = self.builder.build_int_compare(inkwell::IntPredicate::EQ, v, pv, "mc").unwrap();
                            let next = if check_bbs.is_empty() { merge_bb } else { check_bbs[0] };
                            self.builder.build_conditional_branch(cmp, arm_bbs[0], next).unwrap();
                        }
                        None => { self.builder.build_unconditional_branch(arm_bbs[0]).unwrap(); }
                    }
                }

                let mut incoming: Vec<(IntValue<'ctx>, BasicBlock<'ctx>)> = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    self.builder.position_at_end(arm_bbs[i]);
                    let arm_val = self.gen_expr(&arm.val);
                    let arm_int = self.as_int(arm_val);
                    let arm_end = self.builder.get_insert_block().unwrap();
                    incoming.push((arm_int, arm_end));
                    self.builder.build_unconditional_branch(merge_bb).unwrap();

                    if i + 1 < arms.len() {
                        self.builder.position_at_end(check_bbs[i]);
                        let next_arm = arm_bbs[i + 1];
                        let fall = if i + 1 < check_bbs.len() { check_bbs[i + 1] } else { merge_bb };
                        match self.resolve_match_pat(&arms[i + 1].pat) {
                            Some(pv) => {
                                let cmp = self.builder.build_int_compare(inkwell::IntPredicate::EQ, v, pv, "mc").unwrap();
                                self.builder.build_conditional_branch(cmp, next_arm, fall).unwrap();
                            }
                            None => { self.builder.build_unconditional_branch(next_arm).unwrap(); }
                        }
                    }
                }

                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(self.i64_ty, "mexpr.val").unwrap();
                for (ival, bb) in &incoming {
                    phi.add_incoming(&[(ival as &dyn inkwell::values::BasicValue, *bb)]);
                }
                Val::Int(phi.as_basic_value().into_int_value())
            }

            // name!(args) — macro-style call: dispatch same as regular calls
            // println!(x) → same as println(x); assert!(x) → same; etc.
            Expr::MacroCall { name, args } => {
                if name == "println" {
                    let e = args.first().cloned().unwrap_or(Expr::Number(0));
                    match &e {
                        Expr::Str(s) => self.print_str(s),
                        Expr::Interpolated(parts) => {
                            let parts = parts.clone();
                            self.print_interpolated(&parts);
                        }
                        _ => match self.gen_expr(&e) {
                            Val::Int(i)       => self.print_int(i),
                            Val::Float(f)     => self.print_float(f),
                            Val::Struct(_, n) => panic!("Cannot print struct '{}' directly", n),
                            Val::Array(_)     => panic!("Cannot print an array directly"),
                        }
                    }
                    return Val::Int(self.i64_ty.const_int(0, false));
                }
                // For other macros, dispatch as a regular call
                return self.gen_expr(&Expr::Call { name: name.clone(), args: args.clone() });
            }

            // left ?: right — Elvis / null-coalescing  (Kotlin)
            // If left != 0, return left; otherwise return right.
            Expr::Elvis { left, right } => {
                // Evaluate left once before branching
                let left_val = self.gen_expr(left);
                let left_int = self.as_int(left_val);
                let func     = self.cur_fn();
                let nz_bb    = self.ctx.append_basic_block(func, "elvis.nz");
                let zero_bb  = self.ctx.append_basic_block(func, "elvis.zero");
                let merge_bb = self.ctx.append_basic_block(func, "elvis.merge");

                let cond = self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, left_int, self.i64_ty.const_zero(), "elvis.cond"
                ).unwrap();
                self.builder.build_conditional_branch(cond, nz_bb, zero_bb).unwrap();

                // non-zero branch: use already-computed left_int
                self.builder.position_at_end(nz_bb);
                // left_int is still valid here (it's an LLVM value from the current block)
                let nz_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // zero branch: return right
                self.builder.position_at_end(zero_bb);
                let rhs_val  = self.gen_expr(right);
                let rhs_int  = self.as_int(rhs_val);
                let zero_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(self.i64_ty, "elvis.val").unwrap();
                phi.add_incoming(&[(&left_int, nz_end), (&rhs_int, zero_end)]);
                Val::Int(phi.as_basic_value().into_int_value())
            }

            // expr?.field — optional chaining  (Swift / Kotlin)
            // If expr evaluates to 0 (null), return 0; otherwise access field.
            Expr::OptChain { expr: inner, field } => {
                let obj_val  = self.gen_expr(inner);
                let obj_int  = self.as_int(obj_val.clone());
                let func     = self.cur_fn();
                let acc_bb   = self.ctx.append_basic_block(func, "optch.acc");
                let null_bb  = self.ctx.append_basic_block(func, "optch.null");
                let merge_bb = self.ctx.append_basic_block(func, "optch.merge");

                let cond = self.builder.build_int_compare(
                    inkwell::IntPredicate::NE, obj_int, self.i64_ty.const_zero(), "optch.cond"
                ).unwrap();
                self.builder.build_conditional_branch(cond, acc_bb, null_bb).unwrap();

                // non-null: access the field
                self.builder.position_at_end(acc_bb);
                let accessed = self.gen_expr(&Expr::FieldAccess {
                    obj: inner.clone(),
                    field: field.clone(),
                });
                let acc_int = self.as_int(accessed);
                let acc_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // null: return 0
                self.builder.position_at_end(null_bb);
                let null_int = self.i64_ty.const_zero();
                let null_end = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                self.builder.position_at_end(merge_bb);
                let phi = self.builder.build_phi(self.i64_ty, "optch.val").unwrap();
                phi.add_incoming(&[(&acc_int, acc_end), (&null_int, null_end)]);
                Val::Int(phi.as_basic_value().into_int_value())
            }

            Expr::Call { name, args } => {
                // ── ptr_write(addr, val) — store i64 val at raw address addr ──
                if name == "ptr_write" {
                    if args.len() != 2 {
                        panic!("ptr_write(addr, val) takes exactly 2 arguments");
                    }
                    let ptr_ty = self.ctx.ptr_type(inkwell::AddressSpace::default());
                    let addr_v = { let v = self.gen_expr(&args[0]); self.as_int(v) };
                    let val_v  = { let v = self.gen_expr(&args[1]); self.as_int(v) };
                    let ptr    = self.builder.build_int_to_ptr(addr_v, ptr_ty, "pw.ptr").unwrap();
                    self.builder.build_store(ptr, val_v).unwrap();
                    return Val::Int(self.i64_ty.const_int(0, false));
                }

                // ── ptr_write_byte(addr, val) — store i8 at raw address ──
                if name == "ptr_write_byte" {
                    if args.len() != 2 {
                        panic!("ptr_write_byte(addr, val) takes exactly 2 arguments");
                    }
                    let ptr_ty  = self.ctx.ptr_type(inkwell::AddressSpace::default());
                    let i8_ty   = self.ctx.i8_type();
                    let addr_v  = { let v = self.gen_expr(&args[0]); self.as_int(v) };
                    let val_v   = { let v = self.gen_expr(&args[1]); self.as_int(v) };
                    let byte_v  = self.builder.build_int_truncate(val_v, i8_ty, "byte").unwrap();
                    let ptr     = self.builder.build_int_to_ptr(addr_v, ptr_ty, "pwb.ptr").unwrap();
                    self.builder.build_store(ptr, byte_v).unwrap();
                    return Val::Int(self.i64_ty.const_int(0, false));
                }

                // ── ptr_read(addr) — load i64 from raw address ──
                if name == "ptr_read" {
                    if args.len() != 1 {
                        panic!("ptr_read(addr) takes exactly 1 argument");
                    }
                    let ptr_ty = self.ctx.ptr_type(inkwell::AddressSpace::default());
                    let addr_v = { let v = self.gen_expr(&args[0]); self.as_int(v) };
                    let ptr    = self.builder.build_int_to_ptr(addr_v, ptr_ty, "pr.ptr").unwrap();
                    return Val::Int(
                        self.builder.build_load(self.i64_ty, ptr, "pr.val")
                            .unwrap().into_int_value()
                    );
                }

                // ── sign_ext(v) — sign-extend low 32 bits of v to i64 ──
                // Necessary when extern C functions return int (32-bit) in an i64.
                // Example: sign_ext(connect(...)) to correctly detect -1.
                if name == "sign_ext" {
                    if args.len() != 1 {
                        panic!("sign_ext(v) takes exactly 1 argument");
                    }
                    let i32_ty = self.ctx.i32_type();
                    let v    = self.gen_expr(&args[0]);
                    let i64v = self.as_int(v);
                    let i32v = self.builder.build_int_truncate(i64v, i32_ty, "trunc32").unwrap();
                    return Val::Int(
                        self.builder.build_int_s_extend(i32v, self.i64_ty, "sext64").unwrap()
                    );
                }

                // ── assert(cond) — abort on falsy condition ──
                if name == "assert" {
                    if args.len() != 1 {
                        panic!("assert(cond) takes exactly 1 argument");
                    }
                    let cond_i1 = self.bool_cond(&args[0]);
                    let func     = self.cur_fn();
                    let ok_bb    = self.ctx.append_basic_block(func, "assert.ok");
                    let fail_bb  = self.ctx.append_basic_block(func, "assert.fail");
                    self.builder.build_conditional_branch(cond_i1, ok_bb, fail_bb).unwrap();
                    self.builder.position_at_end(fail_bb);
                    let abort_fn = self.module.get_function("abort").unwrap();
                    self.builder.build_call(abort_fn, &[], "abort").unwrap();
                    self.builder.build_unreachable().unwrap();
                    self.builder.position_at_end(ok_bb);
                    return Val::Int(self.i64_ty.const_int(0, false));
                }

                // ── assert_eq(a, b) — abort if a != b ──
                if name == "assert_eq" {
                    if args.len() != 2 {
                        panic!("assert_eq(a, b) takes exactly 2 arguments");
                    }
                    let a = { let v = self.gen_expr(&args[0]); self.as_int(v) };
                    let b = { let v = self.gen_expr(&args[1]); self.as_int(v) };
                    let eq = self.builder.build_int_compare(inkwell::IntPredicate::EQ, a, b, "aeq").unwrap();
                    let func    = self.cur_fn();
                    let ok_bb   = self.ctx.append_basic_block(func, "aeq.ok");
                    let fail_bb = self.ctx.append_basic_block(func, "aeq.fail");
                    self.builder.build_conditional_branch(eq, ok_bb, fail_bb).unwrap();
                    self.builder.position_at_end(fail_bb);
                    let abort_fn = self.module.get_function("abort").unwrap();
                    self.builder.build_call(abort_fn, &[], "abort").unwrap();
                    self.builder.build_unreachable().unwrap();
                    self.builder.position_at_end(ok_bb);
                    return Val::Int(self.i64_ty.const_int(0, false));
                }

                // Fill in default parameters when call provides fewer args than declared
                let defaults = self.fn_defaults.get(name).cloned().unwrap_or_default();
                let mut call_args: Vec<Expr> = args.to_vec();
                if call_args.len() < defaults.len() {
                    for i in call_args.len()..defaults.len() {
                        if let Some(Some(default_expr)) = defaults.get(i) {
                            call_args.push(default_expr.clone());
                        } else {
                            panic!(
                                "Missing argument {} for function '{}' and no default provided",
                                i, name
                            );
                        }
                    }
                }

                let callee = self.module.get_function(name)
                    .unwrap_or_else(|| panic!("Undefined function '{}'", name));
                let argv: Vec<BasicMetadataValueEnum> = call_args.iter()
                    .map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    })
                    .collect();
                let result = self.builder
                    .build_call(callee, &argv, "call")
                    .unwrap()
                    .try_as_basic_value()
                    .expect_basic("function must return a value")
                    .into_int_value();
                Val::Int(result)
            }
        }
    }

    pub(super) fn gen_binop(&mut self, l: Val<'ctx>, r: Val<'ctx>, op: &BinOp) -> Val<'ctx> {
        // Power operator: always uses libm pow()  (from Python)
        if matches!(op, BinOp::Pow) {
            let both_int = !l.is_float() && !r.is_float();
            let lf = self.as_float(l);
            let rf = self.as_float(r);
            let pow_fn = self.module.get_function("pow").unwrap();
            let result = self.builder
                .build_call(pow_fn, &[lf.into(), rf.into()], "pow")
                .unwrap()
                .try_as_basic_value()
                .expect_basic("pow must return a value")
                .into_float_value();
            return if both_int {
                Val::Int(self.builder
                    .build_float_to_signed_int(result, self.i64_ty, "pow2i")
                    .unwrap())
            } else {
                Val::Float(result)
            };
        }

        if l.is_float() || r.is_float() {
            return self.float_binop(self.as_float(l), self.as_float(r), op);
        }
        let (li, ri) = (self.as_int(l), self.as_int(r));
        match op {
            BinOp::Add => Val::Int(self.builder.build_int_add(li, ri, "add").unwrap()),
            BinOp::Sub => Val::Int(self.builder.build_int_sub(li, ri, "sub").unwrap()),
            BinOp::Mul => Val::Int(self.builder.build_int_mul(li, ri, "mul").unwrap()),
            BinOp::Div => Val::Int(self.builder.build_int_signed_div(li, ri, "div").unwrap()),
            BinOp::Mod => Val::Int(self.builder.build_int_signed_rem(li, ri, "mod").unwrap()),
            BinOp::And => Val::Int(self.builder.build_and(li, ri, "and").unwrap()),
            BinOp::Or  => Val::Int(self.builder.build_or(li, ri, "or").unwrap()),
            // ^ XOR  (from C / Java)
            BinOp::Xor => Val::Int(self.builder.build_xor(li, ri, "xor").unwrap()),
            cmp        => Val::Int(self.int_cmp(li, ri, cmp)),
        }
    }

    fn float_binop(&mut self, l: FloatValue<'ctx>, r: FloatValue<'ctx>, op: &BinOp) -> Val<'ctx> {
        match op {
            BinOp::Add => Val::Float(self.builder.build_float_add(l, r, "fadd").unwrap()),
            BinOp::Sub => Val::Float(self.builder.build_float_sub(l, r, "fsub").unwrap()),
            BinOp::Mul => Val::Float(self.builder.build_float_mul(l, r, "fmul").unwrap()),
            BinOp::Div => Val::Float(self.builder.build_float_div(l, r, "fdiv").unwrap()),
            BinOp::Mod => Val::Float(self.builder.build_float_rem(l, r, "fmod").unwrap()),
            BinOp::And => {
                let (li, ri) = (self.as_int(Val::Float(l)), self.as_int(Val::Float(r)));
                Val::Int(self.builder.build_and(li, ri, "and").unwrap())
            }
            BinOp::Or => {
                let (li, ri) = (self.as_int(Val::Float(l)), self.as_int(Val::Float(r)));
                Val::Int(self.builder.build_or(li, ri, "or").unwrap())
            }
            cmp => Val::Int(self.float_cmp(l, r, cmp)),
        }
    }

    fn int_cmp(&self, l: IntValue<'ctx>, r: IntValue<'ctx>, op: &BinOp) -> IntValue<'ctx> {
        let pred = match op {
            BinOp::Eq => inkwell::IntPredicate::EQ,
            BinOp::Ne => inkwell::IntPredicate::NE,
            BinOp::Lt => inkwell::IntPredicate::SLT,
            BinOp::Le => inkwell::IntPredicate::SLE,
            BinOp::Gt => inkwell::IntPredicate::SGT,
            BinOp::Ge => inkwell::IntPredicate::SGE,
            _         => unreachable!(),
        };
        let bit = self.builder.build_int_compare(pred, l, r, "icmp").unwrap();
        self.builder.build_int_s_extend(bit, self.i64_ty, "icmp.ext").unwrap()
    }

    fn float_cmp(&self, l: FloatValue<'ctx>, r: FloatValue<'ctx>, op: &BinOp) -> IntValue<'ctx> {
        let pred = match op {
            BinOp::Eq => inkwell::FloatPredicate::OEQ,
            BinOp::Ne => inkwell::FloatPredicate::ONE,
            BinOp::Lt => inkwell::FloatPredicate::OLT,
            BinOp::Le => inkwell::FloatPredicate::OLE,
            BinOp::Gt => inkwell::FloatPredicate::OGT,
            BinOp::Ge => inkwell::FloatPredicate::OGE,
            _         => unreachable!(),
        };
        let bit = self.builder.build_float_compare(pred, l, r, "fcmp").unwrap();
        self.builder.build_int_s_extend(bit, self.i64_ty, "fcmp.ext").unwrap()
    }

    /// Extract a raw array pointer from an expression (must resolve to Val::Array).
    pub(super) fn extract_array_ptr(&mut self, arr: &Expr) -> inkwell::values::PointerValue<'ctx> {
        match arr {
            Expr::Ident(name) => {
                let var = self.vars.get(name).cloned()
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name));
                match var.kind {
                    VarKind::Array => var.ptr,
                    _ => panic!("'{}' is not an array", name),
                }
            }
            _ => panic!("Indexed object must be an array variable name"),
        }
    }
}
