#![allow(unused)]

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};

/// Convert a frontastic component schema to a storybook story file.
#[derive(Parser)]
struct Cli {
    /// The path to the file to convert
    #[clap(parse(from_os_str))]
    file_path: std::path::PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Schema {
    name: String,
    schema: Vec<SchemaGroup>,
}

#[derive(Serialize, Deserialize)]
struct SchemaGroup {
    fields: Vec<HashMap<String, Value>>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let file = File::open(&args.file_path)
        .with_context(|| format!("could not read file `{}`", &args.file_path.display()))?;

    let reader = BufReader::new(file);

    let schema: Schema = serde_json::from_reader(reader).with_context(|| {
        format!(
            "could not serialize json from file `{}`",
            &args.file_path.display()
        )
    })?;

    let name = if schema.name.contains(' ') {
        schema.name.replace(' ', "")
    } else {
        schema.name
    };
    let file_name = format!("{}.stories.tsx", name);
    let mut file = File::create(&file_name)
        .with_context(|| format!("could not create file `{}`", &file_name))?;

    let basic_imports  = String::from("import React from \"react\";\nimport { Story, Meta } from \"@storybook/react\";\nimport { documentationPath } from \"@srcDS/storybook/constants\";\n\n");
    let component_import = format!(
        "import {}, {{\n  I{},\n}} from \"@srcDS/components/organisms/{}\";\n\n",
        name, name, name
    );
    let meta_header = format!(
        "export default {{\n  component: {},\n  title: `${{documentationPath}}/{}`,\n  ",
        name, name
    );

    let mut arg_types = String::from("argTypes: {\n");

    for schema_group in schema.schema {
        for schema_item in schema_group.fields {
            if schema_item["type"] != "description" {
                let mut default_value = String::from("");

                if schema_item.contains_key("default") {
                    match &schema_item["default"] {
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

                let arg_type = format!(
                    "    {}: {{ {}table: {{ disable: true }} }},\n",
                    schema_item["field"].as_str().unwrap(),
                    default_value
                );
                arg_types += arg_type.as_str();
            }
        }
    }

    let meta_footer = "  },\n} as Meta;\n\n";

    let story_template = format!("//TODO: Wrap component with decorators if needed\nconst StoryTpl: Story<I{}> = (args) => <{} {{...args}} />;\n\n", name, name);

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
