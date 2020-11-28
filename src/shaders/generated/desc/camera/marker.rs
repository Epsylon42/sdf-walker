use super::Statement;

use std::num::ParseFloatError;

pub fn parse_marker(stmt: Statement) -> Result<(String, glm::Vec3), MarkerError> {
    if stmt.args.len() != 4 {
        return Err(MarkerError::WrongNumberOfArguments);
    }

    let name = stmt.args[0].clone();

    let x = stmt.args[1].parse()?;
    let y = stmt.args[2].parse()?;
    let z = stmt.args[3].parse()?;

    Ok((name, glm::Vec3::new(x, y, z)))
}

#[derive(Debug, thiserror::Error)]
pub enum MarkerError {
    #[error("Marker expects four arguments")]
    WrongNumberOfArguments,
    #[error("Failed to parse a number: {}", .0)]
    NumberParseError(#[from] ParseFloatError),
}
