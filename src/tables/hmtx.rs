// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use core::num::NonZeroU16;

use crate::GlyphId;
use crate::parser::{Stream, LazyArray16};
use crate::raw::hmtx as raw;

#[derive(Clone, Copy)]
pub struct Table<'a> {
    metrics: LazyArray16<'a, raw::HorizontalMetrics>,
    bearings: Option<LazyArray16<'a, i16>>,
}

impl<'a> Table<'a> {
    pub fn parse(
        data: &'a [u8],
        number_of_hmetrics: NonZeroU16,
        number_of_glyphs: NonZeroU16,
    ) -> Option<Self> {
        let mut s = Stream::new(data);
        let metrics = s.read_array16(number_of_hmetrics.get())?;

        // 'If the number_of_hmetrics is less than the total number of glyphs,
        // then that array is followed by an array for the left side bearing values
        // of the remaining glyphs.'
        let bearings = if number_of_hmetrics < number_of_glyphs {
            s.read_array16(number_of_glyphs.get() - number_of_hmetrics.get())
        } else {
            None
        };

        Some(Table {
            metrics,
            bearings,
        })
    }

    #[inline]
    pub fn advance(&self, glyph_id: GlyphId) -> Option<u16> {
        if let Some(metrics) = self.metrics.get(glyph_id.0) {
            Some(metrics.advance_width())
        } else {
            // 'As an optimization, the number of records can be less than the number of glyphs,
            // in which case the advance width value of the last record applies
            // to all remaining glyph IDs.'
            self.metrics.last().map(|m| m.advance_width())
        }
    }

    #[inline]
    pub fn side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        if let Some(metrics) = self.metrics.get(glyph_id.0) {
            Some(metrics.lsb())
        } else if let Some(bearings) = self.bearings {
            // 'If the number_of_hmetrics is less than the total number of glyphs,
            // then that array is followed by an array for the left side bearing values
            // of the remaining glyphs.'

            let number_of_hmetrics = self.metrics.len();

            // Check for overflow.
            if glyph_id.0 < number_of_hmetrics {
                return None;
            }

            bearings.get(glyph_id.0 - number_of_hmetrics)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer;
    use writer::TtfType::*;

    macro_rules! nzu16 {
        ($n:expr) => { NonZeroU16::new($n).unwrap() };
    }

    #[test]
    fn simple_case() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
        ]);

        let table = Table::parse(&data, nzu16!(1), nzu16!(1)).unwrap();
        assert_eq!(table.advance(GlyphId(0)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
    }

    #[test]
    fn empty() {
        assert!(Table::parse(&[], nzu16!(1), nzu16!(1)).is_none());
    }

    #[test]
    fn smaller_than_glyphs_count() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
            Int16(3), // sideBearing[1]
        ]);

        let table = Table::parse(&data, nzu16!(1), nzu16!(2)).unwrap();
        assert_eq!(table.advance(GlyphId(0)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
        assert_eq!(table.advance(GlyphId(1)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(1)), Some(3));
    }

    #[test]
    fn less_metrics_than_glyphs() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
            UInt16(3), // advanceWidth[1]
            Int16(4), // sideBearing[1]
            Int16(5), // sideBearing[2]
        ]);

        let table = Table::parse(&data, nzu16!(2), nzu16!(1)).unwrap();
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
        assert_eq!(table.side_bearing(GlyphId(1)), Some(4));
        assert_eq!(table.side_bearing(GlyphId(2)), None);
    }
}
