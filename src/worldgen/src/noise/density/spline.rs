use spherix_math::vector::Vector3;
use std::fmt::{Debug, Formatter};

use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use crate::noise::math::lerp;

#[derive(Clone)]
pub enum Spline {
    Constant(ConstantSpline),
    Multipoint(MultipointSpline),
}

impl Debug for Spline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Spline::Constant(spline) => spline.fmt(f),
            Spline::Multipoint(spline) => spline.fmt(f),
        }
    }
}

impl DensityFunction for Spline {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        match self {
            Spline::Constant(spline) => spline.sample(at, ctx),
            Spline::Multipoint(spline) => spline.sample(at, ctx),
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            Spline::Constant(spline) => spline.min_value(),
            Spline::Multipoint(spline) => spline.min_value(),
        }
    }

    fn max_value(&self) -> f64 {
        match self {
            Spline::Constant(spline) => spline.max_value(),
            Spline::Multipoint(spline) => spline.max_value(),
        }
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        match self {
            Spline::Constant(spline) => spline.map(mapper),
            Spline::Multipoint(spline) => spline.map(mapper)
        }
    }
}

#[derive(Clone)]
pub struct ConstantSpline {
    value: f64,
}

impl ConstantSpline {
    #[inline]
    pub fn new(value: f64) -> Self {
        Self {
            value
        }
    }
}

impl Debug for ConstantSpline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConstantSpline ({})", self.value)
    }
}

impl DensityFunction for ConstantSpline {
    #[inline]
    fn sample(&self, _: Vector3, _: &mut DensityFunctionContext) -> f64 {
        self.value
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Spline(
                Box::new(
                    Spline::Constant(self)
                )
            )
        )
    }
}

/// Represents multipoint cubic spline
#[derive(Clone)]
pub struct MultipointSpline {
    coordinate: DensityFunctions,
    locations: Vec<f64>,
    derivatives: Vec<f64>,
    values: Vec<DensityFunctions>,
    min_value: f64,
    max_value: f64,
}

impl MultipointSpline {
    pub fn new(coordinate: DensityFunctions, locations: Vec<f64>, derivatives: Vec<f64>, values: Vec<DensityFunctions>) -> MultipointSpline {
        if locations.len() == values.len() && locations.len() == derivatives.len() {
            if locations.len() == 0 {
                panic!("Cannot create a multipoint spline with no points");
            }
        } else {
            panic!("All lengths must be equal, got: {} {} {}", locations.len(), values.len(), derivatives.len());
        }

        // The following code calculates the overall minimum and maximum values for the
        // spline, taking into account possible linear extensions beyond the provided
        // locations and the values at each spline.

        let segment_count = locations.len() - 1;
        let mut global_min_value = f64::INFINITY;
        let mut global_max_value = f64::NEG_INFINITY;

        let input_min_value = coordinate.min_value();
        let input_max_value = coordinate.max_value();

        if input_min_value < locations[0] {
            let linear_min_extend = Self::linear_extend(input_min_value, &locations, &derivatives, values[0].min_value(), 0);
            let linear_max_extend = Self::linear_extend(input_min_value, &locations, &derivatives, values[0].max_value(), 0);

            global_min_value = global_min_value.min(linear_min_extend.min(linear_max_extend));
            global_max_value = global_max_value.max(linear_min_extend.max(linear_max_extend));
        }

        if input_max_value > locations[segment_count] {
            let linear_min_extend = Self::linear_extend(input_max_value, &locations, &derivatives, values[0].min_value(), segment_count);
            let linear_max_extend = Self::linear_extend(input_max_value, &locations, &derivatives, values[0].max_value(), segment_count);
            global_min_value = global_min_value.min(linear_min_extend.min(linear_max_extend));
            global_max_value = global_max_value.max(linear_min_extend.max(linear_max_extend));
        }

        for value in &values {
            global_min_value = global_min_value.min(value.min_value());
            global_max_value = global_max_value.max(value.max_value());
        }

        for j in 0..segment_count {
            let x1 = locations[j];
            let x2 = locations[j + 1];
            let segment_length = x2 - x1;
            let spline1 = &values[j];
            let spline2 = &values[j + 1];
            let min_y_1 = spline1.min_value();
            let max_y_1 = spline1.max_value();
            let min_y_2 = spline2.min_value();
            let max_y_2 = spline2.max_value();
            let slope1 = derivatives[j];
            let slope2 = derivatives[j + 1];

            if slope1 != 0.0 || slope2 != 0.0 {
                let slope_product_1 = slope1 * segment_length;
                let slope_product_2 = slope2 * segment_length;
                let min_combined_y = min_y_1.min(min_y_2);
                let max_combined_y = max_y_1.max(max_y_2);
                let extreme_y_1 = slope_product_1 - max_y_2 + min_y_1;
                let extreme_y_2 = slope_product_1 - min_y_2 + max_y_1;
                let extreme_y_3 = -slope_product_2 + min_y_2 - max_y_1;
                let extreme_y_4 = -slope_product_2 + max_y_2 - min_y_1;
                let min_extreme_y = extreme_y_1.min(extreme_y_3);
                let max_extreme_y = extreme_y_2.max(extreme_y_4);
                global_min_value = global_min_value.min(min_combined_y + 0.25 * min_extreme_y);
                global_max_value = global_max_value.max(max_combined_y + 0.25 * max_extreme_y);
            }
        }

        MultipointSpline {
            coordinate,
            locations,
            values,
            derivatives,
            min_value: global_min_value,
            max_value: global_max_value,
        }
    }

    /// Performs a linear extension (or extrapolation) of a value based on the provided parameters.
    ///
    /// `f2`: A floating-point value representing some point on the x-axis where the linear
    /// extension is evaluated.
    ///
    /// `locations`: A reference to a vector of floating-point values representing the
    /// x-coordinates of known data points.
    ///
    /// `derivatives`: A reference to a vector of floating-point values representing the
    /// derivatives (slopes) at the corresponding x-coordinates in `locations`.
    ///
    /// `value`: The known value (y-coordinate) at the x-coordinate given by `locations[index]`.
    ///
    /// `index`: The index specifying the position in the locations and derivatives vectors to
    /// be used for the extension.
    #[inline]
    fn linear_extend(f2: f64, locations: &Vec<f64>, derivatives: &Vec<f64>, value: f64, index: usize) -> f64 {
        let slope = derivatives[index];
        if slope == 0.0 {
            value
        } else {
            value + slope * (f2 - locations[index])
        }
    }

    #[inline]
    fn find_interval_start(locations: &[f64], f: f64) -> isize {
        locations.binary_search_by(|a| a.total_cmp(&f)).unwrap_or_else(|index| index) as isize - 1
    }
}

impl Debug for MultipointSpline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultipointSpline (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for MultipointSpline {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let f = self.coordinate.sample(at, ctx);
        // Find the index of the interval start for the given input value
        let start_interval_index = Self::find_interval_start(&self.locations, f);
        let last_interval_index = self.locations.len() - 1;
        
        if start_interval_index < 0 {
            // If the input is before the first interval, use linear extension
            Self::linear_extend(
                f,
                &self.locations,
                &self.derivatives,
                self.values[0].sample(at, ctx),
                0,
            )
        } else if start_interval_index as usize == last_interval_index {
            // If the input is after the last interval, use linear extension
            Self::linear_extend(
                f,
                &self.locations,
                &self.derivatives,
                self.values[last_interval_index].sample(at, ctx),
                last_interval_index,
            )
        } else {
            let start_interval_index = start_interval_index as usize;
            // If the input is within the intervals, perform spline interpolation
            let x1 = self.locations[start_interval_index];
            let x2 = self.locations[start_interval_index + 1];
            // Calculate the normalized position within the interval
            let normalized_x = (f - x1) / (x2 - x1);
            // Get the slope at the current and next control points
            let current_slope = self.derivatives[start_interval_index];
            let next_slope = self.derivatives[start_interval_index + 1];
            // Evaluate the spline at the beginning of the current interval
            let y1 = self.values[start_interval_index].sample(at, ctx);
            let y2 = self.values[start_interval_index + 1].sample(at, ctx);
            // Intermediate values
            let t1 = current_slope * (x2 - x1) - (y2 - y1);
            let t2 = -next_slope * (x2 - x1) + (y2 - y1);

            lerp(normalized_x, y1, y2) + normalized_x * (1.0 - normalized_x) * lerp(normalized_x, t1, t2)
        }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Spline(
                Box::new(
                    Spline::Multipoint(
                        MultipointSpline::new(
                            self.coordinate.map(mapper),
                            self.locations,
                            self.derivatives,
                            self.values
                                .into_iter()
                                .map(|spline| spline.map(mapper))
                                .collect()
                        )
                    )
                )
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use spherix_math::vector::Vector3;
    use spherix_util::assert_f64_eq;

    use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions};
    use crate::noise::density::spline::{ConstantSpline, MultipointSpline, Spline};

    #[test]
    fn multipoint_spline_all() {
        let spline = MultipointSpline::new(
            DensityFunctions::Spline(Box::new(Spline::Constant(ConstantSpline::new(1.2)))),
            vec![
                -0.5, 0.4, 1.0,
            ],
            vec![
                0.2, 0.73, -0.6,
            ],
            vec![
                DensityFunctions::Spline(Box::new(Spline::Constant(ConstantSpline::new(0.84)))),
                DensityFunctions::Spline(Box::new(Spline::Constant(ConstantSpline::new(-0.586)))),
                DensityFunctions::Spline(Box::new(Spline::Constant(ConstantSpline::new(-0.002)))),
            ],
        );

        assert_f64_eq!(-1.10675, spline.min_value(), 5);
        assert_f64_eq!(1.2414999, spline.max_value(), 5);
        assert_f64_eq!(-0.12200003, spline.sample(Vector3::new(1, 2, -4), &mut DensityFunctionContext::default()), 5);
        assert_f64_eq!(-0.12200003, spline.sample(Vector3::new(743, -50, 18403), &mut DensityFunctionContext::default()), 5);
    }
}
