mod instance;
mod model;
mod vertex;

pub use instance::InstanceData;
pub use model::Model;
pub use vertex::VertexData;

#[derive(Debug, Clone)]
pub struct InvalidHandle;
impl std::fmt::Display for InvalidHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid handle")
    }
}
impl std::error::Error for InvalidHandle {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
