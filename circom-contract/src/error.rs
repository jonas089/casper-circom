use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum CircomError{
    InvalidProof = 0
}
impl From<CircomError> for ApiError{
    fn from(e: CircomError) -> Self{
        ApiError::User(e as u16)
    }
}