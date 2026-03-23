use crate::parser::ast::{Expr, MatchPat, Stmt};
use super::{CodeGen, Val, Var, VarKind};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {

            // var name = expr;  (old syntax)
            Stmt::Let { name, expr } => {
                self.gen_let(name, expr);
            }

            // let name = expr;  / mut name = expr;  (new syntax)
            // Mutability flag is ignored in LLVM backend (everything is an alloca).
            Stmt::LetNew { name, expr, .. } => {
                self.gen_let(name, expr);
            }

            // type Name = Type;  — no code generated (type alias, for documentation only)
            Stmt::TypeAlias { .. } => {}

            // const NAME = expr;  (from Rust / C++)
            // Inside a function body, treated as var (value inlined if literal).
            Stmt::Const { name, expr } => {
                // Register as compile-time constant if it's a literal
                use crate::parser::ast::UnaryOp;
                let registered = match expr {
                    Expr::Number(n) => {
                        self.consts.insert(name.clone(), super::ConstVal::Int(*n));
                        true
                    }
                    Expr::Float(f) => {
                        self.consts.insert(name.clone(), super::ConstVal::Float(*f));
                        true
                    }
                    Expr::Unary(UnaryOp::Neg, inner) => match inner.as_ref() {
                        Expr::Number(n) => {
                            self.consts.insert(name.clone(), super::ConstVal::Int(-n));
                            true
                        }
                        Expr::Float(f) => {
                            self.consts.insert(name.clone(), super::ConstVal::Float(-f));
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                };
                // If not a pure literal, fall back to var-like allocation
                if !registered {
                    self.gen_let(name, expr);
                }
            }

            // name = expr;
            Stmt::Assign { name, expr } => {
                let var = self.vars.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Undefined variable '{}'", name));
                let val = self.gen_expr(expr);
                match var.kind {
                    VarKind::Float     => {
                        self.builder.build_store(var.ptr, self.as_float(val)).unwrap();
                    }
                    VarKind::Int       => {
                        self.builder.build_store(var.ptr, self.as_int(val)).unwrap();
                    }
                    VarKind::Struct(_) => {
                        panic!("Reassigning a struct via '=' is not supported");
                    }
                    VarKind::Array | VarKind::Tuple => {
                        panic!("Reassigning an array/tuple via '=' is not supported");
                    }
                    VarKind::FnPtr => {
                        self.builder.build_store(var.ptr, self.as_int(val)).unwrap();
                    }
                }
            }

            // obj.field = expr;
            Stmt::FieldAssign { obj, field, val } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let type_name = type_name.clone();
                    let field_info = self.struct_fields.get(&type_name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Unknown struct '{}'", type_name));
                    let idx = field_info.iter().position(|(n, _)| n == field)
                        .unwrap_or_else(|| panic!("Unknown field '{}' in '{}'", field, type_name));
                    let (_, is_float) = field_info[idx];
                    let st = *self.struct_types.get(&type_name)
                        .unwrap_or_else(|| panic!("Unknown struct type '{}'", type_name));
                    let gep = self.builder
                        .build_struct_gep(st, ptr, idx as u32, "fset")
                        .unwrap();
                    let v = self.gen_expr(val);
                    if is_float {
                        self.builder.build_store(gep, self.as_float(v)).unwrap();
                    } else {
                        self.builder.build_store(gep, self.as_int(v)).unwrap();
                    }
                } else {
                    panic!("Field assignment on a non-struct value");
                }
            }

            // arr[idx] = val;  (from Python / JS)
            Stmt::IndexAssign { arr, idx, val } => {
                let arr_ptr = self.extract_array_ptr(arr);
                let idx_v   = { let v = self.gen_expr(idx); self.as_int(v) };
                let val_v   = { let v = self.gen_expr(val); self.as_int(v) };
                let elem    = unsafe {
                    self.builder.build_gep(self.i64_ty, arr_ptr, &[idx_v], "idx.set").unwrap()
                };
                self.builder.build_store(elem, val_v).unwrap();
            }

            Stmt::Expr(e) => { self.gen_expr(e); }

            // println(expr);
            Stmt::Print(e) => {
                match e {
                    Expr::Str(s) => self.print_str(s),
                    // $"Hello, {name}!"  — interpolated string  (from C# / Kotlin)
                    Expr::Interpolated(parts) => self.print_interpolated(parts),
                    _ => match self.gen_expr(e) {
                        Val::Int(i)       => self.print_int(i),
                        Val::Float(f)     => self.print_float(f),
                        Val::Struct(_, n) => panic!("Cannot print struct '{}' directly", n),
                        Val::Array(_)     => panic!("Cannot print an array directly — use an index"),
                    }
                }
            }

            // return expr;  — emit deferred before returning  (Go defer semantics)
            Stmt::Return(e) => {
                let val = self.gen_expr(e);
                let ret = self.as_int(val);
                // Emit all deferred expressions before returning  (from Go)
                self.emit_deferred();
                self.builder.build_return(Some(&ret)).unwrap();
            }

            // { ... }
            Stmt::Block(stmts) => {
                let saved = self.vars.clone();
                for s in stmts {
                    if self.terminated() { break; }
                    self.gen_stmt(s);
                }
                self.vars.retain(|k, _| saved.contains_key(k));
                for (k, v) in &saved {
                    self.vars.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }

            // if (cond) { then } [else { els }]
            Stmt::If { cond, then, els } => {
                let cond_i1  = self.bool_cond(cond);
                let func     = self.cur_fn();
                let then_bb  = self.ctx.append_basic_block(func, "if.then");
                let else_bb  = self.ctx.append_basic_block(func, "if.else");
                let merge_bb = self.ctx.append_basic_block(func, "if.merge");

                self.builder.build_conditional_branch(cond_i1, then_bb, else_bb).unwrap();

                self.builder.position_at_end(then_bb);
                self.gen_stmt(then);
                if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                self.builder.position_at_end(else_bb);
                if let Some(e) = els { self.gen_stmt(e); }
                if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                self.builder.position_at_end(merge_bb);
            }

            // while (cond) { body }
            Stmt::While { cond, body } => {
                let func    = self.cur_fn();
                let hdr_bb  = self.ctx.append_basic_block(func, "while.hdr");
                let body_bb = self.ctx.append_basic_block(func, "while.body");
                let exit_bb = self.ctx.append_basic_block(func, "while.exit");

                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(hdr_bb);
                let cond_i1 = self.bool_cond(cond);
                self.builder.build_conditional_branch(cond_i1, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((exit_bb, hdr_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(hdr_bb).unwrap(); }

                self.builder.position_at_end(exit_bb);
            }

            // do { body } while (cond);
            Stmt::DoWhile { body, cond } => {
                let func    = self.cur_fn();
                let body_bb = self.ctx.append_basic_block(func, "dowhile.body");
                let cond_bb = self.ctx.append_basic_block(func, "dowhile.cond");
                let exit_bb = self.ctx.append_basic_block(func, "dowhile.exit");

                self.builder.build_unconditional_branch(body_bb).unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((exit_bb, cond_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(cond_bb).unwrap(); }

                self.builder.position_at_end(cond_bb);
                let cond_i1 = self.bool_cond(cond);
                self.builder.build_conditional_branch(cond_i1, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(exit_bb);
            }

            // for i in from..to / from..=to { body }
            Stmt::For { var, from, to, inclusive, body } => {
                let start = { let v = self.gen_expr(from); self.as_int(v) };
                let end   = { let v = self.gen_expr(to);   self.as_int(v) };

                let func     = self.cur_fn();
                let pre_bb   = self.builder.get_insert_block().unwrap();
                let hdr_bb   = self.ctx.append_basic_block(func, "for.hdr");
                let body_bb  = self.ctx.append_basic_block(func, "for.body");
                let step_bb  = self.ctx.append_basic_block(func, "for.step");
                let exit_bb  = self.ctx.append_basic_block(func, "for.exit");

                let loop_alloca = self.builder.build_alloca(self.i64_ty, var).unwrap();

                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(hdr_bb);
                let phi   = self.builder.build_phi(self.i64_ty, "for.i").unwrap();
                phi.add_incoming(&[(&start, pre_bb)]);
                let phi_v = phi.as_basic_value().into_int_value();

                self.builder.build_store(loop_alloca, phi_v).unwrap();

                let pred = if *inclusive {
                    inkwell::IntPredicate::SLE
                } else {
                    inkwell::IntPredicate::SLT
                };
                let cond_b = self.builder.build_int_compare(
                    pred, phi_v, end, "for.cond",
                ).unwrap();
                self.builder.build_conditional_branch(cond_b, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(body_bb);
                let prev = self.vars.insert(
                    var.clone(),
                    Var { ptr: loop_alloca, kind: VarKind::Int },
                );
                self.loop_stack.push((exit_bb, step_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(step_bb).unwrap(); }
                match prev {
                    Some(v) => { self.vars.insert(var.clone(), v); }
                    None    => { self.vars.remove(var); }
                }

                self.builder.position_at_end(step_bb);
                let inc = self.builder
                    .build_int_add(phi_v, self.i64_ty.const_int(1, false), "for.inc")
                    .unwrap();
                phi.add_incoming(&[(&inc, step_bb)]);
                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(exit_bb);
            }

            // loop { body }
            Stmt::Loop { body } => {
                let func    = self.cur_fn();
                let loop_bb = self.ctx.append_basic_block(func, "loop");
                let exit_bb = self.ctx.append_basic_block(func, "loop.exit");

                self.builder.build_unconditional_branch(loop_bb).unwrap();
                self.builder.position_at_end(loop_bb);

                self.loop_stack.push((exit_bb, loop_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();

                if !self.terminated() { self.builder.build_unconditional_branch(loop_bb).unwrap(); }
                self.builder.position_at_end(exit_bb);
            }

            // break;
            Stmt::Break => {
                let (exit_bb, _) = *self.loop_stack.last()
                    .expect("'break' outside of a loop");
                self.builder.build_unconditional_branch(exit_bb).unwrap();
            }

            // continue;
            Stmt::Continue => {
                let (_, cont_bb) = *self.loop_stack.last()
                    .expect("'continue' outside of a loop");
                self.builder.build_unconditional_branch(cont_bb).unwrap();
            }

            // defer stmt;  (from Go) — register for execution at function exit
            Stmt::Defer(s) => {
                self.deferred.push(*s.clone());
            }

            // enum Name { Variant, ... }  — already handled in pass 0, skip
            Stmt::EnumDecl { .. } => {}

            // import "module"  — resolved before codegen, skip
            Stmt::Import { .. } => {}

            // extern func ...  — declared in pass 1, no body to generate
            Stmt::ExternFn { .. } => {}

            // match expr { pat => { body }, ... }
            Stmt::Match { expr, arms } => {
                let val   = self.gen_expr(expr);
                let v     = self.as_int(val);
                let func  = self.cur_fn();
                let merge_bb = self.ctx.append_basic_block(func, "match.end");

                let arm_bbs: Vec<_> = (0..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("match.arm.{}", i)))
                    .collect();
                let check_bbs: Vec<_> = (1..arms.len())
                    .map(|i| self.ctx.append_basic_block(func, &format!("match.chk.{}", i)))
                    .collect();

                let first_arm = arm_bbs[0];
                if let Some(arm) = arms.first() {
                    let pat_val = self.resolve_match_pat(&arm.pat);
                    match pat_val {
                        Some(pat_v) => {
                            let cmp = self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ, v, pat_v, "mc",
                            ).unwrap();
                            let next = if check_bbs.is_empty() { merge_bb } else { check_bbs[0] };
                            self.builder.build_conditional_branch(cmp, first_arm, next).unwrap();
                        }
                        None => {
                            self.builder.build_unconditional_branch(first_arm).unwrap();
                        }
                    }
                }

                for (i, arm) in arms.iter().enumerate() {
                    self.builder.position_at_end(arm_bbs[i]);
                    for s in &arm.body {
                        if self.terminated() { break; }
                        self.gen_stmt(s);
                    }
                    if !self.terminated() { self.builder.build_unconditional_branch(merge_bb).unwrap(); }

                    if i + 1 < arms.len() {
                        self.builder.position_at_end(check_bbs[i]);
                        let next_arm = arm_bbs[i + 1];
                        let fall = if i + 1 < check_bbs.len() { check_bbs[i + 1] } else { merge_bb };
                        let pat_val = self.resolve_match_pat(&arms[i + 1].pat);
                        match pat_val {
                            Some(pat_v) => {
                                let cmp = self.builder.build_int_compare(
                                    inkwell::IntPredicate::EQ, v, pat_v, "mc",
                                ).unwrap();
                                self.builder.build_conditional_branch(cmp, next_arm, fall).unwrap();
                            }
                            None => {
                                self.builder.build_unconditional_branch(next_arm).unwrap();
                            }
                        }
                    }
                }

                self.builder.position_at_end(merge_bb);
            }

            // for x in array { body }  — array iteration  (from Python)
            Stmt::ForIn { var, iter, body } => {
                let arr_name = match iter {
                    Expr::Ident(n) => n.clone(),
                    _ => panic!("for-in: iterator must be an array variable name"),
                };
                let len = *self.array_lens.get(&arr_name)
                    .unwrap_or_else(|| panic!(
                        "for-in: unknown array '{}' or array length not tracked", arr_name
                    ));
                let arr_ptr = self.extract_array_ptr(iter);

                let func    = self.cur_fn();
                let zero    = self.i64_ty.const_int(0, false);
                let end     = self.i64_ty.const_int(len as u64, false);
                let pre_bb  = self.builder.get_insert_block().unwrap();
                let hdr_bb  = self.ctx.append_basic_block(func, "forin.hdr");
                let body_bb = self.ctx.append_basic_block(func, "forin.body");
                let step_bb = self.ctx.append_basic_block(func, "forin.step");
                let exit_bb = self.ctx.append_basic_block(func, "forin.exit");

                let var_alloca = self.builder.build_alloca(self.i64_ty, var).unwrap();
                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(hdr_bb);
                let phi = self.builder.build_phi(self.i64_ty, "forin.i").unwrap();
                phi.add_incoming(&[(&zero, pre_bb)]);
                let phi_v = phi.as_basic_value().into_int_value();
                let cond_b = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT, phi_v, end, "forin.cond",
                ).unwrap();
                self.builder.build_conditional_branch(cond_b, body_bb, exit_bb).unwrap();

                self.builder.position_at_end(body_bb);
                let elem_ptr = unsafe {
                    self.builder.build_gep(self.i64_ty, arr_ptr, &[phi_v], "forin.elem").unwrap()
                };
                let elem_val = self.builder
                    .build_load(self.i64_ty, elem_ptr, "forin.val")
                    .unwrap()
                    .into_int_value();
                self.builder.build_store(var_alloca, elem_val).unwrap();

                let prev = self.vars.insert(var.clone(), Var { ptr: var_alloca, kind: VarKind::Int });
                self.loop_stack.push((exit_bb, step_bb));
                self.gen_stmt(body);
                self.loop_stack.pop();
                if !self.terminated() { self.builder.build_unconditional_branch(step_bb).unwrap(); }
                match prev {
                    Some(v) => { self.vars.insert(var.clone(), v); }
                    None    => { self.vars.remove(var); }
                }

                self.builder.position_at_end(step_bb);
                let inc = self.builder
                    .build_int_add(phi_v, self.i64_ty.const_int(1, false), "forin.inc")
                    .unwrap();
                phi.add_incoming(&[(&inc, step_bb)]);
                self.builder.build_unconditional_branch(hdr_bb).unwrap();

                self.builder.position_at_end(exit_bb);
            }

            // var (a, b) = expr;  — tuple destructuring  (from Python / Rust)
            Stmt::LetTuple { names, expr } => {
                let val = self.gen_expr(expr);
                let tuple_ptr = match val {
                    Val::Array(ptr) => ptr,
                    _ => panic!("LetTuple: right-hand side must evaluate to a tuple"),
                };
                for (i, name) in names.iter().enumerate() {
                    let idx = self.i64_ty.const_int(i as u64, false);
                    let elem_ptr = unsafe {
                        self.builder.build_gep(self.i64_ty, tuple_ptr, &[idx], "tup.unpack").unwrap()
                    };
                    let v = self.builder
                        .build_load(self.i64_ty, elem_ptr, name)
                        .unwrap()
                        .into_int_value();
                    let alloca = self.builder.build_alloca(self.i64_ty, name).unwrap();
                    self.builder.build_store(alloca, v).unwrap();
                    self.vars.insert(name.clone(), Var { ptr: alloca, kind: VarKind::Int });
                }
            }

            // @annotation  — no code generated, metadata only
            Stmt::Annotation { .. } => {}

            // trait / impl Trait for Type  — handled in pass 0/1/2 at top level
            Stmt::TraitDecl { .. } | Stmt::ImplTrait { .. } => {
                panic!("trait / impl Trait must be at the top level");
            }

            Stmt::FnDecl { .. } => {
                panic!("Nested function declarations are not supported");
            }
            Stmt::StructDecl { .. } | Stmt::ImplDecl { .. } | Stmt::ClassDecl { .. } => {
                panic!("struct/impl/class must be at the top level");
            }
        }
    }

    // ── Helper: resolve a match pattern to an LLVM integer constant ───────────

    /// Returns `Some(int_val)` for patterns that compare against a constant,
    /// or `None` for wildcard patterns.
    pub(super) fn resolve_match_pat(
        &self,
        pat: &MatchPat,
    ) -> Option<inkwell::values::IntValue<'ctx>> {
        match pat {
            MatchPat::Int(n) =>
                Some(self.i64_ty.const_int(*n as u64, true)),
            MatchPat::Wildcard =>
                None,
            // EnumName.Variant  (from Rust / Swift)
            MatchPat::EnumVariant(enum_name, variant) => {
                let val = self.enums
                    .get(enum_name)
                    .and_then(|m| m.get(variant))
                    .copied()
                    .unwrap_or_else(|| panic!(
                        "Unknown enum variant '{}.{}'", enum_name, variant
                    ));
                Some(self.i64_ty.const_int(val as u64, true))
            }
        }
    }

    // ── Helper: allocate and store a `var` / `const` ─────────────────────────

    fn gen_let(&mut self, name: &str, expr: &Expr) {
        // Track array length so ForIn can use it
        if let Expr::ArrayLit(elems) = expr {
            self.array_lens.insert(name.to_string(), elems.len() as i64);
        }
        let val = self.gen_expr(expr);
        match val {
            Val::Int(i) => {
                let p = self.builder.build_alloca(self.i64_ty, name).unwrap();
                self.builder.build_store(p, i).unwrap();
                self.vars.insert(name.to_string(), Var { ptr: p, kind: VarKind::Int });
            }
            Val::Float(f) => {
                let p = self.builder.build_alloca(self.f64_ty, name).unwrap();
                self.builder.build_store(p, f).unwrap();
                self.vars.insert(name.to_string(), Var { ptr: p, kind: VarKind::Float });
            }
            Val::Struct(ptr, type_name) => {
                self.vars.insert(name.to_string(), Var { ptr, kind: VarKind::Struct(type_name) });
            }
            Val::Array(ptr) => {
                self.vars.insert(name.to_string(), Var { ptr, kind: VarKind::Array });
            }
        }
    }
}
