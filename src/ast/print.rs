use crate::ast::argument::Name;
use crate::ast::expression::Expression;
use crate::ast::module::Module;
use crate::ast::statement::Statement;
use crate::ast::{argument, definition, statement};
use std::fmt;

use super::reassignment::ReassignmentTarget;

fn make_prefix(levels: &[bool]) -> String {
    if levels.is_empty() {
        return "→ ".to_string();
    }

    let mut prefix = String::new();

    for &has_next in &levels[..levels.len() - 1] {
        if has_next {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

    prefix.push_str(if levels[levels.len() - 1] {
        "├→ "
    } else {
        "└→ "
    });
    prefix
}

pub fn print_parse_tree(
    module: &Module<definition::Untyped>,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "{}Module:", make_prefix(&[]))?;

    for (i, definition) in module.definitions.iter().enumerate() {
        let has_next = i < module.definitions.len() - 1;
        print_definition(definition, &[has_next], f)?;
    }
    Ok(())
}

fn print_definition(
    definition: &definition::Untyped,
    levels: &[bool],
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match definition {
        definition::Untyped::Struct {
            location,
            name,
            fields,
        } => {
            writeln!(
                f,
                "{}Struct {} ({}..{})",
                make_prefix(levels),
                name,
                location.start,
                location.end
            )?;

            if let Some(fields) = fields {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                for (i, field) in fields.iter().enumerate() {
                    let mut field_levels = new_levels.clone();
                    field_levels.push(i < fields.len() - 1);
                    writeln!(
                        f,
                        "{}Field: {} - {:?}",
                        make_prefix(&field_levels),
                        field.name,
                        field.type_annotation
                    )?;
                }
            }
        }
        definition::Untyped::Function {
            location,
            name,
            arguments,
            body,
            return_type_annotation,
        } => {
            writeln!(
                f,
                "{}Function {} ({}..{})",
                make_prefix(levels),
                name,
                location.start,
                location.end
            )?;

            let mut items = Vec::new();
            if arguments.is_some() {
                items.push("arguments");
            }
            if return_type_annotation.is_some() {
                items.push("return_type");
            }
            if body.is_some() {
                items.push("body");
            }

            let mut new_levels = levels.to_vec();

            if let Some(args) = arguments {
                items.remove(0);
                let has_more = !items.is_empty();
                new_levels.push(has_more);
                writeln!(f, "{}Arguments:", make_prefix(&new_levels))?;

                for (i, arg) in args.iter().enumerate() {
                    let mut arg_levels = new_levels.clone();
                    arg_levels.push(i < args.len() - 1);
                    print_argument(arg, &arg_levels, f)?;
                }
                new_levels.pop();
            }

            if let Some(return_type) = return_type_annotation {
                let has_more = body.is_some();
                new_levels.push(has_more);
                writeln!(
                    f,
                    "{}Return type: {:?}",
                    make_prefix(&new_levels),
                    return_type
                )?;
                new_levels.pop();
            }

            if let Some(statements) = body {
                new_levels.push(false);
                writeln!(f, "{}Body:", make_prefix(&new_levels))?;

                for (i, stmt) in statements.iter().enumerate() {
                    let mut stmt_levels = new_levels.clone();
                    stmt_levels.push(i < statements.len() - 1);
                    print_statement(stmt, &stmt_levels, f)?;
                }
            }
        }
    }
    Ok(())
}

fn print_argument(
    arg: &argument::Untyped,
    levels: &[bool],
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match &arg.name {
        Name::Named { name, location } => {
            writeln!(
                f,
                "{}Argument {} ({}..{})",
                make_prefix(levels),
                name,
                location.start,
                location.end
            )?;

            if let Some(annotation) = &arg.annotation {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                writeln!(f, "{}Type: {:?}", make_prefix(&new_levels), annotation)?;
            }
        }
    }
    Ok(())
}

fn print_statement(
    stmt: &statement::Untyped,
    levels: &[bool],
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match stmt {
        Statement::Expression(expr) => {
            writeln!(f, "{}Expression:", make_prefix(levels))?;
            let mut new_levels = levels.to_vec();
            new_levels.push(false);
            print_expression(expr, &new_levels, f)?;
        }
        Statement::Assignment(assignment) => {
            writeln!(
                f,
                "{}Assignment ({}..{}):",
                make_prefix(levels),
                assignment.location.start,
                assignment.location.end
            )?;

            let mut new_levels = levels.to_vec();
            new_levels.push(true);
            writeln!(
                f,
                "{}Type: {:?}",
                make_prefix(&new_levels),
                assignment.type_annotation
            )?;
            writeln!(
                f,
                "{}Variable name: {}",
                make_prefix(&new_levels),
                assignment.variable_name
            )?;
            new_levels.pop();
            new_levels.push(false);
            writeln!(f, "{}Value:", make_prefix(&new_levels))?;

            let mut value_levels = new_levels.clone();
            value_levels.push(false);
            print_expression(&assignment.value, &value_levels, f)?;
        }
        Statement::Reassignment(reassignment) => {
            writeln!(
                f,
                "{}Reassignment ({}..{})",
                make_prefix(levels),
                reassignment.location.start,
                reassignment.location.end
            )?;
            match &reassignment.target {
                ReassignmentTarget::Variable { location, name } => {
                    writeln!(
                        f,
                        "{}Target: Variable {} ({}..{})",
                        make_prefix(&[levels, &[true]].concat()),
                        name,
                        location.start,
                        location.end
                    )?;
                }
                ReassignmentTarget::FieldAccess {
                    location,
                    struct_name,
                    field_name,
                } => {
                    writeln!(
                        f,
                        "{}Target: Field access {}.{} ({}..{})",
                        make_prefix(&[levels, &[true]].concat()),
                        struct_name,
                        field_name,
                        location.start,
                        location.end
                    )?;
                }
                ReassignmentTarget::ArrayAccess {
                    location,
                    array_name,
                    index_expression,
                } => {
                    writeln!(
                        f,
                        "{}Target: Array access {} ({}..{})",
                        make_prefix(&[levels, &[true]].concat()),
                        array_name,
                        location.start,
                        location.end
                    )?;
                    writeln!(f, "{}Index:", make_prefix(&[levels, &[true]].concat()))?;
                    print_expression(index_expression, &[levels, &[true, false]].concat(), f)?;
                }
            }
            writeln!(f, "{}Value:", make_prefix(&[levels, &[false]].concat()))?;
            print_expression(
                &reassignment.new_value,
                &[levels, &[false, false]].concat(),
                f,
            )?;
        }
        Statement::Loop { body, location } => {
            writeln!(
                f,
                "{}Loop ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
            if let Some(statements) = body {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                for (i, stmt) in statements.iter().enumerate() {
                    let mut stmt_levels = new_levels.clone();
                    stmt_levels.push(i < statements.len() - 1);
                    print_statement(stmt, &stmt_levels, f)?;
                }
            }
        }
        Statement::If {
            condition,
            if_body,
            else_body,
            location,
        } => {
            writeln!(
                f,
                "{}If ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;

            let mut new_levels = levels.to_vec();
            let has_more = if_body.is_some() || else_body.is_some();
            new_levels.push(has_more);
            writeln!(f, "{}Condition:", make_prefix(&new_levels))?;

            let mut cond_levels = new_levels.clone();
            cond_levels.push(false);
            print_expression(condition, &cond_levels, f)?;
            new_levels.pop();

            if let Some(if_statements) = if_body {
                let has_more = else_body.is_some();
                new_levels.push(has_more);
                writeln!(f, "{}If body:", make_prefix(&new_levels))?;

                for (i, stmt) in if_statements.iter().enumerate() {
                    let mut stmt_levels = new_levels.clone();
                    stmt_levels.push(i < if_statements.len() - 1);
                    print_statement(stmt, &stmt_levels, f)?;
                }
                new_levels.pop();
            }

            if let Some(else_statements) = else_body {
                new_levels.push(false);
                writeln!(f, "{}Else body:", make_prefix(&new_levels))?;

                for (i, stmt) in else_statements.iter().enumerate() {
                    let mut stmt_levels = new_levels.clone();
                    stmt_levels.push(i < else_statements.len() - 1);
                    print_statement(stmt, &stmt_levels, f)?;
                }
            }
        }
        Statement::Break { location } => {
            writeln!(
                f,
                "{}Break ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
        }
        Statement::Return { location, value } => {
            writeln!(
                f,
                "{}Return ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
            if let Some(expr) = value {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                print_expression(expr, &new_levels, f)?;
            }
        }
        Statement::Todo { location } => {
            writeln!(
                f,
                "{}Todo ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
        }
        Statement::Panic { location } => {
            writeln!(
                f,
                "{}Panic ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
        }
        Statement::Exit { location } => {
            writeln!(
                f,
                "{}Exit ({}..{})",
                make_prefix(levels),
                location.start,
                location.end
            )?;
        }
    }
    Ok(())
}

fn print_expression(expr: &Expression, levels: &[bool], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match expr {
        Expression::IntLiteral { location, value } => {
            writeln!(
                f,
                "{}Integer: {} ({}..{})",
                make_prefix(levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::FloatLiteral { location, value } => {
            writeln!(
                f,
                "{}Float: {} ({}..{})",
                make_prefix(levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::StringLiteral { location, value } => {
            writeln!(
                f,
                "{}String: {} ({}..{})",
                make_prefix(levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::CharLiteral { location, value } => {
            writeln!(
                f,
                "{}Char: {} ({}..{})",
                make_prefix(levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::VariableValue { location, name } => {
            writeln!(
                f,
                "{}Variable: {} ({}..{})",
                make_prefix(levels),
                name,
                location.start,
                location.end
            )?;
        }
        Expression::BinaryOperation {
            location,
            operator,
            left,
            right,
        } => {
            writeln!(
                f,
                "{}Binary operation: {:?} ({}..{})",
                make_prefix(levels),
                operator,
                location.start,
                location.end
            )?;

            let mut new_levels = levels.to_vec();

            new_levels.push(true);
            writeln!(f, "{}Left operand:", make_prefix(&new_levels))?;
            let mut left_levels = new_levels.clone();
            left_levels.push(false);
            print_expression(left, &left_levels, f)?;
            new_levels.pop();

            new_levels.push(false);
            writeln!(f, "{}Right operand:", make_prefix(&new_levels))?;
            let mut right_levels = new_levels.clone();
            right_levels.push(false);
            print_expression(right, &right_levels, f)?;
        }
        Expression::FunctionCall {
            location,
            function_name,
            arguments,
        } => {
            writeln!(
                f,
                "{}Function call: {} ({}..{})",
                make_prefix(levels),
                function_name,
                location.start,
                location.end
            )?;
            if let Some(args) = arguments {
                let mut new_levels = levels.to_vec();
                for (i, arg) in args.iter().enumerate() {
                    new_levels.push(i < args.len() - 1);
                    writeln!(
                        f,
                        "{}Argument ({}..{}):",
                        make_prefix(&new_levels),
                        arg.location.start,
                        arg.location.end
                    )?;

                    let mut arg_levels = new_levels.clone();
                    arg_levels.push(false);
                    print_expression(&arg.value, &arg_levels, f)?;
                    new_levels.pop();
                }
            }
        }
        Expression::StructFieldAccess {
            location,
            struct_name,
            field_name,
        } => {
            writeln!(
                f,
                "{}Struct field access: {}.{} ({}..{})",
                make_prefix(levels),
                struct_name,
                field_name,
                location.start,
                location.end
            )?;
        }
        Expression::ArrayElementAccess {
            location,
            array_name,
            index_expression,
        } => {
            writeln!(
                f,
                "{}Array element access: {} ({}..{})",
                make_prefix(levels),
                array_name,
                location.start,
                location.end
            )?;

            let mut new_levels = levels.to_vec();
            new_levels.push(false);
            writeln!(f, "{}Index:", make_prefix(&new_levels))?;

            let mut index_levels = new_levels.clone();
            index_levels.push(false);
            print_expression(index_expression, &index_levels, f)?;
        }
        Expression::ArrayInitialization {
            location,
            type_annotation,
            elements,
        } => {
            writeln!(
                f,
                "{}Array initialization of type {:?} ({}..{})",
                make_prefix(levels),
                type_annotation,
                location.start,
                location.end
            )?;

            if let Some(elems) = elements {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                writeln!(f, "{}Elements:", make_prefix(&new_levels))?;

                for (i, elem) in elems.iter().enumerate() {
                    let mut elem_levels = new_levels.clone();
                    elem_levels.push(i < elems.len() - 1);
                    print_expression(elem, &elem_levels, f)?;
                }
            }
        }
        Expression::StructInitialization {
            location,
            type_annotation,
            fields,
        } => {
            writeln!(
                f,
                "{}Struct initialization of type {:?} ({}..{})",
                make_prefix(levels),
                type_annotation,
                location.start,
                location.end
            )?;

            if let Some(field_values) = fields {
                let mut new_levels = levels.to_vec();
                new_levels.push(false);
                writeln!(f, "{}Fields:", make_prefix(&new_levels))?;

                for (i, field) in field_values.iter().enumerate() {
                    let mut field_levels = new_levels.clone();
                    field_levels.push(i < field_values.len() - 1);
                    writeln!(f, "{}Field {}: ", make_prefix(&field_levels), field.name)?;

                    let mut value_levels = field_levels.clone();
                    value_levels.push(false);
                    print_expression(&field.value, &value_levels, f)?;
                }
            }
        }
    }
    Ok(())
}
