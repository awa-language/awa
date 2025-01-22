use crate::ast::argument::Name;
use crate::ast::expression::Expression;
use crate::ast::module::Module;
use crate::ast::statement::Statement;
use crate::ast::{argument, definition, statement};
use std::fmt;

use super::reassignment::ReassignmentTarget;

/// Prints structure of untyped AST module
///
/// # Errors
///
/// This function will return `fmt::Error` if it cannot perform writes.
pub fn print_parse_tree(
    module: &Module<definition::Untyped>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(formatter, "{}Module:", make_prefix(&[]))?;

    if let Some(definitions) = &module.definitions {
        let total_len = definitions.len();
        for (i, definition) in definitions.iter().enumerate() {
            let has_next = i < total_len - 1;
            print_definition(definition, &[has_next], formatter)?;
        }
    }

    Ok(())
}

fn make_prefix(indentation_levels: &[bool]) -> String {
    if indentation_levels.is_empty() {
        return "→ ".to_string();
    }

    let mut prefix = String::new();

    for &has_next in &indentation_levels[..indentation_levels.len() - 1] {
        if has_next {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

    prefix.push_str(if indentation_levels[indentation_levels.len() - 1] {
        "├→ "
    } else {
        "└→ "
    });

    prefix
}

fn print_definition(
    definition: &definition::Untyped,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match definition {
        definition::Untyped::Struct {
            location,
            name,
            fields,
        } => {
            writeln!(
                formatter,
                "{}Struct {} ({}..{})",
                make_prefix(indentation_levels),
                name,
                location.start,
                location.end
            )?;

            if let Some(fields) = fields {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);

                for (i, field) in fields.iter().enumerate() {
                    let mut field_levels = new_indentation_levels.clone();
                    field_levels.push(i < fields.len() - 1);
                    writeln!(
                        formatter,
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
                formatter,
                "{}Function {} ({}..{})",
                make_prefix(indentation_levels),
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

            let mut new_indentation_levels = indentation_levels.to_vec();

            if let Some(args) = arguments {
                items.remove(0);
                let has_more = !items.is_empty();
                new_indentation_levels.push(has_more);
                writeln!(
                    formatter,
                    "{}Arguments:",
                    make_prefix(&new_indentation_levels)
                )?;

                for (i, arg) in args.iter().enumerate() {
                    let mut arg_levels = new_indentation_levels.clone();
                    arg_levels.push(i < args.len() - 1);
                    print_argument(arg, &arg_levels, formatter)?;
                }

                new_indentation_levels.pop();
            }

            if let Some(return_type) = return_type_annotation {
                let has_more = body.is_some();
                new_indentation_levels.push(has_more);
                writeln!(
                    formatter,
                    "{}Return type: {:?}",
                    make_prefix(&new_indentation_levels),
                    return_type
                )?;
                new_indentation_levels.pop();
            }

            if let Some(statements) = body {
                new_indentation_levels.push(false);
                writeln!(formatter, "{}Body:", make_prefix(&new_indentation_levels))?;

                for (i, statement) in statements.iter().enumerate() {
                    let mut statement_levels = new_indentation_levels.clone();
                    statement_levels.push(i < statements.len() - 1);
                    print_statement(statement, &statement_levels, formatter)?;
                }
            }
        }
    }

    Ok(())
}

fn print_argument(
    arg: &argument::Untyped,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match &arg.name {
        Name::Named { name, location } => {
            writeln!(
                formatter,
                "{}Argument {} ({}..{})",
                make_prefix(indentation_levels),
                name,
                location.start,
                location.end
            )?;

            if let Some(annotation) = &arg.annotation {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);
                writeln!(
                    formatter,
                    "{}Type: {:?}",
                    make_prefix(&new_indentation_levels),
                    annotation
                )?;
            }
        }
    }

    Ok(())
}

fn print_statement(
    statement: &statement::Untyped,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match statement {
        Statement::Expression(expression) => {
            writeln!(formatter, "{}Expression:", make_prefix(indentation_levels))?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(false);

            print_expression(expression, &new_indentation_levels, formatter)?;
        }
        Statement::Assignment(assignment) => {
            writeln!(
                formatter,
                "{}Assignment ({}..{}):",
                make_prefix(indentation_levels),
                assignment.location.start,
                assignment.location.end
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(true);

            writeln!(
                formatter,
                "{}Type: {:?}",
                make_prefix(&new_indentation_levels),
                assignment.type_annotation
            )?;
            writeln!(
                formatter,
                "{}Variable name: {}",
                make_prefix(&new_indentation_levels),
                assignment.variable_name
            )?;

            new_indentation_levels.pop();
            new_indentation_levels.push(false);
            writeln!(formatter, "{}Value:", make_prefix(&new_indentation_levels))?;

            let mut value_levels = new_indentation_levels.clone();
            value_levels.push(false);

            print_expression(&assignment.value, &value_levels, formatter)?;
        }
        Statement::Reassignment(reassignment) => {
            writeln!(
                formatter,
                "{}Reassignment ({}..{})",
                make_prefix(indentation_levels),
                reassignment.location.start,
                reassignment.location.end
            )?;

            match &reassignment.target {
                ReassignmentTarget::Variable { location, name } => {
                    writeln!(
                        formatter,
                        "{}Target: Variable {} ({}..{})",
                        make_prefix(&[indentation_levels, &[true]].concat()),
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
                        formatter,
                        "{}Target: Field access {}.{} ({}..{})",
                        make_prefix(&[indentation_levels, &[true]].concat()),
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
                        formatter,
                        "{}Target: Array access {} ({}..{})",
                        make_prefix(&[indentation_levels, &[true]].concat()),
                        array_name,
                        location.start,
                        location.end
                    )?;
                    writeln!(
                        formatter,
                        "{}Index:",
                        make_prefix(&[indentation_levels, &[true]].concat())
                    )?;
                    print_expression(
                        index_expression,
                        &[indentation_levels, &[true, false]].concat(),
                        formatter,
                    )?;
                }
            }

            writeln!(
                formatter,
                "{}Value:",
                make_prefix(&[indentation_levels, &[false]].concat())
            )?;

            print_expression(
                &reassignment.new_value,
                &[indentation_levels, &[false, false]].concat(),
                formatter,
            )?;
        }
        Statement::Loop { body, location } => {
            writeln!(
                formatter,
                "{}Loop ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;

            if let Some(statements) = body {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);

                for (i, statement) in statements.iter().enumerate() {
                    let mut statement_levels = new_indentation_levels.clone();
                    statement_levels.push(i < statements.len() - 1);
                    print_statement(statement, &statement_levels, formatter)?;
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
                formatter,
                "{}If ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            let has_more = if_body.is_some() || else_body.is_some();
            new_indentation_levels.push(has_more);

            writeln!(
                formatter,
                "{}Condition:",
                make_prefix(&new_indentation_levels)
            )?;

            let mut condition_levels = new_indentation_levels.clone();
            condition_levels.push(false);

            print_expression(condition, &condition_levels, formatter)?;
            new_indentation_levels.pop();

            if let Some(if_statements) = if_body {
                let has_more = else_body.is_some();
                new_indentation_levels.push(has_more);

                writeln!(
                    formatter,
                    "{}If body:",
                    make_prefix(&new_indentation_levels)
                )?;

                for (i, statement) in if_statements.iter().enumerate() {
                    let mut statement_levels = new_indentation_levels.clone();
                    statement_levels.push(i < if_statements.len() - 1);

                    print_statement(statement, &statement_levels, formatter)?;
                }

                new_indentation_levels.pop();
            }

            if let Some(else_statements) = else_body {
                new_indentation_levels.push(false);
                writeln!(
                    formatter,
                    "{}Else body:",
                    make_prefix(&new_indentation_levels)
                )?;

                for (i, statement) in else_statements.iter().enumerate() {
                    let mut statement_levels = new_indentation_levels.clone();
                    statement_levels.push(i < else_statements.len() - 1);

                    print_statement(statement, &statement_levels, formatter)?;
                }
            }
        }
        Statement::Break { location } => {
            writeln!(
                formatter,
                "{}Break ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        Statement::Return { location, value } => {
            writeln!(
                formatter,
                "{}Return ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;

            if let Some(expression) = value {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);
                print_expression(expression, &new_indentation_levels, formatter)?;
            }
        }
        Statement::Todo { location } => {
            writeln!(
                formatter,
                "{}Todo ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        Statement::Panic { location } => {
            writeln!(
                formatter,
                "{}Panic ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        Statement::Exit { location } => {
            writeln!(
                formatter,
                "{}Exit ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
    }

    Ok(())
}

fn print_expression(
    expr: &Expression,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match expr {
        Expression::IntLiteral { location, value } => {
            writeln!(
                formatter,
                "{}Integer: {} ({}..{})",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::FloatLiteral { location, value } => {
            writeln!(
                formatter,
                "{}Float: {} ({}..{})",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::StringLiteral { location, value } => {
            writeln!(
                formatter,
                "{}String: {} ({}..{})",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::CharLiteral { location, value } => {
            writeln!(
                formatter,
                "{}Char: {} ({}..{})",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end
            )?;
        }
        Expression::VariableValue { location, name } => {
            writeln!(
                formatter,
                "{}Variable: {} ({}..{})",
                make_prefix(indentation_levels),
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
                formatter,
                "{}Binary operation: {:?} ({}..{})",
                make_prefix(indentation_levels),
                operator,
                location.start,
                location.end
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();

            new_indentation_levels.push(true);
            writeln!(
                formatter,
                "{}Left operand:",
                make_prefix(&new_indentation_levels)
            )?;

            let mut left_levels = new_indentation_levels.clone();
            left_levels.push(false);

            print_expression(left, &left_levels, formatter)?;
            new_indentation_levels.pop();

            new_indentation_levels.push(false);
            writeln!(
                formatter,
                "{}Right operand:",
                make_prefix(&new_indentation_levels)
            )?;

            let mut right_levels = new_indentation_levels.clone();
            right_levels.push(false);

            print_expression(right, &right_levels, formatter)?;
        }
        Expression::FunctionCall {
            location,
            function_name,
            arguments,
        } => {
            writeln!(
                formatter,
                "{}Function call: {} ({}..{})",
                make_prefix(indentation_levels),
                function_name,
                location.start,
                location.end
            )?;
            if let Some(arguments) = arguments {
                let mut new_indentation_levels = indentation_levels.to_vec();

                for (i, argument) in arguments.iter().enumerate() {
                    new_indentation_levels.push(i < arguments.len() - 1);
                    writeln!(
                        formatter,
                        "{}Argument ({}..{}):",
                        make_prefix(&new_indentation_levels),
                        argument.location.start,
                        argument.location.end
                    )?;

                    let mut argument_levels = new_indentation_levels.clone();
                    argument_levels.push(false);

                    print_expression(&argument.value, &argument_levels, formatter)?;
                    new_indentation_levels.pop();
                }
            }
        }
        Expression::StructFieldAccess {
            location,
            struct_name,
            field_name,
        } => {
            writeln!(
                formatter,
                "{}Struct field access: {}.{} ({}..{})",
                make_prefix(indentation_levels),
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
                formatter,
                "{}Array element access: {} ({}..{})",
                make_prefix(indentation_levels),
                array_name,
                location.start,
                location.end
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(false);
            writeln!(formatter, "{}Index:", make_prefix(&new_indentation_levels))?;

            let mut index_levels = new_indentation_levels.clone();
            index_levels.push(false);
            print_expression(index_expression, &index_levels, formatter)?;
        }
        Expression::ArrayInitialization {
            location,
            type_annotation,
            elements,
        } => {
            writeln!(
                formatter,
                "{}Array initialization of type {:?} ({}..{})",
                make_prefix(indentation_levels),
                type_annotation,
                location.start,
                location.end
            )?;

            if let Some(elements) = elements {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);

                writeln!(
                    formatter,
                    "{}Elements:",
                    make_prefix(&new_indentation_levels)
                )?;

                for (i, element) in elements.iter().enumerate() {
                    let mut element_levels = new_indentation_levels.clone();
                    element_levels.push(i < elements.len() - 1);

                    print_expression(element, &element_levels, formatter)?;
                }
            }
        }
        Expression::StructInitialization {
            location,
            type_annotation,
            fields,
        } => {
            writeln!(
                formatter,
                "{}Struct initialization of type {:?} ({}..{})",
                make_prefix(indentation_levels),
                type_annotation,
                location.start,
                location.end
            )?;

            if let Some(field_values) = fields {
                let mut new_indentation_levels = indentation_levels.to_vec();
                new_indentation_levels.push(false);
                writeln!(formatter, "{}Fields:", make_prefix(&new_indentation_levels))?;

                for (i, field) in field_values.iter().enumerate() {
                    let mut field_levels = new_indentation_levels.clone();
                    field_levels.push(i < field_values.len() - 1);

                    writeln!(
                        formatter,
                        "{}Field {}: ",
                        make_prefix(&field_levels),
                        field.name
                    )?;

                    let mut value_levels = field_levels.clone();
                    value_levels.push(false);

                    print_expression(&field.value, &value_levels, formatter)?;
                }
            }
        }
    }

    Ok(())
}
