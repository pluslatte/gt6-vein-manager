use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub name: Option<String>,
}

impl SearchQuery {
    pub fn has_name_filter(&self) -> bool {
        self.name
            .as_ref()
            .is_some_and(|name| !name.trim().is_empty())
    }

    pub fn get_name_filter(&self) -> Option<&str> {
        self.name
            .as_ref()
            .filter(|name| !name.trim().is_empty())
            .map(|s| s.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct AddVeinForm {
    pub name: String,
    pub x_coord: String,
    pub y_coord: String,
    pub z_coord: String,
    pub notes: Option<String>,
    pub confirmed: Option<bool>,
    pub depleted: Option<bool>,
}

impl AddVeinForm {
    pub fn parse_x_coord(&self) -> Result<i32, std::num::ParseIntError> {
        self.x_coord.parse::<i32>()
    }

    pub fn parse_y_coord(&self) -> Result<Option<i32>, std::num::ParseIntError> {
        if self.y_coord.trim().is_empty() {
            Ok(None)
        } else {
            self.y_coord.parse::<i32>().map(Some)
        }
    }

    pub fn parse_z_coord(&self) -> Result<i32, std::num::ParseIntError> {
        self.z_coord.parse::<i32>()
    }

    pub fn is_confirmed(&self) -> bool {
        self.confirmed.unwrap_or(false)
    }

    pub fn is_depleted(&self) -> bool {
        self.depleted.unwrap_or(false)
    }
}
