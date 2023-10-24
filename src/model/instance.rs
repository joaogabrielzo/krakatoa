use nalgebra::Matrix4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub inverse_model_matrix: [[f32; 4]; 4],
    pub colour: [f32; 3],
}

impl InstanceData {
    pub fn from_matrix_and_colour(model_matrix: Matrix4<f32>, colour: [f32; 3]) -> InstanceData {
        InstanceData {
            model_matrix: model_matrix.into(),
            inverse_model_matrix: model_matrix.try_inverse().unwrap().into(),
            colour,
        }
    }
}
