use std::{
    io::Read,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::Deserialize;

use crate::{Error, ResourceReader};

/// A World is a list of maps files or regex patterns that define a layout of TMX maps.
/// You can use the loader to further load the maps defined by the world.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct World {
    /// The path first used in a [`ResourceReader`] to load this world.
    #[serde(skip_deserializing)]
    pub source: PathBuf,
    /// The [`WorldMap`]s defined by the world file.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub maps: Vec<WorldMap>,
    /// Optional regex pattern to load maps.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub patterns: Vec<WorldPattern>,
}

impl World {
    /// Utility function to test a single path against all defined patterns.
    /// Returns a parsed [`WorldMap`] on the first matched pattern or an error if no patterns match.
    pub fn match_path(&self, path: impl AsRef<Path>) -> Result<WorldMap, Error> {
        let path_str = path.as_ref().to_str().expect("obtaining valid UTF-8 path");

        for pattern in self.patterns.iter() {
            match pattern.match_path_impl(path_str) {
                Ok(world_map) => return Ok(world_map),
                // We ignore match errors here as the path may be matched by another pattern.
                Err(Error::NoMatchFound { .. }) => continue,
                Err(err) => return Err(err),
            }
        }

        Err(Error::NoMatchFound {
            path: path_str.to_owned(),
        })
    }

    /// Utility function to test a vec of filenames against all defined patterns.
    /// Returns a vec of results with the parsed [`WorldMap`]s if it matches the pattern.
    pub fn match_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<Result<WorldMap, Error>> {
        paths.iter().map(|path| self.match_path(path)).collect()
    }
}

/// A WorldMap provides the information for a map in the world and its layout.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct WorldMap {
    /// The filename of the tmx map.
    #[serde(rename = "fileName")]
    pub filename: String,
    /// The x position of the map.
    pub x: i32,
    /// The y position of the map.
    pub y: i32,
    /// The optional width of the map.
    pub width: Option<i32>,
    /// The optional height of the map.
    pub height: Option<i32>,
}

/// A WorldPattern defines a regex pattern to automatically determine which maps to load and how to lay them out.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorldPattern {
    /// The regex pattern to match against filenames.
    /// The first two capture groups should be the x integer and y integer positions.
    #[serde(with = "serde_regex")]
    pub regexp: Regex,
    /// The multiplier for the x position.
    pub multiplier_x: i32,
    /// The multiplier for the y position.
    pub multiplier_y: i32,
    /// The offset for the x position.
    pub offset_x: i32,
    /// The offset for the y position.
    pub offset_y: i32,
}

impl PartialEq for WorldPattern {
    fn eq(&self, other: &Self) -> bool {
        self.multiplier_x == other.multiplier_x
            && self.multiplier_y == other.multiplier_y
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.regexp.to_string() == other.regexp.to_string()
    }
}

impl WorldPattern {
    /// Utility function to test a path against this pattern.
    /// Returns a parsed [`WorldMap`] on the first matched pattern or an error if no patterns match.
    pub fn match_path(&self, path: impl AsRef<Path>) -> Result<WorldMap, Error> {
        let path_str = path.as_ref().to_str().expect("obtaining valid UTF-8 path");

        self.match_path_impl(path_str)
    }

    pub(crate) fn match_path_impl(&self, path: &str) -> Result<WorldMap, Error> {
        let captures = match self.regexp.captures(path) {
            Some(captures) => captures,
            None => {
                return Err(Error::NoMatchFound {
                    path: path.to_owned(),
                });
            }
        };

        let x = match captures.get(1) {
            Some(x) => x.as_str().parse::<i32>().unwrap(),
            None => {
                return Err(Error::NoMatchFound {
                    path: path.to_owned(),
                });
            }
        };

        let y = match captures.get(2) {
            Some(y) => y.as_str().parse::<i32>().unwrap(),
            None => {
                return Err(Error::NoMatchFound {
                    path: path.to_owned(),
                });
            }
        };

        // Calculate x and y positions based on the multiplier and offset.
        let x = x
            .checked_mul(self.multiplier_x)
            .ok_or(Error::RangeError(
                "Capture x * multiplierX causes overflow".to_string(),
            ))?
            .checked_add(self.offset_x)
            .ok_or(Error::RangeError(
                "Capture x * multiplierX + offsetX causes overflow".to_string(),
            ))?;

        let y = y
            .checked_mul(self.multiplier_y)
            .ok_or(Error::RangeError(
                "Capture y * multiplierY causes overflow".to_string(),
            ))?
            .checked_add(self.offset_y)
            .ok_or(Error::RangeError(
                "Capture y * multiplierY + offsetY causes overflow".to_string(),
            ))?;

        Ok(WorldMap {
            filename: path.to_owned(),
            x,
            y,
            width: None,
            height: None,
        })
    }
}

pub(crate) fn parse_world(
    world_path: &Path,
    reader: &mut impl ResourceReader,
) -> Result<World, Error> {
    let mut path = reader
        .read_from(world_path)
        .map_err(|err| Error::ResourceLoadingError {
            path: world_path.to_owned(),
            err: Box::new(err),
        })?;

    let mut world_string = String::new();
    path.read_to_string(&mut world_string)
        .map_err(|err| Error::ResourceLoadingError {
            path: world_path.to_owned(),
            err: Box::new(err),
        })?;

    let mut world: World = serde_json::from_str(&world_string).map_err(Error::JsonDecodingError)?;

    world.source = world_path.to_owned();

    Ok(world)
}
