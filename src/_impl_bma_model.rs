use crate::bma_model::*;
use crate::enums::VariableType;
use crate::json_model::JsonBmaModel;
use crate::traits::{JsonSerde, XmlSerde};
use crate::xml_model::XmlBmaModel;
use std::collections::HashMap;

impl<'de> JsonSerde<'de> for BmaModel {
    fn to_json_str(&self) -> String {
        todo!()
    }

    fn to_pretty_json_str(&self) -> String {
        todo!()
    }

    fn from_json_str(json_str: &'de str) -> Result<Self, String> {
        let json_model: JsonBmaModel = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        let model = BmaModel::from(json_model);
        Ok(model)
    }
}

impl<'de> XmlSerde<'de> for BmaModel {
    fn to_xml_str(&self) -> String {
        todo!()
    }

    fn to_pretty_xml_str(&self) -> String {
        todo!()
    }

    fn from_xml_str(xml_str: &'de str) -> Result<Self, String> {
        let xml_model: XmlBmaModel = serde_xml_rs::from_str(xml_str).map_err(|e| e.to_string())?;
        let model = BmaModel::from(xml_model);
        Ok(model)
    }
}

impl From<JsonBmaModel> for BmaModel {
    fn from(json_model: JsonBmaModel) -> Self {
        // Create a mapping from variable IDs to their names from the layout
        let layout_var_names: HashMap<u32, String> = json_model
            .layout
            .as_ref()
            .map(|layout| {
                layout
                    .variables
                    .iter()
                    .filter(|layout_var| layout_var.name.is_some())
                    .map(|layout_var| (layout_var.id, layout_var.name.clone().unwrap()))
                    .collect()
            })
            .unwrap_or_default();

        // Convert the model
        let model = Model {
            name: json_model.model.name,
            variables: json_model
                .model
                .variables
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var
                        .name
                        .unwrap_or(layout_var_names.get(&var.id).cloned().unwrap_or_default()), // Use the name from layout
                    variable_type: json_model
                        .layout
                        .as_ref()
                        .and_then(|layout| layout.variables.iter().find(|v| v.id == var.id))
                        .map(|v| v.r#type)
                        .unwrap_or(VariableType::Default), // Use the type from layout if available
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: var.formula,
                })
                .collect(),
            relationships: json_model
                .model
                .relationships
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable,
                    to_variable: rel.to_variable,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| Layout {
                variables: layout
                    .variables
                    .into_iter()
                    .map(|var| LayoutVariable {
                        id: var.id,
                        container_id: var.container_id,
                        position_x: var.position_x,
                        position_y: var.position_y,
                        cell_x: var.cell_x,
                        cell_y: var.cell_y,
                        angle: var.angle,
                    })
                    .collect(),
                containers: layout
                    .containers
                    .into_iter()
                    .map(|container| Container {
                        id: container.id,
                        name: container.name,
                        size: container.size,
                        position_x: container.position_x,
                        position_y: container.position_y,
                    })
                    .collect(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            })
            .unwrap_or_else(|| Layout {
                variables: vec![],
                containers: vec![],
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            });

        // metadata not present in JsonBmaModel
        let metadata = HashMap::new();

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}

impl From<XmlBmaModel> for BmaModel {
    fn from(xml_model: XmlBmaModel) -> Self {
        // Convert the model
        let model = Model {
            name: xml_model.name,
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var.name,
                    variable_type: var.r#type,
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: var.formula,
                })
                .collect(),
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable_id,
                    to_variable: rel.to_variable_id,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = Layout {
            variables: xml_model
                .variables
                .variable
                .into_iter()
                .map(|var| LayoutVariable {
                    id: var.id,
                    container_id: var.container_id,
                    position_x: var.position_x,
                    position_y: var.position_y,
                    cell_x: Some(var.cell_x),
                    cell_y: Some(var.cell_y),
                    angle: var.angle,
                })
                .collect(),
            containers: xml_model
                .containers
                .container
                .into_iter()
                .map(|container| Container {
                    id: container.id,
                    name: Some(container.name),
                    size: container.size,
                    position_x: container.position_x,
                    position_y: container.position_y,
                })
                .collect(),
            zoom_level: Some(xml_model.layout.zoom_level),
            pan_x: Some(xml_model.layout.pan_x),
            pan_y: Some(xml_model.layout.pan_y),
        };

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        metadata.insert("biocheck_version".to_string(), xml_model.biocheck_version);
        metadata.insert("description".to_string(), xml_model.description);
        metadata.insert("created_date".to_string(), xml_model.created_date);
        metadata.insert("modified_date".to_string(), xml_model.modified_date);

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}
