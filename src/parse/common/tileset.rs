use std::path::PathBuf;

use crate::{Error, Gid, Image, Result, Tileset};

pub(crate) enum EmbeddedParseResultType {
    ExternalReference { tileset_path: PathBuf },
    Embedded { tileset: Tileset },
}

pub(crate) struct EmbeddedParseResult {
    pub first_gid: Gid,
    pub result_type: EmbeddedParseResultType,
}

impl Tileset {
    pub(crate) fn calculate_columns(
        image: &Option<Image>,
        tile_width: u32,
        margin: u32,
        spacing: u32,
    ) -> Result<u32> {
        image
            .as_ref()
            .map(|image| (image.width as u32 - margin + spacing) / (tile_width + spacing))
            .ok_or_else(|| {
                Error::MalformedAttributes(
                    "No <image> nor columns attribute in <tileset>".to_string(),
                )
            })
    }
}
