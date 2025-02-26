use std::collections::HashMap;
use std::fmt::Write;

use capnp::schema_capnp;
use capnp::schema_capnp::code_generator_request::requested_file;
use capnp::schema_capnp::field::NO_DISCRIMINANT;

const OCAML_KEYWORDS: [&str; 56] = [
    "and",
    "as",
    "assert",
    "asr",
    "begin",
    "class",
    "constraint",
    "do",
    "done",
    "downto",
    "else",
    "end",
    "exception",
    "external",
    "false",
    "for",
    "fun",
    "function",
    "functor",
    "if",
    "in",
    "include",
    "inherit",
    "initializer",
    "land",
    "lazy",
    "let",
    "lor",
    "lsl",
    "lsr",
    "lxor",
    "match",
    "method",
    "mod",
    "module",
    "mutable",
    "new",
    "nonrec",
    "object",
    "of",
    "open",
    "or",
    "private",
    "rec",
    "sig",
    "struct",
    "then",
    "to",
    "true",
    "try",
    "type",
    "val",
    "virtual",
    "when",
    "while",
    "with",
];

lazy_static::lazy_static! {
    static ref OCAML_KEYWORDS_SET: std::collections::HashSet<&'static str> = {
        let mut set = std::collections::HashSet::new();
        for keyword in OCAML_KEYWORDS.iter() {
            set.insert(*keyword);
        }
        set
    };
}

fn escape_keyword(name: String) -> String {
    if OCAML_KEYWORDS_SET.contains(name.as_str()) {
        format!("{}_", name)
    } else {
        name
    }
}

fn pascal_to_snake(name: &str) -> String {
    let mut result = String::new();
    let mut last_was_upper = false;
    for c in name.chars() {
        if c.is_uppercase() {
            if last_was_upper {
                result.push(c.to_lowercase().next().unwrap());
            } else {
                if !result.is_empty() {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
            }
            last_was_upper = true;
        } else {
            result.push(c);
            last_was_upper = false;
        }
    }
    result
}

struct ParamEnv<'a> {
    scope_id: u64,
    parameters: Vec<String>,
    parent_env: Option<&'a ParamEnv<'a>>,
}

impl<'a> ParamEnv<'a> {
    fn get(&self, scope_id: u64) -> &Vec<String> {
        if self.scope_id == scope_id {
            &self.parameters
        } else {
            self.parent_env.unwrap().get(scope_id)
        }
    }
}

fn print_type_decoder(
    decoder: &mut String,
    node_name_map: &HashMap<u64, String>,
    param_env: Option<&ParamEnv<'_>>,
    type_: schema_capnp::type_::Reader<'_>,
) {
    match type_.which().unwrap() {
        schema_capnp::type_::Struct(struct_) => {
            let type_id = struct_.get_type_id();
            let type_name = node_name_map.get(&type_id).unwrap();
            write!(decoder, "decode_{}", type_name).unwrap();
        }
        schema_capnp::type_::Enum(enum_) => {
            let type_id = enum_.get_type_id();
            let type_name = node_name_map.get(&type_id).unwrap();
            write!(decoder, "decode_{}", type_name).unwrap();
        }
        schema_capnp::type_::AnyPointer(any_pointer) => match any_pointer.which().unwrap() {
            schema_capnp::type_::any_pointer::Parameter(parameter) => {
                write!(
                    decoder,
                    "decode_{}",
                    param_env.unwrap().get(parameter.get_scope_id())
                        [parameter.get_parameter_index() as usize]
                )
                .unwrap();
            }
            _ => todo!(),
        },
        schema_capnp::type_::Text(()) => {
            write!(decoder, "(fun x: string -> x)").unwrap();
        }
        _ => todo!(),
    }
}

fn print_type_decoding(
    decoder: &mut String,
    node_name_map: &HashMap<u64, String>,
    param_env: Option<&ParamEnv<'_>>,
    type_: schema_capnp::type_::Reader<'_>,
    reader: &str,
) {
    match type_.which().unwrap() {
        schema_capnp::type_::Void(())
        | schema_capnp::type_::Bool(())
        | schema_capnp::type_::Int8(())
        | schema_capnp::type_::Int16(())
        | schema_capnp::type_::Int32(())
        | schema_capnp::type_::Int64(())
        | schema_capnp::type_::Uint8(())
        | schema_capnp::type_::Uint16(())
        | schema_capnp::type_::Uint32(())
        | schema_capnp::type_::Uint64(())
        | schema_capnp::type_::Float32(())
        | schema_capnp::type_::Float64(())
        | schema_capnp::type_::Text(())
        | schema_capnp::type_::Data(()) => {
            write!(decoder, "{}", reader).unwrap();
        }
        schema_capnp::type_::List(list) => {
            write!(decoder, "Capnp.Array.map_list {} ~f:", reader).unwrap();
            print_type_decoder(
                decoder,
                node_name_map,
                param_env,
                list.get_element_type().unwrap(),
            );
        }
        schema_capnp::type_::Enum(enum_) => {
            write!(
                decoder,
                "decode_{} {}",
                node_name_map.get(&enum_.get_type_id()).unwrap(),
                reader
            )
            .unwrap();
        }
        schema_capnp::type_::Struct(struct_) => {
            write!(
                decoder,
                "decode_{}",
                node_name_map.get(&struct_.get_type_id()).unwrap()
            )
            .unwrap();
            if struct_.has_brand() {
                let brand = struct_.get_brand().unwrap();
                for scope in brand.get_scopes().unwrap().iter() {
                    //let scope_id = scope.get_scope_id();
                    match scope.which().unwrap() {
                        schema_capnp::brand::scope::Bind(bindings) => {
                            for binding in bindings.unwrap().iter() {
                                match binding.which().unwrap() {
                                    schema_capnp::brand::binding::Which::Type(t) => {
                                        write!(decoder, " ").unwrap();
                                        print_type_decoder(
                                            decoder,
                                            node_name_map,
                                            param_env,
                                            t.unwrap(),
                                        );
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
            write!(decoder, " {}", reader).unwrap();
        }
        schema_capnp::type_::AnyPointer(any_pointer) => match any_pointer.which().unwrap() {
            schema_capnp::type_::any_pointer::Parameter(parameter) => {
                write!(
                    decoder,
                    "decode_{} (R.of_pointer {})",
                    param_env.unwrap().get(parameter.get_scope_id())
                        [parameter.get_parameter_index() as usize],
                    reader
                )
                .unwrap();
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}

fn print_type<'a>(
    node_name_map: &HashMap<u64, String>,
    param_env: Option<&ParamEnv<'_>>,
    type_: schema_capnp::type_::Reader<'a>,
) {
    match type_.which().unwrap() {
        schema_capnp::type_::Which::Void(()) => {
            print!("unit");
        }
        schema_capnp::type_::Bool(()) => {
            print!("bool");
        }
        schema_capnp::type_::Int8(()) => {
            print!("int");
        }
        schema_capnp::type_::Int16(()) => {
            print!("int");
        }
        schema_capnp::type_::Int32(()) => {
            print!("int32");
        }
        schema_capnp::type_::Int64(()) => {
            print!("int64");
        }
        schema_capnp::type_::Uint8(()) => {
            print!("int");
        }
        schema_capnp::type_::Uint16(()) => {
            print!("int");
        }
        schema_capnp::type_::Uint32(()) => {
            print!("Stdint.uint32");
        }
        schema_capnp::type_::Uint64(()) => {
            print!("Stdint.uint64");
        }
        schema_capnp::type_::Float32(()) => {
            print!("float");
        }
        schema_capnp::type_::Float64(()) => {
            print!("float");
        }
        schema_capnp::type_::Text(()) => {
            print!("string");
        }
        schema_capnp::type_::Data(()) => {
            print!("string");
        }
        schema_capnp::type_::List(list) => {
            print_type(node_name_map, param_env, list.get_element_type().unwrap());
            print!(" list");
        }
        schema_capnp::type_::Enum(enum_) => {
            print!("{}", node_name_map.get(&enum_.get_type_id()).unwrap());
        }
        schema_capnp::type_::Struct(struct_) => {
            if struct_.has_brand() {
                let brand = struct_.get_brand().unwrap();
                for scope in brand.get_scopes().unwrap().iter() {
                    //let scope_id = scope.get_scope_id();
                    match scope.which().unwrap() {
                        schema_capnp::brand::scope::Bind(bindings) => {
                            for binding in bindings.unwrap().iter() {
                                match binding.which().unwrap() {
                                    schema_capnp::brand::binding::Which::Type(t) => {
                                        print_type(node_name_map, param_env, t.unwrap());
                                        print!(" ");
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
            print!("{}", node_name_map.get(&struct_.get_type_id()).unwrap());
        }
        schema_capnp::type_::Interface(interface) => {
            print!("{}", interface.get_type_id());
        }
        schema_capnp::type_::AnyPointer(any_pointer) => match any_pointer.which().unwrap() {
            schema_capnp::type_::any_pointer::Parameter(parameter) => {
                print!(
                    "'{}",
                    param_env.unwrap().get(parameter.get_scope_id())
                        [parameter.get_parameter_index() as usize]
                );
            }
            _ => todo!(),
        },
    }
}

fn enter_nested_nodes(
    node_map: &HashMap<u64, schema_capnp::node::Reader>,
    node_name_map: &mut HashMap<u64, String>,
    qualifier: &str,
    nested_nodes: capnp::struct_list::Reader<schema_capnp::node::nested_node::Owned>,
) {
    for nested_node in nested_nodes.iter() {
        let nested_id = nested_node.get_id();
        let nested_name = escape_keyword(pascal_to_snake(
            nested_node.get_name().unwrap().to_str().unwrap(),
        ));
        let nested_qualifier = if qualifier.is_empty() {
            nested_name.to_string()
        } else {
            format!("{}_{}", qualifier, nested_name)
        };
        node_name_map.insert(nested_id, nested_qualifier.clone());
        let node = node_map.get(&nested_id).unwrap();
        enter_nested_nodes(
            node_map,
            node_name_map,
            &nested_qualifier,
            node.get_nested_nodes().unwrap(),
        );
    }
}

fn print_nested_nodes(
    decoder: &mut String,
    node_map: &HashMap<u64, schema_capnp::node::Reader>,
    node_name_map: &HashMap<u64, String>,
    is_first_type: &mut bool,
    reader_path: &str,
    nested_nodes: capnp::struct_list::Reader<schema_capnp::node::nested_node::Owned>,
) {
    for nested_node in nested_nodes.iter() {
        let nested_id = nested_node.get_id();
        let nested_node_name = nested_node.get_name().unwrap().to_str().unwrap();
        let nested_reader_path = format!("{}.{}", reader_path, nested_node_name);
        let nested_node = node_map.get(&nested_id).unwrap();
        match nested_node.which().unwrap() {
            schema_capnp::node::Struct(struct_node) => {
                print_nested_nodes(
                    decoder,
                    node_map,
                    node_name_map,
                    is_first_type,
                    &nested_reader_path,
                    nested_node.get_nested_nodes().unwrap(),
                );

                if struct_node.has_fields() {
                    if *is_first_type {
                        *is_first_type = false;
                        print!("type ");
                        write!(decoder, "let rec ").unwrap();
                    } else {
                        print!("\nand ");
                        write!(decoder, "\nand ").unwrap();
                    }
                    let name = node_name_map.get(&nested_id).unwrap();
                    write!(decoder, "decode_{}", name).unwrap();
                    let mut param_env = ParamEnv {
                        scope_id: nested_id,
                        parameters: Vec::new(),
                        parent_env: None,
                    };
                    if nested_node.has_parameters() {
                        let mut generic_args = String::new();
                        let mut fun_param_types = String::new();
                        let mut fun_args = String::new();
                        write!(decoder, ":").unwrap();
                        let params = nested_node.get_parameters().unwrap();
                        for param in params.iter() {
                            let name = pascal_to_snake(param.get_name().unwrap().to_str().unwrap());
                            print!("'{} ", name);
                            write!(generic_args, "'{} ", name).unwrap();
                            write!(decoder, " 'r{} '{}", name, name).unwrap();
                            write!(fun_param_types, "('r{} S.reader_t -> '{}) -> ", name, name)
                                .unwrap();
                            write!(fun_args, " decode_{}", name).unwrap();

                            param_env.parameters.push(name);
                        }
                        print!("{} =\n", name);
                        write!(
                            decoder,
                            ". {}{}.t -> {}{} = fun{} r ->\n",
                            fun_param_types, nested_reader_path, generic_args, name, fun_args
                        )
                        .unwrap();
                    } else {
                        print!("{} =\n", name);
                        write!(decoder, " r: {} =\n", name).unwrap();
                    }

                    let discriminant_count = struct_node.get_discriminant_count();
                    if discriminant_count > 0 {
                        write!(decoder, "  match {}.get r with\n", nested_reader_path).unwrap();
                        let mut is_first_variant = true;
                        let fields = struct_node.get_fields().unwrap();
                        for field in fields.iter() {
                            if is_first_variant {
                                is_first_variant = false;
                            } else {
                                println!();
                            }
                            assert!(field.get_discriminant_value() != NO_DISCRIMINANT);
                            let name = field.get_name().unwrap().to_str().unwrap();
                            let capitalized_name = name
                                .chars()
                                .next()
                                .unwrap()
                                .to_uppercase()
                                .collect::<String>()
                                + &name[1..];
                            print!("  | {}", capitalized_name);
                            write!(decoder, "  | {}", capitalized_name).unwrap();
                            match field.which().unwrap() {
                                schema_capnp::field::Slot(slot) => {
                                    let type_ = slot.get_type().unwrap();
                                    if let schema_capnp::type_::Void(()) = type_.which().unwrap() {
                                        write!(decoder, " -> {}\n", capitalized_name).unwrap();
                                    } else {
                                        print!(" of ");
                                        print_type(&node_name_map, Some(&param_env), type_);
                                        write!(decoder, " r' -> {} (", capitalized_name).unwrap();
                                        print_type_decoding(
                                            decoder,
                                            &node_name_map,
                                            Some(&param_env),
                                            type_,
                                            "r'",
                                        );
                                        write!(decoder, ")\n").unwrap();
                                    }
                                }
                                schema_capnp::field::Group(group) => {
                                    let type_ = group.get_type_id();
                                    let group_node = node_map.get(&type_).unwrap();
                                    match group_node.which().unwrap() {
                                        schema_capnp::node::Struct(struct_node) => {
                                            print!(" of {{");
                                            write!(decoder, " r' -> {} {{", capitalized_name)
                                                .unwrap();
                                            let mut is_first_field = true;
                                            let fields = struct_node.get_fields().unwrap();
                                            for field in fields.iter() {
                                                let name =
                                                    field.get_name().unwrap().to_str().unwrap();
                                                match field.which().unwrap() {
                                                    schema_capnp::field::Slot(slot) => {
                                                        if is_first_field {
                                                            is_first_field = false;
                                                        } else {
                                                            print!("; ");
                                                            write!(decoder, "; ").unwrap();
                                                        }
                                                        let snake_name = pascal_to_snake(name);
                                                        let escaped_snake_name =
                                                            escape_keyword(snake_name.clone());
                                                        print!("{}: ", escaped_snake_name);
                                                        write!(
                                                            decoder,
                                                            "{} = ",
                                                            escaped_snake_name
                                                        )
                                                        .unwrap();
                                                        let type_ = slot.get_type().unwrap();
                                                        print_type(
                                                            &node_name_map,
                                                            Some(&param_env),
                                                            type_,
                                                        );
                                                        print_type_decoding(
                                                            decoder,
                                                            &node_name_map,
                                                            Some(&param_env),
                                                            type_,
                                                            &format!(
                                                                "({}.{}.{}_get r')",
                                                                nested_reader_path,
                                                                capitalized_name,
                                                                snake_name
                                                            ),
                                                        );
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            print!("}}");
                                            write!(decoder, "}}").unwrap();
                                        }
                                        _ => todo!(),
                                    }
                                }
                            }
                        }
                        write!(
                            decoder,
                            "  | Undefined _ -> failwith \"Undefined discriminant\"\n"
                        )
                        .unwrap();
                    } else {
                        print!("  {{");
                        write!(decoder, "  {{").unwrap();
                        let mut is_first_field = true;
                        let fields = struct_node.get_fields().unwrap();
                        for field in fields.iter() {
                            let name = field.get_name().unwrap().to_str().unwrap();
                            match field.which().unwrap() {
                                schema_capnp::field::Slot(slot) => {
                                    if is_first_field {
                                        is_first_field = false;
                                    } else {
                                        print!(";");
                                        write!(decoder, ";").unwrap();
                                    }
                                    let snake_name = pascal_to_snake(name);
                                    let escaped_snake_name = escape_keyword(snake_name.clone());
                                    print!("\n    {}: ", escaped_snake_name);
                                    write!(decoder, "\n    {} = ", escaped_snake_name).unwrap();
                                    let type_ = slot.get_type().unwrap();
                                    print_type(&node_name_map, Some(&param_env), type_);
                                    print_type_decoding(
                                        decoder,
                                        &node_name_map,
                                        Some(&param_env),
                                        type_,
                                        &format!("({}.{}_get r)", nested_reader_path, snake_name),
                                    );
                                }
                                _ => todo!(),
                            }
                        }
                        print!("\n  }}");
                        write!(decoder, "\n  }}").unwrap();
                    }
                }
            }
            schema_capnp::node::Which::Enum(enum_) => {
                if *is_first_type {
                    *is_first_type = false;
                    print!("type ");
                    write!(decoder, "let rec ").unwrap();
                } else {
                    print!("\nand ");
                    write!(decoder, "\nand ").unwrap();
                }
                let name = node_name_map.get(&nested_id).unwrap();
                print!("{} =", name);
                write!(
                    decoder,
                    "decode_{} (r: {}.t): {} = match r with",
                    name, nested_reader_path, name
                )
                .unwrap();
                let enumerants = enum_.get_enumerants().unwrap();
                for enumerant in enumerants.iter() {
                    let name = enumerant.get_name().unwrap().to_str().unwrap();
                    let capitalized_name = name
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .collect::<String>()
                        + &name[1..];
                    print!("\n  | {}", capitalized_name);
                    write!(
                        decoder,
                        "\n  | {} -> {}",
                        capitalized_name, capitalized_name
                    )
                    .unwrap();
                }
                write!(
                    decoder,
                    "\n  | Undefined _ -> failwith \"Undefined enumerant\""
                )
                .unwrap();
            }
            _ => todo!(),
        }
    }
}

fn process_requested_file(
    node_map: &HashMap<u64, schema_capnp::node::Reader<'_>>,
    requested_file: requested_file::Reader,
) {
    let mut decoder = String::new();
    // Create a map of node id to node name
    let mut node_name_map = std::collections::HashMap::new();
    let id = requested_file.get_id();
    let node = node_map.get(&id).unwrap();
    assert!(match node.which().unwrap() {
        schema_capnp::node::File(()) => true,
        _ => false,
    });
    let nested_nodes = node.get_nested_nodes().unwrap();
    enter_nested_nodes(&node_map, &mut node_name_map, "", nested_nodes);

    let mut is_first_type = true;
    let id = requested_file.get_id();
    let filename = requested_file.get_filename().unwrap().to_str().unwrap();
    let basename = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let capitalized_basename = basename
        .chars()
        .next()
        .unwrap()
        .to_uppercase()
        .collect::<String>()
        + &basename[1..];
    let node = node_map.get(&id).unwrap();
    assert!(match node.which().unwrap() {
        schema_capnp::node::File(()) => true,
        _ => false,
    });
    let nested_nodes = node.get_nested_nodes().unwrap();
    print_nested_nodes(
        &mut decoder,
        &node_map,
        &node_name_map,
        &mut is_first_type,
        "R",
        nested_nodes,
    );
    println!();
    println!();
    println!(
        "module S = {}.Make (Capnp.BytesMessage)",
        capitalized_basename
    );
    println!("module R = S.Reader");
    println!();
    println!("{}", decoder);
}

fn main() {
    // Print message to standard error
    eprintln!("Reading code generator request from stdin...");
    let stdin = ::std::io::stdin();
    let message_reader =
        capnp::serialize::read_message(&mut stdin.lock(), ::capnp::message::ReaderOptions::new())
            .unwrap();
    let code_generator_request = message_reader
        .get_root::<schema_capnp::code_generator_request::Reader>()
        .unwrap();
    let nodes = code_generator_request.get_nodes().unwrap();
    // Create a map of node id to node
    let mut node_map = std::collections::HashMap::new();
    for node in nodes.iter() {
        let id = node.get_id();
        node_map.insert(id, node);
    }
    let requested_files = code_generator_request.get_requested_files().unwrap();
    for requested_file in requested_files.iter() {
        process_requested_file(&node_map, requested_file);
    }
}
