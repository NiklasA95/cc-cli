use anyhow::{Context, Error, Result};
use clap::Parser;
use convert_case::{Case, Casing};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

/// Convert a frontastic component schema to a storybook story file.
#[derive(Parser)]
struct Cli {
    /// The path of the file to convert
    #[clap(parse(from_os_str))]
    file_path: PathBuf,
}

#[derive(Deserialize)]
struct Schema {
    name: String,
    schema: Vec<SchemaGroup>,
}

#[derive(Deserialize)]
struct SchemaGroup {
    name: String,
    fields: Vec<SchemaItem>,
}
#[derive(Deserialize)]
struct SchemaItem {
    #[serde(rename(deserialize = "type"))]
    type_: String,
    field: Option<String>,
    default: Option<Value>,
    text: Option<String>,
}

fn deserialize_file_content(file_path: &PathBuf) -> Result<Schema, Error> {
    let file = File::open(file_path)
        .with_context(|| format!("could not read file `{}`", file_path.display()))?;

    let reader = BufReader::new(file);

    let schema: Schema = serde_json::from_reader(reader).with_context(|| {
        format!(
            "could not deserialize json from file `{}`",
            file_path.display()
        )
    })?;

    Ok(schema)
}

fn create_component_name_and_file(name: String) -> Result<(String, File), Error> {
    let name = name.replace('/', " ").to_case(Case::Pascal);
    let file_name = format!("{}.stories.tsx", name);
    let file = File::create(&file_name)
        .with_context(|| "could not create file for name provided in schema")?;

    Ok((name, file))
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let schema = deserialize_file_content(&args.file_path)?;

    let (name, mut file) = create_component_name_and_file(schema.name)?;

    let basic_imports  = String::from("import React from \"react\";\nimport { Story, Meta } from \"@storybook/react\";\nimport { documentationPath } from \"@srcDS/storybook/constants\";\n\n");
    let component_import = format!(
        "import {}, {{\n    I{},\n}} from \"@srcDS/components/organisms/{}\";\n\n",
        name, name, name
    );
    let meta_header = format!(
        "export default {{\n    component: {},\n    title: `${{documentationPath}}/{}`,\n    ",
        name, name
    );

    let mut arg_types = String::from("argTypes: {\n");

    let mut description_cache = None;

    for schema_group in schema.schema {
        for schema_item in schema_group.fields {
            if let Some(description) = schema_item.text {
                description_cache = Some(format!("description: \"{}\", ", description));
            } else {
                // TODO: Add `enum` to types
                let value_type = match schema_item.type_.as_str() {
                    "string" | "markdown" => {
                        format!(
                            "control: \"text\", table: {{ category: \"{}\" }}, ",
                            schema_group.name
                        )
                    }
                    "boolean" => {
                        format!(
                            "control: \"boolean\", table: {{ category: \"{}\" }}, ",
                            schema_group.name
                        )
                    }
                    "number" => format!(
                        "control: \"number\", table: {{ category: \"{}\" }}, ",
                        schema_group.name
                    ),
                    "enum" => format!(
                        "control: \"radio\", table: {{ category: \"{}\" }}, ",
                        schema_group.name
                    ),
                    _ => String::from("table: { disable: true }, "),
                };

                let mut default_value = String::from("");

                if let Some(default) = schema_item.default {
                    match default {
                        Value::String(default) => {
                            default_value = format!("defaultValue: \"{}\", ", default)
                        }
                        Value::Bool(default) => {
                            default_value = format!("defaultValue: {}, ", default)
                        }
                        Value::Number(default) => {
                            default_value = format!("defaultValue: {}, ", default)
                        }
                        _ => (),
                    }
                };

                let description: String = if let Some(description) = &description_cache {
                    description.to_string()
                } else {
                    String::from("")
                };

                description_cache = None;

                let arg_type = format!(
                    "        {}: {{ {}{}{}}},\n",
                    schema_item.field.unwrap(),
                    value_type,
                    default_value,
                    description
                );
                arg_types += arg_type.as_str();
            }
        }
    }

    let meta_footer = "    },\n} as Meta;\n\n";

    let story_template = format!("// TODO: Wrap component with decorators if needed\nconst StoryTpl: Story<I{}> = (args) => <{} {{...args}} />;\n\n", name, name);

    let default_story = format!("export const DefaultStory = StoryTpl.bind({{}});\nDefaultStory.storyName = \"Default {}\";\nDefaultStory.args = {{}};\n", name);

    let buf_string = basic_imports
        + component_import.as_str()
        + meta_header.as_str()
        + arg_types.as_str()
        + meta_footer
        + story_template.as_str()
        + default_story.as_str();

    file.write_all(buf_string.as_bytes())
        .with_context(|| "the given schema could't be converted to a story file")?;

    Ok(())
}
