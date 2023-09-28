use rustc_hash::FxHashMap;
use shared_utils::load_yaml::TempMatrix;

pub fn matrix_mapper(matrix: &TempMatrix) -> FxHashMap<u8, u8> {
    matrix
        .iter()
        .zip(matrix.iter().skip(1))
        .enumerate()
        .flat_map(|(index, (current_point, next_point))| {
            let initial_point = if index == 0 { 0 } else { current_point[0] };
            let final_point = if index == matrix.len() - 2 {
                100
            } else {
                next_point[0]
            };

            let interpolated_range = (initial_point..=final_point).map(move |temperature| {
                if initial_point == 0 && temperature < current_point[0] {
                    (temperature, 0)
                } else if final_point == 100 && temperature > next_point[0] {
                    (temperature, next_point[1])
                } else {
                    let fan_speed = linear_interpolation(
                        current_point[0] as f32,
                        current_point[1] as f32,
                        next_point[0] as f32,
                        next_point[1] as f32,
                        temperature as f32,
                    );
                    (temperature, fan_speed.round() as u8)
                }
            });

            interpolated_range
        })
        .collect()
}

fn linear_interpolation(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    y1 + (x - x1) * ((y2 - y1) / (x2 - x1))
}
