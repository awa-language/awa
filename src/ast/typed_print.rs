use crate::ast::expression::TypedExpression;
use crate::ast::module::Module;
use crate::ast::reassignment::TypedReassignmentTarget;
use crate::ast::statement::TypedStatement;
use crate::ast::{argument, definition, statement};
use crate::type_::Type;
use std::fmt;

/// Prints structure of typed AST module
///
/// # Errors
///
/// This function will return `fmt::Error` if it cannot perform writes.
pub fn print_typed(
    module: &Module<definition::DefinitionTyped>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(formatter, "{}Module:", make_prefix(&[]))?;

    if let Some(definitions) = &module.definitions {
        let total_len = definitions.len();
        for (i, definition) in definitions.iter().enumerate() {
            let has_next = i < total_len - 1;
            crate::ast::typed_print::print_definition(definition, &[has_next], formatter)?;
        }
    }

    Ok(())
}

impl fmt::Display for Type {
    fn fmt(&self, formater: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formater, "{self:?}") // Customize this to fit your needs
    }
}

fn print_expression(
    expr: &TypedExpression,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match expr {
        TypedExpression::IntLiteral {
            location,
            value,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Integer: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::FloatLiteral {
            location,
            value,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Float: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::StringLiteral {
            location,
            value,
            type_,
        } => {
            writeln!(
                formatter,
                "{}String: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::CharLiteral {
            location,
            value,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Char: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                value,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::VariableValue {
            location,
            name,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Variable: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                name,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::BinaryOperation {
            location,
            operator,
            left,
            right,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Binary operation: {:?} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                operator,
                location.start,
                location.end,
                type_
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
        TypedExpression::FunctionCall {
            location,
            function_name,
            arguments,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Function call: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                function_name,
                location.start,
                location.end,
                type_
            )?;
            if let Some(arguments) = arguments {
                let mut new_indentation_levels = indentation_levels.to_vec();

                for (i, argument) in arguments.iter().enumerate() {
                    new_indentation_levels.push(i < arguments.len() - 1);
                    writeln!(
                        formatter,
                        "{}Argument ({}..{}): type: {}",
                        make_prefix(&new_indentation_levels),
                        argument.location.start,
                        argument.location.end,
                        argument.type_,
                    )?;

                    let mut argument_levels = new_indentation_levels.clone();
                    argument_levels.push(false);

                    print_expression(&argument.value, &argument_levels, formatter)?;
                    new_indentation_levels.pop();
                }
            }
        }
        TypedExpression::StructFieldAccess {
            location,
            struct_name,
            field_name,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Struct field access: {}.{} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                struct_name,
                field_name,
                location.start,
                location.end,
                type_
            )?;
        }
        TypedExpression::ArrayElementAccess {
            location,
            array_name,
            index_expression,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Array element access: {} ({}..{}) type: {}",
                make_prefix(indentation_levels),
                array_name,
                location.start,
                location.end,
                type_
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(false);
            writeln!(formatter, "{}Index:", make_prefix(&new_indentation_levels))?;

            let mut index_levels = new_indentation_levels.clone();
            index_levels.push(false);
            print_expression(index_expression, &index_levels, formatter)?;
        }
        TypedExpression::ArrayInitialization {
            location,
            elements,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Array initialization of type {:?} ({}..{})",
                make_prefix(indentation_levels),
                type_,
                location.start,
                location.end,
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
        TypedExpression::StructInitialization {
            location,
            fields,
            type_,
        } => {
            writeln!(
                formatter,
                "{}Struct initialization of type {:?} ({}..{})",
                make_prefix(indentation_levels),
                type_,
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
                        "{}Field {}: type: {}",
                        make_prefix(&field_levels),
                        field.name,
                        field.type_
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

fn print_statement(
    statement: &statement::TypedStatement,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match statement {
        TypedStatement::Expression(expression) => {
            writeln!(formatter, "{}Expression:", make_prefix(indentation_levels))?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(false);

            print_expression(expression, &new_indentation_levels, formatter)?;
        }
        TypedStatement::Assignment(assignment) => {
            writeln!(
                formatter,
                "{}Assignment ({}..{}): type: {}",
                make_prefix(indentation_levels),
                assignment.location.start,
                assignment.location.end,
                assignment.type_
            )?;

            let mut new_indentation_levels = indentation_levels.to_vec();
            new_indentation_levels.push(true);

            writeln!(
                formatter,
                "{}Type: {:?}",
                make_prefix(&new_indentation_levels),
                assignment.type_
            )?;
            writeln!(
                formatter,
                "{}Variable name: {} type: {}",
                make_prefix(&new_indentation_levels),
                assignment.variable_name,
                assignment.type_
            )?;

            new_indentation_levels.pop();
            new_indentation_levels.push(false);
            writeln!(formatter, "{}Value:", make_prefix(&new_indentation_levels))?;

            let mut value_levels = new_indentation_levels.clone();
            value_levels.push(false);

            print_expression(&assignment.value, &value_levels, formatter)?;
        }
        TypedStatement::Reassignment(reassignment) => {
            writeln!(
                formatter,
                "{}Reassignment ({}..{}) type: {}",
                make_prefix(indentation_levels),
                reassignment.location.start,
                reassignment.location.end,
                reassignment.type_
            )?;

            match &reassignment.target {
                TypedReassignmentTarget::Variable {
                    location,
                    name,
                    type_,
                } => {
                    writeln!(
                        formatter,
                        "{}Target: Variable {} ({}..{}) type: {}",
                        make_prefix(&[indentation_levels, &[true]].concat()),
                        name,
                        location.start,
                        location.end,
                        type_
                    )?;
                }
                TypedReassignmentTarget::FieldAccess {
                    location,
                    struct_name,
                    field_name,
                    type_,
                } => {
                    writeln!(
                        formatter,
                        "{}Target: Field access {}.{} ({}..{}) type: {}",
                        make_prefix(&[indentation_levels, &[true]].concat()),
                        struct_name,
                        field_name,
                        location.start,
                        location.end,
                        type_
                    )?;
                }
                TypedReassignmentTarget::ArrayAccess {
                    location,
                    array_name,
                    index_expression,
                    type_,
                } => {
                    writeln!(
                        formatter,
                        "{}Target: Array access {} ({}..{}) type: {}",
                        make_prefix(&[indentation_levels, &[true]].concat()),
                        array_name,
                        location.start,
                        location.end,
                        type_
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
        TypedStatement::Loop { body, location } => {
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
        TypedStatement::If {
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
        TypedStatement::Break { location } => {
            writeln!(
                formatter,
                "{}Break ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        TypedStatement::Return { location, value } => {
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
        TypedStatement::Todo { location } => {
            writeln!(
                formatter,
                "{}Todo ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        TypedStatement::Panic { location } => {
            writeln!(
                formatter,
                "{}Panic ({}..{})",
                make_prefix(indentation_levels),
                location.start,
                location.end
            )?;
        }
        TypedStatement::Exit { location } => {
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

fn print_argument(
    arg: &argument::ArgumentTyped,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(
        formatter,
        "{}Argument {} ({}..{}) type: {}",
        make_prefix(indentation_levels),
        arg.name,
        arg.location.start,
        arg.location.end,
        arg.type_
    )?;

    let mut new_indentation_levels = indentation_levels.to_vec();
    new_indentation_levels.push(false);
    writeln!(
        formatter,
        "{}Type: {:?}",
        make_prefix(&new_indentation_levels),
        arg.type_
    )?;

    Ok(())
}

fn print_definition(
    definition: &definition::DefinitionTyped,
    indentation_levels: &[bool],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match definition {
        definition::DefinitionTyped::Struct {
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
                        field.type_
                    )?;
                }
            }
        }
        definition::DefinitionTyped::Function {
            location,
            name,
            arguments,
            body,
            return_type,
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
            items.push("return_type");
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

            let has_more = body.is_some();
            new_indentation_levels.push(has_more);
            writeln!(
                formatter,
                "{}Return type: {:?}",
                make_prefix(&new_indentation_levels),
                return_type // Use directly without `if let Some(...)`
            )?;
            new_indentation_levels.pop();

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
