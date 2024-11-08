mod diff_location;
mod named;

use std::{fmt::Display, fs::File, io::Read, path::Path};

use async_graphql::{Name, Value};
use async_graphql_parser::{
    parse_schema,
    types::{
        ConstDirective, DirectiveDefinition, EnumType, EnumValueDefinition, FieldDefinition,
        InputObjectType, InputValueDefinition, InterfaceType, ObjectType, SchemaDefinition,
        ServiceDocument, TypeDefinition, TypeKind, TypeSystemDefinition, UnionType,
    },
};
use diff_location::{DiffLocation, DiffLocationSegmentType};
use named::Named;

use crate::try_into_service_document::TryIntoServiceDocument;

pub fn diff_schema(
    schema_left: impl TryIntoServiceDocument<Error: std::error::Error>,
    schema_right: impl TryIntoServiceDocument<Error: std::error::Error>,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema_left = schema_left.try_into_service_document()?;
    let schema_right = schema_right.try_into_service_document()?;

    compare_iterators(
        DiffLocation::new(DiffLocationSegmentType::DirectiveDefinition, None),
        || filter_directive_definitions_of_service_document(&schema_left),
        || filter_directive_definitions_of_service_document(&schema_right),
        compare_directive_definitions,
    );

    compare_iterators(
        DiffLocation::new(DiffLocationSegmentType::SchemaDefinition, None),
        || filter_schemas_of_service_document(&schema_left),
        || filter_schemas_of_service_document(&schema_right),
        compare_schema_definitions,
    );

    compare_iterators(
        DiffLocation::new(DiffLocationSegmentType::TypeDefinition, None),
        || filter_types_of_service_document(&schema_left),
        || filter_types_of_service_document(&schema_right),
        compare_type_definitions,
    );

    Ok(())
}

fn filter_directive_definitions_of_service_document(
    service_document: &ServiceDocument,
) -> impl Iterator<Item = &DirectiveDefinition> {
    service_document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            TypeSystemDefinition::Directive(directive) => Some(&directive.node),
            TypeSystemDefinition::Schema(_schema) => None,
            TypeSystemDefinition::Type(r#_type) => None,
        })
}

fn filter_schemas_of_service_document(
    service_document: &ServiceDocument,
) -> impl Iterator<Item = &SchemaDefinition> {
    service_document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            TypeSystemDefinition::Directive(_directive) => None,
            TypeSystemDefinition::Schema(schema) => Some(&schema.node),
            TypeSystemDefinition::Type(r#_type) => None,
        })
}

fn filter_types_of_service_document(
    service_document: &ServiceDocument,
) -> impl Iterator<Item = &TypeDefinition> {
    service_document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            TypeSystemDefinition::Directive(_directive) => None,
            TypeSystemDefinition::Schema(_schema) => None,
            TypeSystemDefinition::Type(r#type) => Some(&r#type.node),
        })
}

fn compare_iterators<
    T: Named,
    LeftIteratorType: Iterator<Item = T>,
    RightIteratorType: Iterator<Item = T>,
>(
    diff_location: DiffLocation,
    left_iter_generator: impl Fn() -> LeftIteratorType,
    right_iter_generator: impl Fn() -> RightIteratorType,
    item_comparator_fn: impl Fn(T, T),
) {
    // breaking changes
    for right in right_iter_generator() {
        match left_iter_generator().find(|left| left.name() == right.name()) {
            Some(left) => item_comparator_fn(left, right),
            None => {
                println!(
                    "{}: item is added to the right, name = '{}'",
                    diff_location,
                    right.name(),
                );
            }
        }
    }

    // non-breaking changes
    for left in left_iter_generator() {
        match right_iter_generator().find(|right| right.name() == left.name()) {
            Some(_right) => (),
            None => {
                println!(
                    "{}: item is removed from right, name = '{}'",
                    diff_location,
                    left.name(),
                );
            }
        }
    }
}

fn load_schema(location: impl AsRef<Path>) -> Result<ServiceDocument, Box<dyn std::error::Error>> {
    let location = location.as_ref();

    println!("Opening schema, path = {:?}", location);

    let mut file = File::open(location)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    Ok(parse_schema(&file_contents)?)
}

fn compare_directive_definitions(
    definition_left: &DirectiveDefinition,
    definition_right: &DirectiveDefinition,
) {
    let DirectiveDefinition {
        description: description_left,
        name: name_left,
        arguments: arguments_left,
        is_repeatable: is_repeatable_left,
        locations: _locations_left,
    } = definition_left;

    let DirectiveDefinition {
        description: description_right,
        name: name_right,
        arguments: arguments_right,
        is_repeatable: is_repeatable_right,
        locations: _locations_right,
    } = definition_right;

    assert_eq!(name_left.node.as_str(), name_right.node.as_str());

    let diff_location = DiffLocation::new(
        DiffLocationSegmentType::DirectiveDefinition,
        Some(name_right.node.as_str()),
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Description, None),
        false,
        description_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        description_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::InputArgument, None),
        || arguments_left.iter().map(|positioned| &positioned.node),
        || arguments_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_input_value_definitions(
                diff_location.push(DiffLocationSegmentType::InputArgument, Some(right.name())),
                left,
                right,
            )
        },
    );

    compare_comparable(
        diff_location.push(DiffLocationSegmentType::IsRepeatable, None),
        is_repeatable_left,
        is_repeatable_right,
    );
}

fn compare_schema_definitions(
    definition_left: &SchemaDefinition,
    definition_right: &SchemaDefinition,
) {
    let SchemaDefinition {
        extend: extend_left,
        directives: directives_left,
        query: query_left,
        mutation: mutation_left,
        subscription: subscription_left,
    } = definition_left;

    let SchemaDefinition {
        extend: extend_right,
        directives: directives_right,
        query: query_right,
        mutation: mutation_right,
        subscription: subscription_right,
    } = definition_right;

    let diff_location = DiffLocation::new(
        DiffLocationSegmentType::SchemaDefinition,
        Some(definition_right.name()),
    );

    compare_comparable(
        diff_location.push(DiffLocationSegmentType::Extends, None),
        extend_left,
        extend_right,
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Directive, None),
        || directives_left.iter().map(|positioned| &positioned.node),
        || directives_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_const_directives(
                diff_location
                    .push(DiffLocationSegmentType::Directive, Some(right.name()))
                    .clone(),
                left,
                right,
            )
        },
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Query, None),
        true,
        query_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        query_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Mutation, None),
        true,
        mutation_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        mutation_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Subscription, None),
        true,
        subscription_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        subscription_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );
}

fn compare_type_definitions(definition_left: &TypeDefinition, definition_right: &TypeDefinition) {
    let TypeDefinition {
        extend: extend_left,
        description: description_left,
        name: name_left,
        directives: directives_left,
        kind: kind_left,
    } = definition_left;

    let TypeDefinition {
        extend: extend_right,
        description: description_right,
        name: name_right,
        directives: directives_right,
        kind: kind_right,
    } = definition_right;

    assert_eq!(name_left.node.as_str(), name_right.node.as_str());
    let name = name_right.node.as_str();

    let mut diff_location = DiffLocation::new(DiffLocationSegmentType::TypeDefinition, Some(name));

    match (kind_left, kind_right) {
        (TypeKind::Scalar, TypeKind::Scalar) => {
            diff_location =
                DiffLocation::new(DiffLocationSegmentType::ScalarDefinition, Some(name));
        }
        (TypeKind::Object(type_left), TypeKind::Object(type_right)) => {
            diff_location =
                DiffLocation::new(DiffLocationSegmentType::ObjectDefinition, Some(name));
            compare_object_types(diff_location.clone(), type_left, type_right)
        }
        (TypeKind::Interface(type_left), TypeKind::Interface(type_right)) => {
            diff_location =
                DiffLocation::new(DiffLocationSegmentType::InterfaceDefinition, Some(name));
            compare_interface_types(diff_location.clone(), type_left, type_right)
        }
        (TypeKind::Union(type_left), TypeKind::Union(type_right)) => {
            diff_location = DiffLocation::new(DiffLocationSegmentType::UnionDefinition, Some(name));
            compare_union_types(diff_location.clone(), type_left, type_right)
        }
        (TypeKind::Enum(type_left), TypeKind::Enum(type_right)) => {
            diff_location = DiffLocation::new(DiffLocationSegmentType::EnumDefinition, Some(name));
            compare_enum_types(diff_location.clone(), type_left, type_right)
        }
        (TypeKind::InputObject(type_left), TypeKind::InputObject(type_right)) => {
            diff_location = DiffLocation::new(DiffLocationSegmentType::InputObject, Some(name));
            compare_input_object_types(diff_location.clone(), type_left, type_right)
        }
        (_kind_left, _kind_right) => {
            println!("{}: type mismatch, name = '{}'", diff_location, name,);
        }
    }

    compare_comparable(
        diff_location.push(DiffLocationSegmentType::Extends, None),
        extend_left,
        extend_right,
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Description, None),
        false,
        description_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        description_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Directive, None),
        || directives_left.iter().map(|positioned| &positioned.node),
        || directives_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_const_directives(
                diff_location.push(DiffLocationSegmentType::Directive, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_optional_strings(
    diff_location: DiffLocation,
    print_string: bool,
    left: Option<&str>,
    right: Option<&str>,
) {
    match (left, right) {
        (Some(mut left), Some(mut right)) => {
            if left != right {
                if !print_string {
                    left = "?";
                    right = "?";
                }

                println!(
                    "{}: left value = '{}', right value = '{}'",
                    diff_location, left, right,
                );
            }
        }
        (None, None) => (),
        // breaking change
        (Some(mut left), None) => {
            if !print_string {
                left = "?";
            }

            println!("{}: removed from right, value = '{}'", diff_location, left,);
        }
        // non-breaking change
        (None, Some(mut right)) => {
            if !print_string {
                right = "?";
            }

            println!("{}: added to right, value = '{}'", diff_location, right,);
        }
    }
}

fn compare_object_types(
    diff_location: DiffLocation,
    type_left: &ObjectType,
    type_right: &ObjectType,
) {
    let ObjectType {
        implements: implements_left,
        fields: fields_left,
    } = type_left;

    let ObjectType {
        implements: implements_right,
        fields: fields_right,
    } = type_right;

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Implements, None),
        || implements_left.iter().map(|positioned| &positioned.node),
        || implements_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_comparable(
                diff_location.push(DiffLocationSegmentType::Implements, Some(left.name())),
                left.as_str(),
                right.as_str(),
            )
        },
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Field, None),
        || fields_left.iter().map(|positioned| &positioned.node),
        || fields_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_field_definitions(
                diff_location.push(DiffLocationSegmentType::Field, Some(left.name())),
                left,
                right,
            )
        },
    );
}

fn compare_interface_types(
    diff_location: DiffLocation,
    type_left: &InterfaceType,
    type_right: &InterfaceType,
) {
    let InterfaceType {
        implements: implements_left,
        fields: fields_left,
    } = type_left;

    let InterfaceType {
        implements: implements_right,
        fields: fields_right,
    } = type_right;

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Implements, None),
        || implements_left.iter().map(|positioned| &positioned.node),
        || implements_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_comparable(
                diff_location.push(DiffLocationSegmentType::Implements, Some(right.name())),
                left.as_str(),
                right.as_str(),
            )
        },
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Field, None),
        || fields_left.iter().map(|positioned| &positioned.node),
        || fields_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_field_definitions(
                diff_location.push(DiffLocationSegmentType::Field, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_union_types(diff_location: DiffLocation, type_left: &UnionType, type_right: &UnionType) {
    let UnionType {
        members: members_left,
    } = type_left;

    let UnionType {
        members: members_right,
    } = type_right;

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::UnionMemberDefinition, None),
        || members_left.iter().map(|positioned| &positioned.node),
        || members_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_comparable(
                diff_location.push(
                    DiffLocationSegmentType::UnionMemberDefinition,
                    Some(right.name()),
                ),
                left.as_str(),
                right.as_str(),
            )
        },
    );
}

fn compare_enum_types(diff_location: DiffLocation, type_left: &EnumType, type_right: &EnumType) {
    let EnumType {
        values: values_left,
    } = type_left;

    let EnumType {
        values: values_right,
    } = type_right;

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::EnumValueDefinition, None),
        || values_left.iter().map(|positioned| &positioned.node),
        || values_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_enum_value_definitions(
                diff_location.push(
                    DiffLocationSegmentType::EnumValueDefinition,
                    Some(right.name()),
                ),
                left,
                right,
            )
        },
    );
}

fn compare_input_object_types(
    diff_location: DiffLocation,
    type_left: &InputObjectType,
    type_right: &InputObjectType,
) {
    let InputObjectType {
        fields: fields_left,
    } = type_left;

    let InputObjectType {
        fields: fields_right,
    } = type_right;

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Field, None),
        || fields_left.iter().map(|positioned| &positioned.node),
        || fields_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_input_value_definitions(
                diff_location.push(DiffLocationSegmentType::Field, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_field_definitions(
    diff_location: DiffLocation,
    definition_left: &FieldDefinition,
    definition_right: &FieldDefinition,
) {
    let FieldDefinition {
        description: description_left,
        name: name_left,
        arguments: arguments_left,
        ty: ty_left,
        directives: directives_left,
    } = definition_left;

    let FieldDefinition {
        description: description_right,
        name: name_right,
        arguments: arguments_right,
        ty: ty_right,
        directives: directives_right,
    } = definition_right;

    assert_eq!(name_left.node.as_str(), name_right.node.as_str());

    compare_comparable(
        diff_location.push(DiffLocationSegmentType::Type, None),
        &ty_left.node,
        &ty_right.node,
    );

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Description, None),
        false,
        description_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        description_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::InputArgument, None),
        || arguments_left.iter().map(|positioned| &positioned.node),
        || arguments_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_input_value_definitions(
                diff_location.push(DiffLocationSegmentType::InputArgument, Some(right.name())),
                left,
                right,
            )
        },
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Directive, None),
        || directives_left.iter().map(|positioned| &positioned.node),
        || directives_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_const_directives(
                diff_location.push(DiffLocationSegmentType::Directive, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_const_directives(
    diff_location: DiffLocation,
    directive_left: &ConstDirective,
    directive_right: &ConstDirective,
) {
    let ConstDirective {
        name: name_left,
        arguments: arguments_left,
    } = directive_left;

    let ConstDirective {
        name: name_right,
        arguments: arguments_right,
    } = directive_right;

    assert_eq!(name_left.node.as_str(), name_right.node.as_str());

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::DirectiveArgument, None),
        || {
            arguments_left
                .iter()
                .map(|(name, value)| (&name.node, &value.node))
        },
        || {
            arguments_right
                .iter()
                .map(|(name, value)| (&name.node, &value.node))
        },
        |left, right| {
            compare_const_directive_argument_value(
                diff_location.push(
                    DiffLocationSegmentType::DirectiveArgument,
                    Some(right.name()),
                ),
                left,
                right,
            )
        },
    );
}

fn compare_enum_value_definitions(
    diff_location: DiffLocation,
    definition_left: &EnumValueDefinition,
    definition_right: &EnumValueDefinition,
) {
    let EnumValueDefinition {
        description: description_left,
        value: value_left,
        directives: directives_left,
    } = definition_left;

    let EnumValueDefinition {
        description: description_right,
        value: value_right,
        directives: directives_right,
    } = definition_right;

    assert_eq!(value_left.node.as_str(), value_right.node.as_str());

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Description, None),
        false,
        description_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        description_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Directive, None),
        || directives_left.iter().map(|positioned| &positioned.node),
        || directives_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_const_directives(
                diff_location.push(DiffLocationSegmentType::Directive, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_input_value_definitions(
    diff_location: DiffLocation,
    definition_left: &InputValueDefinition,
    definition_right: &InputValueDefinition,
) {
    let InputValueDefinition {
        description: description_left,
        name: name_left,
        ty: ty_left,
        default_value: default_value_left,
        directives: directives_left,
    } = definition_left;

    let InputValueDefinition {
        description: description_right,
        name: name_right,
        ty: ty_right,
        default_value: default_value_right,
        directives: directives_right,
    } = definition_right;

    assert_eq!(name_left.node.as_str(), name_right.node.as_str());

    compare_comparable(
        diff_location.push(DiffLocationSegmentType::Type, None),
        &ty_left.node,
        &ty_right.node,
    );

    {
        let diff_location = diff_location.push(DiffLocationSegmentType::DefaultValue, None);

        match (
            default_value_left.as_ref().map(|value| &value.node),
            default_value_right.as_ref().map(|value| &value.node),
        ) {
            (Some(left), Some(right)) => {
                if left != right {
                    println!(
                        "{}: left value = '{}', right value = '{}'",
                        diff_location, left, right,
                    );
                }
            }
            (None, None) => (),
            // breaking change
            (Some(left), None) => {
                println!("{}: removed from right, value = '{}'", diff_location, left,);
            }
            // non-breaking change
            (None, Some(right)) => {
                println!("{}: added to right, value = '{}'", diff_location, right,);
            }
        }
    }

    compare_optional_strings(
        diff_location.push(DiffLocationSegmentType::Description, None),
        false,
        description_left
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
        description_right
            .as_ref()
            .map(|positioned| positioned.node.as_str()),
    );

    compare_iterators(
        diff_location.push(DiffLocationSegmentType::Directive, None),
        || directives_left.iter().map(|positioned| &positioned.node),
        || directives_right.iter().map(|positioned| &positioned.node),
        |left, right| {
            compare_const_directives(
                diff_location.push(DiffLocationSegmentType::Directive, Some(right.name())),
                left,
                right,
            )
        },
    );
}

fn compare_const_directive_argument_value(
    diff_location: DiffLocation,
    arg_left: (&Name, &Value),
    arg_right: (&Name, &Value),
) {
    compare_comparable(
        diff_location.push(DiffLocationSegmentType::DirectiveArgument, None),
        arg_left.1,
        arg_right.1,
    );
}

fn compare_comparable<T: Display + Eq + PartialEq + ?Sized>(
    diff_location: DiffLocation,
    left: &T,
    right: &T,
) {
    if *left != *right {
        println!(
            "{}: left value = {}, right value = {}",
            diff_location, left, right,
        )
    }
}
