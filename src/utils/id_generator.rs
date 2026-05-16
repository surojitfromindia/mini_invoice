use nanoid::nanoid;

pub struct IdGenerator;
impl IdGenerator {
    pub fn get_user_id() -> String {
        nanoid!(12, &nanoid::alphabet::HEX_UPPERCASE)
    }
    pub fn get_organization_id() -> String {
        nanoid!(12, &nanoid::alphabet::HEX_UPPERCASE)
    }
}
