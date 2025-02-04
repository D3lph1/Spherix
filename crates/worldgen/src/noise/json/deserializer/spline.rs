use crate::noise::density::density::DensityFunctions;
use crate::noise::density::spline::{ConstantSpline, MultipointSpline, Spline};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use anyhow::anyhow;
use serde_json::Value;

pub struct SplineDeserializer;

impl SplineDeserializer {
    fn do_deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<Spline> {
        let Value::Object(map) = json else {
            return Err(anyhow!("Expected Object, but given: {}", json))
        };

        // println!("{}", json);

        let coordinate = resolver.resolve_field(json, "coordinate")?;

        if !map.contains_key("points") {
            return Err(anyhow!("No \"points\" field"));
        }

        let mut locations = Vec::new();
        let mut derivatives = Vec::new();
        let mut values = Vec::new();

        let points = map.get("points").unwrap();
        let Value::Array(points) = points else {
            return Err(anyhow!("Expected Array, but given: {}", points));
        };

        for point in points {
            let Value::Object(point) = point else {
                return Err(anyhow!("Expected Object, but given: {}", point));
            };

            if !point.contains_key("location") {
                return Err(anyhow!("No \"location\" field"));
            }

            let location = point.get("location").unwrap();
            let location_casted = location.as_f64();
            if location_casted.is_none() {
                return Err(anyhow!("Field \"location\" must be f64, but given: {}", location));
            }

            locations.push(location_casted.unwrap());

            if !point.contains_key("derivative") {
                return Err(anyhow!("No \"derivative\" field"));
            }

            let derivative = point.get("derivative").unwrap();
            let derivative_casted = derivative.as_f64();
            if derivative_casted.is_none() {
                return Err(anyhow!("Field \"derivative\" must be f64, but given: {}", derivative));
            }

            derivatives.push(derivative_casted.unwrap());

            if !point.contains_key("value") {
                return Err(anyhow!("No \"value\" field"));
            }

            let value = point.get("value").unwrap();

            match value {
                Value::Number(num) => {
                    let num_casted = num.as_f64();
                    if num_casted.is_none() {
                        return Err(anyhow!("Value must be either f64 or Object, but given: {}", num));
                    }

                    values.push(
                        DensityFunctions::Spline(
                            Box::new(
                                Spline::Constant(
                                    ConstantSpline::new(num_casted.unwrap())
                                )
                            )
                        )
                    );
                }
                Value::Object(_) => {
                    values.push(
                        DensityFunctions::Spline(
                            Box::new(
                                self.do_deserialize(value, resolver)?
                            )
                        )
                    )
                }
                _ => {
                    return Err(anyhow!("Value must be either f64 or Object, but given: {}", value));
                }
            }
        }


        Ok(Spline::Multipoint(MultipointSpline::new(
            coordinate,
            locations,
            derivatives,
            values,
        )))
    }
}

impl Deserializer<DensityFunctions> for SplineDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        let Value::Object(map) = json else {
            return Err(anyhow!("Expected Object, but given: {}", json))
        };

        if !map.contains_key("spline") {
            return Err(anyhow!("No \"spline\" field"))
        }

        let spline = map.get("spline").unwrap();

        Ok(DensityFunctions::Spline(Box::new(self.do_deserialize(spline, resolver)?)))
    }
}
