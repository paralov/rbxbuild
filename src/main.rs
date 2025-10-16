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
    
    // Check if there's an explicit Name property override
    let instance_name_override: Option<String> = node.properties.get("Name")
        .and_then(|name_value| {
            name_value.clone().resolve_unambiguous().ok()
        })
        .and_then(|variant| {
            if let rbx_dom_weak::types::Variant::String(s) = variant {
                Some(s.to_string())
            } else {
                None
            }
        });
    
    let instance_name = instance_name_override.as_deref().unwrap_or(name);
    
    let mut builder = InstanceBuilder::new(class_name).with_name(instance_name);
    
    // Add properties with proper resolution
    for (key, unresolved) in &node.properties {
        // Skip the "Name" property as it's already set via with_name()
        if key == "Name" {
            continue;
        }
        
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

#[cfg(test)]
mod tests {
    //! # Test Suite for JSON to XML Conversion
    //!
    //! This module contains comprehensive tests for the rojo-build-lite tool,
    //! which converts Rojo-style JSON project files to Roblox XML format.
    //!
    //! ## Test Coverage
    //!
    //! ### Basic Functionality
    //! - `test_simple_folder`: Verifies basic folder conversion
    //! - `test_nested_children`: Tests hierarchical instance structures
    //! - `test_folder_without_explicit_classname`: Tests default Folder inference
    //!
    //! ### Name Property Handling
    //! - `test_no_duplicate_name_property`: Ensures Name properties aren't duplicated when explicitly provided
    //! - `test_custom_name_override`: Verifies that explicit Name properties override the JSON key
    //!
    //! ### Service Inference
    //! - `test_service_inference`: Tests automatic class name inference for common Roblox services
    //!
    //! ### DataModel vs Model Files
    //! - `test_datamodel_place_file`: Verifies that DataModel children are written as root-level siblings
    //! - `test_model_file`: Ensures Model instances are properly included in the output
    //!
    //! ### Property Type Conversion
    //! - `test_boolean_property`: Boolean values
    //! - `test_number_properties`: Float/Int number properties
    //! - `test_vector3_property`: Vector3 arrays [x, y, z]
    //! - `test_color3_property`: Color3 arrays [r, g, b]
    //! - `test_cframe_property`: CFrame 12-element arrays
    //! - `test_enum_property`: String-based enum values
    //! - `test_multiple_properties`: Multiple properties on a single instance
    //!
    //! ### Scripts
    //! - `test_script_with_source`: Script instances with Source property
    //!
    //! ## Running Tests
    //!
    //! Run all tests with:
    //! ```bash
    //! cargo test
    //! ```
    //!
    //! Run a specific test:
    //! ```bash
    //! cargo test test_no_duplicate_name_property
    //! ```
    //!
    //! Run with verbose output:
    //! ```bash
    //! cargo test -- --nocapture
    //! ```
    
    use super::*;

    /// Helper function to convert JSON to XML string
    fn json_to_xml(json_str: &str) -> Result<String> {
        let project: Project = serde_json::from_str(json_str)?;
        let root_name = project.name.as_deref().unwrap_or("ROOT");
        let dom = instantiate(&project.tree, root_name)?;
        
        let mut buffer = Vec::new();
        let root_ref = dom.root_ref();
        let root_instance = dom.get_by_ref(root_ref).unwrap();
        
        let ids_to_write = if root_instance.class == "DataModel" {
            root_instance.children().to_vec()
        } else {
            vec![root_ref]
        };
        
        to_writer_default(&mut buffer, &dom, &ids_to_write)?;
        Ok(String::from_utf8(buffer)?)
    }

    #[test]
    fn test_simple_folder() {
        let json = r#"{
            "name": "TestProject",
            "tree": {
                "$className": "Folder"
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        assert!(xml.contains(r#"<Item class="Folder""#));
        assert!(xml.contains(r#"<string name="Name">TestProject</string>"#));
    }

    #[test]
    fn test_no_duplicate_name_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "DataModel",
                "ServerScriptService": {
                    "$className": "ServerScriptService",
                    "MyScript": {
                        "$className": "Script",
                        "$properties": {
                            "Name": "MyScript",
                            "Source": "print('hello')"
                        }
                    }
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Ensure no duplicate Name properties
        let name_count = xml.matches(r#"<string name="Name">MyScript</string>"#).count();
        assert_eq!(name_count, 1, "Name property should appear exactly once, not duplicated");
        
        // Verify the Source property is also present
        assert!(xml.contains(r#"<string name="Source">print('hello')</string>"#));
    }

    #[test]
    fn test_custom_name_override() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "DataModel",
                "Workspace": {
                    "$className": "Workspace",
                    "PartKey": {
                        "$className": "Part",
                        "$properties": {
                            "Name": "CustomPartName"
                        }
                    }
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Should use the custom name, not the key
        assert!(xml.contains(r#"<string name="Name">CustomPartName</string>"#));
        assert!(!xml.contains(r#"<string name="Name">PartKey</string>"#));
    }

    #[test]
    fn test_service_inference() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "DataModel",
                "Workspace": {
                    "MyPart": {
                        "$className": "Part"
                    }
                },
                "ReplicatedStorage": {
                    "MyFolder": {
                        "$className": "Folder"
                    }
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Should infer Workspace class
        assert!(xml.contains(r#"<Item class="Workspace""#));
        // Should infer ReplicatedStorage class
        assert!(xml.contains(r#"<Item class="ReplicatedStorage""#));
    }

    #[test]
    fn test_nested_children() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Folder",
                "Child1": {
                    "$className": "Folder",
                    "GrandChild": {
                        "$className": "Folder"
                    }
                },
                "Child2": {
                    "$className": "Folder"
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<string name="Name">Test</string>"#));
        assert!(xml.contains(r#"<string name="Name">Child1</string>"#));
        assert!(xml.contains(r#"<string name="Name">Child2</string>"#));
        assert!(xml.contains(r#"<string name="Name">GrandChild</string>"#));
    }

    #[test]
    fn test_script_with_source() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Script",
                "$properties": {
                    "Source": "print('Hello, World!')"
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<Item class="Script""#));
        assert!(xml.contains(r#"<string name="Source">print('Hello, World!')</string>"#));
    }

    #[test]
    fn test_vector3_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Size": [10, 20, 30]
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<Vector3 name="size">"#));
        assert!(xml.contains("<X>10</X>"));
        assert!(xml.contains("<Y>20</Y>"));
        assert!(xml.contains("<Z>30</Z>"));
    }

    #[test]
    fn test_boolean_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Anchored": true
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<bool name="Anchored">true</bool>"#));
    }

    #[test]
    fn test_number_properties() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Transparency": 0.5
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<float name="Transparency">0.5</float>"#));
    }

    #[test]
    fn test_color3_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Color": [1, 0.5, 0.25]
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Color3 gets serialized as Color3uint8 with a packed integer value
        assert!(xml.contains(r#"<Color3uint8 name="Color3uint8">"#));
    }

    #[test]
    fn test_datamodel_place_file() {
        let json = r#"{
            "name": "PlaceFile",
            "tree": {
                "$className": "DataModel",
                "Workspace": {
                    "$className": "Workspace"
                },
                "ServerScriptService": {
                    "$className": "ServerScriptService"
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // DataModel children should be at root level (siblings)
        assert!(!xml.contains(r#"<Item class="DataModel""#));
        assert!(xml.contains(r#"<Item class="Workspace""#));
        assert!(xml.contains(r#"<Item class="ServerScriptService""#));
    }

    #[test]
    fn test_model_file() {
        let json = r#"{
            "name": "MyModel",
            "tree": {
                "$className": "Model",
                "Part1": {
                    "$className": "Part"
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Model should be included (not like DataModel)
        assert!(xml.contains(r#"<Item class="Model""#));
        assert!(xml.contains(r#"<string name="Name">MyModel</string>"#));
    }

    #[test]
    fn test_cframe_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "CFrame": [0, 10, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<CoordinateFrame name="CFrame">"#));
        assert!(xml.contains("<X>0</X>"));
        assert!(xml.contains("<Y>10</Y>"));
        assert!(xml.contains("<Z>0</Z>"));
    }

    #[test]
    fn test_enum_property() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Material": "Grass"
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Material enum should be serialized as token
        assert!(xml.contains(r#"<token name="Material">"#));
    }

    #[test]
    fn test_multiple_properties() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "Part",
                "$properties": {
                    "Size": [4, 4, 4],
                    "Anchored": true,
                    "Transparency": 0.5,
                    "Color": [1, 0, 0]
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        assert!(xml.contains(r#"<Vector3 name="size">"#));
        assert!(xml.contains(r#"<bool name="Anchored">true</bool>"#));
        assert!(xml.contains(r#"<float name="Transparency">0.5</float>"#));
        assert!(xml.contains(r#"<Color3uint8 name="Color3uint8">"#));
    }

    #[test]
    fn test_folder_without_explicit_classname() {
        let json = r#"{
            "name": "Test",
            "tree": {
                "$className": "DataModel",
                "Workspace": {
                    "$className": "Workspace",
                    "SomeFolder": {
                        "InnerFolder": {}
                    }
                }
            }
        }"#;
        
        let xml = json_to_xml(json).expect("Failed to convert JSON to XML");
        
        // Should default to Folder when no className is specified
        assert!(xml.contains(r#"<string name="Name">SomeFolder</string>"#));
        assert!(xml.contains(r#"<string name="Name">InnerFolder</string>"#));
    }
}
