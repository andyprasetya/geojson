// Copyright 2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::json::{JsonObject, JsonValue};
use crate::{feature, Bbox, Error, FeatureBase, GeometryBase, Position};

pub fn expect_type(value: &mut JsonObject) -> Result<String, Error> {
    let prop = expect_property(value, "type")?;
    expect_string(prop)
}

pub fn expect_string(value: JsonValue) -> Result<String, Error> {
    match value {
        JsonValue::String(s) => Ok(s),
        _ => Err(Error::ExpectedStringValue),
    }
}

pub fn expect_f64(value: &JsonValue) -> Result<f64, Error> {
    match value.as_f64() {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedF64Value),
    }
}

pub fn expect_array(value: &JsonValue) -> Result<&Vec<JsonValue>, Error> {
    match value.as_array() {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedArrayValue),
    }
}

fn expect_property(obj: &mut JsonObject, name: &'static str) -> Result<JsonValue, Error> {
    match obj.remove(name) {
        Some(v) => Ok(v),
        None => Err(Error::ExpectedProperty(name.to_string())),
    }
}

fn expect_owned_array(value: JsonValue) -> Result<Vec<JsonValue>, Error> {
    match value {
        JsonValue::Array(v) => Ok(v),
        _ => Err(Error::ExpectedArrayValue),
    }
}

fn expect_owned_object(value: JsonValue) -> Result<JsonObject, Error> {
    match value {
        JsonValue::Object(o) => Ok(o),
        _ => Err(Error::ExpectedObjectValue),
    }
}

pub fn get_coords_value(object: &mut JsonObject) -> Result<JsonValue, Error> {
    expect_property(object, "coordinates")
}

/// Used by FeatureCollection, Feature, Geometry
pub fn get_bbox(object: &mut JsonObject) -> Result<Option<Bbox>, Error> {
    let bbox_json = match object.remove("bbox") {
        Some(b) => b,
        None => return Ok(None),
    };
    let bbox_array = match bbox_json {
        JsonValue::Array(a) => a,
        _ => return Err(Error::BboxExpectedArray),
    };
    let bbox = bbox_array
        .into_iter()
        .map(|i| i.as_f64().ok_or(Error::BboxExpectedNumericValues))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(bbox))
}

/// Used by FeatureCollection, Feature, Geometry
pub fn get_foreign_members(object: JsonObject) -> Result<Option<JsonObject>, Error> {
    if object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(object))
    }
}

/// Used by Feature
pub fn get_properties(object: &mut JsonObject) -> Result<Option<JsonObject>, Error> {
    let properties = expect_property(object, "properties")?;
    match properties {
        JsonValue::Object(x) => Ok(Some(x)),
        JsonValue::Null => Ok(None),
        _ => Err(Error::PropertiesExpectedObjectOrNull),
    }
}

/// Retrieve a single Position from the value of the "coordinates" key
///
/// Used by ValueBase::Point
pub fn get_coords_one_pos<P: Position>(object: &mut JsonObject) -> Result<P, Error> {
    let coords_json = get_coords_value(object)?;
    Position::from_json_value(&coords_json)
}

/// Retrieve a one dimensional Vec of Positions from the value of the "coordinates" key
///
/// Used by Value::MultiPoint and Value::LineString
pub fn get_coords_1d_pos<P: Position>(object: &mut JsonObject) -> Result<Vec<P>, Error> {
    let coords_json = get_coords_value(object)?;
    json_to_1d_positions(&coords_json)
}

/// Retrieve a two dimensional Vec of Positions from the value of the "coordinates" key
///
/// Used by Value::MultiLineString and Value::Polygon
pub fn get_coords_2d_pos<P: Position>(object: &mut JsonObject) -> Result<Vec<Vec<P>>, Error> {
    let coords_json = get_coords_value(object)?;
    json_to_2d_positions(&coords_json)
}

/// Retrieve a three dimensional Vec of Positions from the value of the "coordinates" key
///
/// Used by Value::MultiPolygon
pub fn get_coords_3d_pos<P: Position>(object: &mut JsonObject) -> Result<Vec<Vec<Vec<P>>>, Error> {
    let coords_json = get_coords_value(object)?;
    json_to_3d_positions(&coords_json)
}

/// Used by Value::GeometryCollection
pub fn get_geometries<Pos: Position>(object: &mut JsonObject) -> Result<Vec<GeometryBase<Pos>>, Error> {
    let geometries_json = expect_property(object, "geometries")?;
    let geometries_array = expect_owned_array(geometries_json)?;
    let mut geometries = Vec::with_capacity(geometries_array.len());
    for json in geometries_array {
        let obj = expect_owned_object(json)?;
        let geometry = GeometryBase::from_json_object(obj)?;
        geometries.push(geometry);
    }
    Ok(geometries)
}

/// Used by Feature
pub fn get_id(object: &mut JsonObject) -> Result<Option<feature::Id>, Error> {
    match object.remove("id") {
        Some(JsonValue::Number(x)) => Ok(Some(feature::Id::Number(x))),
        Some(JsonValue::String(s)) => Ok(Some(feature::Id::String(s))),
        Some(_) => Err(Error::FeatureInvalidIdentifierType),
        None => Ok(None),
    }
}

/// Used by Feature
pub fn get_geometry<P: Position>(
    object: &mut JsonObject,
) -> Result<Option<GeometryBase<P>>, Error> {
    let geometry = expect_property(object, "geometry")?;
    match geometry {
        JsonValue::Object(x) => {
            let geometry_object = GeometryBase::from_json_object(x)?;
            Ok(Some(geometry_object))
        }
        JsonValue::Null => Ok(None),
        _ => Err(Error::FeatureInvalidGeometryValue),
    }
}

/// Used by FeatureCollection
pub fn get_features<P: Position>(object: &mut JsonObject) -> Result<Vec<FeatureBase<P>>, Error> {
    let prop = expect_property(object, "features")?;
    let features_json = expect_owned_array(prop)?;
    let mut features = Vec::with_capacity(features_json.len());
    for feature in features_json {
        let feature = expect_owned_object(feature)?;
        let feature: FeatureBase<P> = FeatureBase::from_json_object(feature)?;
        features.push(feature);
    }
    Ok(features)
}

fn json_to_1d_positions<P: Position>(json: &JsonValue) -> Result<Vec<P>, Error> {
    let coords_array = expect_array(json)?;
    let mut coords = Vec::with_capacity(coords_array.len());
    for item in coords_array {
        coords.push(P::from_json_value(item)?);
    }
    Ok(coords)
}

fn json_to_2d_positions<P: Position>(json: &JsonValue) -> Result<Vec<Vec<P>>, Error> {
    let coords_array = expect_array(json)?;
    let mut coords = Vec::with_capacity(coords_array.len());
    for item in coords_array {
        coords.push(json_to_1d_positions(item)?);
    }
    Ok(coords)
}

fn json_to_3d_positions<P: Position>(json: &JsonValue) -> Result<Vec<Vec<Vec<P>>>, Error> {
    let coords_array = expect_array(json)?;
    let mut coords = Vec::with_capacity(coords_array.len());
    for item in coords_array {
        coords.push(json_to_2d_positions(item)?);
    }
    Ok(coords)
}
