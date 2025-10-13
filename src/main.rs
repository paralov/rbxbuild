use std::{collections::HashMap, io::{Read, IsTerminal}};
use anyhow::Result;
use serde::Deserialize;
use rbx_dom_weak::{InstanceBuilder, WeakDom};
use rbx_xml::to_writer_default;

mod resolution;

use resolution::UnresolvedValue;

// Required by resolution module
const REF_POINTER_ATTRIBUTE_PREFIX: &str = "RojoId_";

fn main() -> Result<()> {
    // Get JSON input either from command-line argument or stdin
    let json_input = if let Some(arg) = std::env::args().nth(1) {
        // Use command-line argument if provided
        arg
    } else if !std::io::stdin().is_terminal() {
        // Read from stdin if it's not a terminal (piped input)
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    } else {
        // No input provided
        eprintln!("Error: No input provided. Please provide JSON as an argument or pipe it to stdin.");
        eprintln!("Usage: rojo-build-lite '<json>' or echo '<json>' | rojo-build-lite");
        std::process::exit(1);
    };

    // Exit if input is empty
    if json_input.trim().is_empty() {
        eprintln!("Error: Empty input provided.");
        std::process::exit(1);
    }

    // Parse JSON as a project file
    let project: Project = serde_json::from_str(&json_input)?;

    // Get the project name for the root instance
    let root_name = project.name.as_deref().unwrap_or("ROOT");

    // Convert tree to WeakDom
    let dom = instantiate(&project.tree, root_name)?;

    // Serialize to XML
    // If the root is DataModel, output its children as siblings (like Rojo does for place files)
    // Otherwise, output the root instance itself
    let mut buffer = Vec::new();
    let root_ref = dom.root_ref();
    let root_instance = dom.get_by_ref(root_ref).unwrap();
    
    let ids_to_write = if root_instance.class == "DataModel" {
        // Place files don't contain an entry for the DataModel
        // Write the children as root-level siblings
        root_instance.children().to_vec()
    } else {
        // For models, write the root instance
        vec![root_ref]
    };
    
    to_writer_default(&mut buffer, &dom, &ids_to_write)?;

    // Print XML to stdout
    println!("{}", String::from_utf8(buffer)?);

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    /// The name of the top-level instance described by the project.
    pub name: Option<String>,
   
    /// The tree of instances described by this project. Projects always
    /// describe at least one instance.
    pub tree: ProjectNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNode {
    #[serde(rename = "$className")]
    pub class_name: Option<String>,
    
    #[serde(rename = "$properties", default)]
    pub properties: HashMap<String, UnresolvedValue>,
    
    #[serde(flatten)]
    pub children: HashMap<String, ProjectNode>,
}

/// Instantiate a ProjectNode tree into a WeakDom (ported from Rojo)
fn instantiate(node: &ProjectNode, instance_name: &str) -> Result<WeakDom> {
    let root = instantiate_node(node, instance_name)?;
    Ok(WeakDom::new(root))
}

/// Convert a ProjectNode into an InstanceBuilder (ported from Rojo)
fn instantiate_node(node: &ProjectNode, name: &str) -> Result<InstanceBuilder> {
    // Determine class name - infer from known service names if not specified
    let class_name = if let Some(class) = &node.class_name {
        class.as_str()
    } else {
        // Try to infer from known services
        infer_class_from_name(name).unwrap_or("Folder")
    };
    
    let mut builder = InstanceBuilder::new(class_name).with_name(name);
    
    // Add properties with proper resolution
    for (key, unresolved) in &node.properties {
        match unresolved.clone().resolve(class_name, key) {
            Ok(variant) => {
                builder = builder.with_property(key, variant);
            }
            Err(e) => {
                eprintln!("Warning: Failed to resolve property {}.{}: {}", class_name, key, e);
            }
        }
    }
    
    // Add children
    for (child_name, child_node) in &node.children {
        match instantiate_node(child_node, child_name) {
            Ok(child_builder) => {
                builder = builder.with_child(child_builder);
            }
            Err(e) => {
                eprintln!("Warning: Failed to instantiate child {}: {}", child_name, e);
            }
        }
    }
    
    Ok(builder)
}

/// Infer a class name from an instance name (common service names)
fn infer_class_from_name(name: &str) -> Option<&'static str> {
    match name {
        "Workspace" => Some("Workspace"),
        "Players" => Some("Players"),
        "Lighting" => Some("Lighting"),
        "ReplicatedFirst" => Some("ReplicatedFirst"),
        "ReplicatedStorage" => Some("ReplicatedStorage"),
        "ServerScriptService" => Some("ServerScriptService"),
        "ServerStorage" => Some("ServerStorage"),
        "StarterGui" => Some("StarterGui"),
        "StarterPack" => Some("StarterPack"),
        "StarterPlayer" => Some("StarterPlayer"),
        "Teams" => Some("Teams"),
        "SoundService" => Some("SoundService"),
        "Chat" => Some("Chat"),
        "LocalizationService" => Some("LocalizationService"),
        "TestService" => Some("TestService"),
        _ => None,
    }
}
