use ggez::*;

/// A resource reader that uses assets from the ggez filesystem.
// Need to do newtype to implement ResourceReader for ggez's filesystem
pub struct GgezResourceReader<'ctx>(pub &'ctx mut ggez::filesystem::Filesystem);

impl tiled::ResourceReader for GgezResourceReader<'_> {
    type Resource = filesystem::File;

    type Error = GameError;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        self.0.open(path)
    }
}
