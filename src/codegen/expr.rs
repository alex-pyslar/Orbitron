use inkwell::values::{BasicMetadataValueEnum, FloatValue, IntValue};

use crate::parser::ast::{BinOp, Expr, UnaryOp};
use super::{CodeGen, Val, VarKind};

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Val<'ctx> {
        match expr {
            Expr::Number(n)  => Val::Int(self.i64_ty.const_int(*n as u64, true)),
            Expr::Float(f)   => Val::Float(self.f64_ty.const_float(*f)),
            Expr::Str(_)     => panic!("Строковые литералы разрешены только внутри println()"),

            Expr::Ident(name) => {
                let var = self.vars.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Неопределённая переменная '{}'", name));
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
                }
            }

            // obj.field
            Expr::FieldAccess { obj, field } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let type_name = type_name.clone();
                    let field_info = self.struct_fields.get(&type_name)
                        .cloned()
                        .unwrap_or_else(|| panic!("Неизвестная структура '{}'", type_name));
                    let idx = field_info.iter().position(|(n, _)| n == field)
                        .unwrap_or_else(|| panic!("Неизвестное поле '{}' в '{}'", field, type_name));
                    let (_, is_float) = field_info[idx];
                    let st  = *self.struct_types.get(&type_name)
                        .unwrap_or_else(|| panic!("Неизвестный тип структуры '{}'", type_name));
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
                    panic!("Обращение к полю на не-структурном значении");
                }
            }

            // obj.method(args)
            Expr::MethodCall { obj, method, args } => {
                let obj_val = self.gen_expr(obj);
                if let Val::Struct(ptr, ref type_name) = obj_val {
                    let func_name = format!("{}_{}", type_name, method);
                    let callee = self.module.get_function(&func_name)
                        .unwrap_or_else(|| panic!("Неизвестный метод '{}'", func_name));
                    let mut argv: Vec<BasicMetadataValueEnum> = vec![ptr.into()];
                    argv.extend(args.iter().map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    }));
                    let result = self.builder
                        .build_call(callee, &argv, "mcall")
                        .unwrap()
                        .try_as_basic_value()
                        .expect_basic("метод должен возвращать значение")
                        .into_int_value();
                    Val::Int(result)
                } else {
                    panic!("Вызов метода на не-структурном значении");
                }
            }

            // StructName { field: expr, ... }  — struct literal (no `new`)
            Expr::StructLit { name, fields } => {
                let field_info = self.struct_fields.get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Неизвестная структура '{}'", name));
                let st = *self.struct_types.get(name)
                    .unwrap_or_else(|| panic!("Неизвестный тип структуры '{}'", name));
                let alloca = self.builder
                    .build_alloca(st, &format!("{}.new", name))
                    .unwrap();
                for (fname, fexpr) in fields {
                    let idx = field_info.iter().position(|(n, _)| n == fname)
                        .unwrap_or_else(|| panic!("Неизвестное поле '{}' в '{}'", fname, name));
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
                    .unwrap_or_else(|| panic!("Неизвестный класс '{}' в конструкторе", class));
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
                        "Класс '{}' не имеет конструктора 'init', но вызван с {} аргументом(ами). \
                         Определите 'init(...)' внутри класса.",
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

            Expr::Unary(op, inner) => {
                let v = self.gen_expr(inner);
                match op {
                    UnaryOp::Neg => match v {
                        Val::Int(i)       => Val::Int(self.builder.build_int_neg(i, "neg").unwrap()),
                        Val::Float(f)     => Val::Float(self.builder.build_float_neg(f, "fneg").unwrap()),
                        Val::Struct(_, n) => panic!("Нельзя применить отрицание к struct '{}'", n),
                    },
                    UnaryOp::Not => {
                        let i   = self.as_int(v);
                        let cmp = self.builder.build_int_compare(
                            inkwell::IntPredicate::EQ, i, self.i64_ty.const_zero(), "not.cmp",
                        ).unwrap();
                        Val::Int(self.builder.build_int_z_extend(cmp, self.i64_ty, "not.ext").unwrap())
                    }
                }
            }

            Expr::Binary(lhs, op, rhs) => {
                let l = self.gen_expr(lhs);
                let r = self.gen_expr(rhs);
                self.gen_binop(l, r, op)
            }

            Expr::Call { name, args } => {
                let callee = self.module.get_function(name)
                    .unwrap_or_else(|| panic!("Неопределённая функция '{}'", name));
                let argv: Vec<BasicMetadataValueEnum> = args.iter()
                    .map(|a| {
                        let v = self.gen_expr(a);
                        BasicMetadataValueEnum::IntValue(self.as_int(v))
                    })
                    .collect();
                let result = self.builder
                    .build_call(callee, &argv, "call")
                    .unwrap()
                    .try_as_basic_value()
                    .expect_basic("функция должна возвращать значение")
                    .into_int_value();
                Val::Int(result)
            }
        }
    }

    pub(super) fn gen_binop(&mut self, l: Val<'ctx>, r: Val<'ctx>, op: &BinOp) -> Val<'ctx> {
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
}
