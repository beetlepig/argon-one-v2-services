use crate::load_yaml::{TempMatrixRKYV, TempMatrixYAML};
use std::cmp::Ordering;
use std::collections::HashMap;

pub fn matrix_mapper(mut matrix: TempMatrixYAML) -> TempMatrixRKYV {
    let mut mapped_temperatures = HashMap::with_capacity(101);
    let len = matrix.len();

    matrix.sort_by(|a, b| {
        let cmp = a[0].cmp(&b[0]);

        if cmp == Ordering::Equal {
            a[1].cmp(&b[1])
        } else {
            cmp
        }
    });

    let matrix_iter = matrix.iter().zip(matrix.iter().skip(1)).enumerate();
    for (index, (current_point, next_point)) in matrix_iter {
        let initial_point = if index == 0 { 0 } else { current_point[0] };
        let final_point = if index == len - 2 { 100 } else { next_point[0] };

        for temperature in initial_point..=final_point {
            let fan_speed = if initial_point == 0 && temperature < current_point[0] {
                0
            } else if final_point == 100 && temperature > next_point[0] {
                next_point[1]
            } else {
                linear_interpolation(
                    current_point[0] as f32,
                    current_point[1] as f32,
                    next_point[0] as f32,
                    next_point[1] as f32,
                    temperature as f32,
                )
                .round() as u8
            };

            mapped_temperatures.insert(temperature, fan_speed);
        }
    }

    mapped_temperatures
}

#[inline]
fn linear_interpolation(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    y1 + (x - x1) * ((y2 - y1) / (x2 - x1))
}
