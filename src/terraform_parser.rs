use std::collections::HashMap;
use ruml::{Entity, EntityType};

pub(crate) fn parse_terraform(code: &str) -> Vec<Entity> {
    // Parse the Terraform code using the hypothetical library
    let parsed_code = terraform_parser::parse(code).expect("Failed to parse Terraform code");

    // Convert parsed Terraform to a vector of Entity objects
    let entities = convert_to_entities(parsed_code);

    entities
}

// Function to convert parsed Terraform data to Entity objects
fn convert_to_entities(parsed_code: HashMap<String, terraform_parser::TerraformBlock>) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (name, block) in parsed_code {
        match block {
            terraform_parser::TerraformBlock::Resource(resource) => {
                let fields = resource
                    .attributes
                    .iter()
                    .map(|(key, value)| Entity::new(EntityType::Field(key.clone()), value, vec![]))
                    .collect();
                let entity = Entity::new(EntityType::Struct, &name, fields);
                entities.push(entity);
            }
            terraform_parser::TerraformBlock::Module(module) => {
                let fields = module
                    .variables
                    .iter()
                    .map(|(key, value)| Entity::new(EntityType::Field(key.clone()), value, vec![]))
                    .collect();
                let entity = Entity::new(EntityType::Struct, &name, fields);
                entities.push(entity);
            }
            terraform_parser::TerraformBlock::Variable(variable) => {
                let fields = vec![
                    Entity::new(EntityType::Field("default".to_string()), variable.default.as_deref().unwrap_or(""), vec![]),
                    Entity::new(EntityType::Field("description".to_string()), variable.description.as_deref().unwrap_or(""), vec![]),
                ];
                let entity = Entity::new(EntityType::Struct, &name, fields);
                entities.push(entity);
            }
            _ => {}
        }
    }

    entities
}

mod terraform_parser {
    use std::collections::HashMap;
    use regex::Regex;

    #[derive(Debug)]
    pub enum TerraformBlock {
        Resource(Resource),
        Module(Module),
        Variable(Variable),
        // Other block types...
    }

    #[derive(Debug)]
    pub struct Resource {
        pub type_name: String,
        pub name: String,
        pub attributes: HashMap<String, String>,
    }

    #[derive(Debug)]
    pub struct Module {
        pub name: String,
        pub source: String,
        pub variables: HashMap<String, String>,
    }

    #[derive(Debug)]
    pub struct Variable {
        pub name: String,
        pub default: Option<String>,
        pub description: Option<String>,
    }

    pub fn parse(code: &str) -> Result<HashMap<String, TerraformBlock>, String> {
        let mut blocks = HashMap::new();

        // Parse resource blocks
        let resource_regex = r#"resource\s+"(\w+)"\s+"(\w+)"\s*\{([^}]*)\}"#;
        for cap in Regex::new(resource_regex).unwrap().captures_iter(code) {
            let type_name = cap[1].to_string();
            let name = cap[2].to_string();
            let attributes_str = &cap[3];
            let attributes = parse_attributes(attributes_str);

            let resource = Resource {
                type_name,
                name: name.clone(), // Clone the name before using it
                attributes,
            };
            blocks.insert(format!("resource.{}", name), TerraformBlock::Resource(resource));
        }

        // Parse module blocks
        let module_regex = r#"module\s+"(\w+)"\s*\{([^}]*)\}"#;
        for cap in Regex::new(module_regex).unwrap().captures_iter(code) {
            let name = cap[1].to_string();
            let content_str = &cap[2];
            let source = parse_attribute(content_str, "source")?;
            let variables = parse_variables(content_str);

            let module = Module {
                name: name.clone(), // Clone the name before using it
                source,
                variables,
            };
            blocks.insert(format!("module.{}", name), TerraformBlock::Module(module));
        }

        // Parse variable blocks
        let variable_regex = r#"variable\s+"(\w+)"\s*\{([^}]*)\}"#;
        for cap in Regex::new(variable_regex).unwrap().captures_iter(code) {
            let name = cap[1].to_string();
            let content_str = &cap[2];
            let default = parse_attribute(content_str, "default").ok();
            let description = parse_attribute(content_str, "description").ok();

            let variable = Variable {
                name: name.clone(), // Clone the name before using it
                default,
                description,
            };
            blocks.insert(format!("variable.{}", name), TerraformBlock::Variable(variable));
        }

        Ok(blocks)
    }

    fn parse_attributes(attributes_str: &str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        let attribute_regex = r#"(\w+)\s*=\s*"([^"]*)""#;
        for cap in Regex::new(attribute_regex).unwrap().captures_iter(attributes_str) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            attributes.insert(key, value);
        }
        attributes
    }

    fn parse_attribute(content_str: &str, attribute_name: &str) -> Result<String, String> {
        let attribute_regex = format!(r#"\b{}\s*=\s*"([^"]*)""#, attribute_name);
        if let Some(cap) = Regex::new(&attribute_regex).unwrap().captures(content_str) {
            Ok(cap[1].to_string())
        } else {
            Err(format!("Attribute {} not found", attribute_name))
        }
    }

    fn parse_variables(content_str: &str) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        let variable_regex = r#"(\w+)\s*=\s*"([^"]*)""#;
        for cap in Regex::new(variable_regex).unwrap().captures_iter(content_str) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            variables.insert(key, value);
        }
        variables
    }
}
