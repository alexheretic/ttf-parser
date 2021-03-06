// https://docs.microsoft.com/en-us/typography/opentype/spec/vorg

use crate::GlyphId;
use crate::parser::{Stream, LazyArray16};
use crate::raw::vorg as raw;

#[derive(Clone, Copy)]
pub struct Table<'a> {
    default_y: i16,
    origins: LazyArray16<'a, raw::VertOriginYMetrics>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }

        let default_y: i16 = s.read()?;
        let count: u16 = s.read()?;
        let origins = s.read_array16(count)?;

        Some(Table {
            default_y,
            origins,
        })
    }

    pub fn glyph_y_origin(&self, glyph_id: GlyphId) -> i16 {
        self.origins.binary_search_by(|m| m.glyph_index().cmp(&glyph_id))
            .map(|(_, m)| m.vert_origin_y())
            .unwrap_or(self.default_y)
    }
}
