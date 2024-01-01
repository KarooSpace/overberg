use super::error::Error as MusicError;
use std::path::Path;

pub struct Current {
	frequency: f64,
	theta: f64,
	phi: f64,
	element: u16,
	current: faer::complex_native::c64,
}

pub struct Currents {
	theta_currents: Vec<Current>,
	phi_currents: Vec<Current>,
}

impl InputData {
	fn new_from_matfile<P: AsRef<Path>>(
		filename: &P,
		theta_currents: &str,
		phi_currents: &str,
		phi_angles: &str,
		theta_angles: &str,
		frequencies: &str,
	) -> Result<Self, MusicError> {
		let file = std::fs::File::open(filename)?;
		let mat_file = matfile::MatFile::parse(file)?;
		let theta_currents =
			mat_file
				.find_by_name(theta_currents)
				.ok_or(MusicError::IO(format!(
					"array with name {} not found",
					theta_currents
				)))?;
		let phi_currents = mat_file
			.find_by_name(phi_currents)
			.ok_or(MusicError::IO(format!(
				"array with name {} not found",
				phi_currents
			)))?;

		Ok(Currents {
			theta_currents,
			phi_currents,
		})
	}
}

impl From<std::io::Error> for MusicError {
	fn from(value: std::io::Error) -> Self {
		Self::IO(value.to_string())
	}
}

impl From<matfile::Error> for MusicError {
	fn from(value: matfile::Error) -> Self {
		Self::IO(value.to_string())
	}
}
