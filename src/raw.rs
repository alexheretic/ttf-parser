// This file is autogenerated by scripts/get-tables.py
// Do not edit it!

use crate::parser::{FromData, Offset32};
use crate::Tag;

#[derive(Clone, Copy)]
pub struct TTCHeader<'a> {
    data: &'a [u8],
}

impl<'a> TTCHeader<'a> {
    #[allow(dead_code)]
    pub const SIZE: usize = 12;

    #[inline(always)]
    pub fn new(input: &'a [u8]) -> Self {
        debug_assert_eq!(input.len(), 12);
        TTCHeader { data: input }
    }

    #[inline(always)]
    pub fn ttc_tag(&self) -> Tag {
        use core::convert::TryInto;
        // Unwrap is safe, because an array and a slice have the same size.
        Tag::from_bytes(&self.data[0..4].try_into().unwrap())
    }

    #[inline(always)]
    pub fn num_fonts(&self) -> u32 {
        u32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]])
    }
}

#[derive(Clone, Copy)]
pub struct TableRecord {
    data: [u8; 16],
}

impl TableRecord {
    #[allow(dead_code)]
    pub const SIZE: usize = 16;

    #[inline(always)]
    pub fn new(input: &[u8]) -> Self {
        let mut data = [0u8; Self::SIZE];
        data.clone_from_slice(input);
        TableRecord { data }
    }

    #[inline(always)]
    pub fn table_tag(&self) -> Tag {
        use core::convert::TryInto;
        // Unwrap is safe, because an array and a slice have the same size.
        Tag::from_bytes(&self.data[0..4].try_into().unwrap())
    }

    #[inline(always)]
    pub fn offset(&self) -> Offset32 {
        Offset32(u32::from_be_bytes([
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
        ]))
    }

    #[inline(always)]
    pub fn length(&self) -> u32 {
        u32::from_be_bytes([self.data[12], self.data[13], self.data[14], self.data[15]])
    }
}

impl FromData for TableRecord {
    const SIZE: usize = TableRecord::SIZE;

    #[inline]
    fn parse(data: &[u8]) -> Self {
        Self::new(data)
    }
}

pub mod head {
    #[derive(Clone, Copy)]
    pub struct Table<'a> {
        data: &'a [u8],
    }

    impl<'a> Table<'a> {
        #[inline(always)]
        pub fn parse(input: &'a [u8]) -> Option<Self> {
            if input.len() == 54 {
                Some(Table { data: input })
            } else {
                None
            }
        }

        #[inline(always)]
        pub fn units_per_em(&self) -> u16 {
            u16::from_be_bytes([self.data[18], self.data[19]])
        }

        #[inline(always)]
        pub fn index_to_loc_format(&self) -> i16 {
            i16::from_be_bytes([self.data[50], self.data[51]])
        }
    }
}

pub mod hhea {
    use core::num::NonZeroU16;

    #[derive(Clone, Copy)]
    pub struct Table<'a> {
        data: &'a [u8],
    }

    impl<'a> Table<'a> {
        #[inline(always)]
        pub fn parse(input: &'a [u8]) -> Option<Self> {
            if input.len() == 36 {
                Some(Table { data: input })
            } else {
                None
            }
        }

        #[inline(always)]
        pub fn ascender(&self) -> i16 {
            i16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn descender(&self) -> i16 {
            i16::from_be_bytes([self.data[6], self.data[7]])
        }

        #[inline(always)]
        pub fn line_gap(&self) -> i16 {
            i16::from_be_bytes([self.data[8], self.data[9]])
        }

        #[inline(always)]
        pub fn number_of_h_metrics(&self) -> Option<NonZeroU16> {
            NonZeroU16::new(u16::from_be_bytes([self.data[34], self.data[35]]))
        }
    }
}

pub mod hmtx {
    use crate::parser::FromData;

    #[derive(Clone, Copy)]
    pub struct HorizontalMetrics {
        data: [u8; 4],
    }

    impl HorizontalMetrics {
        #[allow(dead_code)]
        pub const SIZE: usize = 4;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            HorizontalMetrics { data }
        }

        #[inline(always)]
        pub fn advance_width(&self) -> u16 {
            u16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn lsb(&self) -> i16 {
            i16::from_be_bytes([self.data[2], self.data[3]])
        }
    }

    impl FromData for HorizontalMetrics {
        const SIZE: usize = HorizontalMetrics::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod vhea {
    use core::num::NonZeroU16;

    #[derive(Clone, Copy)]
    pub struct Table<'a> {
        data: &'a [u8],
    }

    impl<'a> Table<'a> {
        #[inline(always)]
        pub fn parse(input: &'a [u8]) -> Option<Self> {
            if input.len() == 36 {
                Some(Table { data: input })
            } else {
                None
            }
        }

        #[inline(always)]
        pub fn ascender(&self) -> i16 {
            i16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn descender(&self) -> i16 {
            i16::from_be_bytes([self.data[6], self.data[7]])
        }

        #[inline(always)]
        pub fn line_gap(&self) -> i16 {
            i16::from_be_bytes([self.data[8], self.data[9]])
        }

        #[inline(always)]
        pub fn num_of_long_ver_metrics(&self) -> Option<NonZeroU16> {
            NonZeroU16::new(u16::from_be_bytes([self.data[34], self.data[35]]))
        }
    }
}

pub mod post {
    #[derive(Clone, Copy)]
    pub struct Table<'a> {
        data: &'a [u8],
    }

    impl<'a> Table<'a> {
        #[allow(dead_code)]
        pub const SIZE: usize = 32;

        #[inline(always)]
        pub fn new(input: &'a [u8]) -> Self {
            debug_assert_eq!(input.len(), 32);
            Table { data: input }
        }

        #[inline(always)]
        pub fn underline_position(&self) -> i16 {
            i16::from_be_bytes([self.data[8], self.data[9]])
        }

        #[inline(always)]
        pub fn underline_thickness(&self) -> i16 {
            i16::from_be_bytes([self.data[10], self.data[11]])
        }
    }
}

pub mod cmap {
    use crate::parser::{FromData, Offset32};
    use crate::GlyphId;

    #[derive(Clone, Copy)]
    pub struct EncodingRecord {
        data: [u8; 8],
    }

    impl EncodingRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 8;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            EncodingRecord { data }
        }

        #[inline(always)]
        pub fn platform_id(&self) -> u16 {
            u16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn encoding_id(&self) -> u16 {
            u16::from_be_bytes([self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn offset(&self) -> Offset32 {
            Offset32(u32::from_be_bytes([
                self.data[4],
                self.data[5],
                self.data[6],
                self.data[7],
            ]))
        }
    }

    impl FromData for EncodingRecord {
        const SIZE: usize = EncodingRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct SubHeaderRecord {
        data: [u8; 8],
    }

    impl SubHeaderRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 8;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            SubHeaderRecord { data }
        }

        #[inline(always)]
        pub fn first_code(&self) -> u16 {
            u16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn entry_count(&self) -> u16 {
            u16::from_be_bytes([self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn id_delta(&self) -> i16 {
            i16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn id_range_offset(&self) -> u16 {
            u16::from_be_bytes([self.data[6], self.data[7]])
        }
    }

    impl FromData for SubHeaderRecord {
        const SIZE: usize = SubHeaderRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct SequentialMapGroup {
        data: [u8; 12],
    }

    impl SequentialMapGroup {
        #[allow(dead_code)]
        pub const SIZE: usize = 12;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            SequentialMapGroup { data }
        }

        #[inline(always)]
        pub fn start_char_code(&self) -> u32 {
            u32::from_be_bytes([self.data[0], self.data[1], self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn end_char_code(&self) -> u32 {
            u32::from_be_bytes([self.data[4], self.data[5], self.data[6], self.data[7]])
        }

        #[inline(always)]
        pub fn start_glyph_id(&self) -> u32 {
            u32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]])
        }
    }

    impl FromData for SequentialMapGroup {
        const SIZE: usize = SequentialMapGroup::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct UnicodeRangeRecord {
        data: [u8; 4],
    }

    impl UnicodeRangeRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 4;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            UnicodeRangeRecord { data }
        }

        #[inline(always)]
        pub fn start_unicode_value(&self) -> u32 {
            (self.data[0] as u32) << 16 | (self.data[1] as u32) << 8 | self.data[2] as u32
        }

        #[inline(always)]
        pub fn additional_count(&self) -> u8 {
            self.data[3]
        }
    }

    impl FromData for UnicodeRangeRecord {
        const SIZE: usize = UnicodeRangeRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct UVSMappingRecord {
        data: [u8; 5],
    }

    impl UVSMappingRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 5;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            UVSMappingRecord { data }
        }

        #[inline(always)]
        pub fn unicode_value(&self) -> u32 {
            (self.data[0] as u32) << 16 | (self.data[1] as u32) << 8 | self.data[2] as u32
        }

        #[inline(always)]
        pub fn glyph_id(&self) -> GlyphId {
            GlyphId(u16::from_be_bytes([self.data[3], self.data[4]]))
        }
    }

    impl FromData for UVSMappingRecord {
        const SIZE: usize = UVSMappingRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct VariationSelectorRecord {
        data: [u8; 11],
    }

    impl VariationSelectorRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 11;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            VariationSelectorRecord { data }
        }

        #[inline(always)]
        pub fn var_selector(&self) -> u32 {
            (self.data[0] as u32) << 16 | (self.data[1] as u32) << 8 | self.data[2] as u32
        }

        #[inline(always)]
        pub fn default_uvs_offset(&self) -> Option<Offset32> {
            let n = u32::from_be_bytes([self.data[3], self.data[4], self.data[5], self.data[6]]);
            if n != 0 {
                Some(Offset32(n))
            } else {
                None
            }
        }

        #[inline(always)]
        pub fn non_default_uvs_offset(&self) -> Option<Offset32> {
            let n = u32::from_be_bytes([self.data[7], self.data[8], self.data[9], self.data[10]]);
            if n != 0 {
                Some(Offset32(n))
            } else {
                None
            }
        }
    }

    impl FromData for VariationSelectorRecord {
        const SIZE: usize = VariationSelectorRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod os_2 {
    pub const SX_HEIGHT_OFFSET: usize = 86;

    #[derive(Clone, Copy)]
    pub struct Table<'a> {
        data: &'a [u8],
    }

    impl<'a> Table<'a> {
        #[allow(dead_code)]
        pub const SIZE: usize = 78;

        #[inline(always)]
        pub fn new(input: &'a [u8]) -> Self {
            debug_assert_eq!(input.len(), 78);
            Table { data: input }
        }

        #[inline(always)]
        pub fn us_weight_class(&self) -> u16 {
            u16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn us_width_class(&self) -> u16 {
            u16::from_be_bytes([self.data[6], self.data[7]])
        }

        #[inline(always)]
        pub fn y_subscript_x_size(&self) -> i16 {
            i16::from_be_bytes([self.data[10], self.data[11]])
        }

        #[inline(always)]
        pub fn y_subscript_y_size(&self) -> i16 {
            i16::from_be_bytes([self.data[12], self.data[13]])
        }

        #[inline(always)]
        pub fn y_subscript_x_offset(&self) -> i16 {
            i16::from_be_bytes([self.data[14], self.data[15]])
        }

        #[inline(always)]
        pub fn y_subscript_y_offset(&self) -> i16 {
            i16::from_be_bytes([self.data[16], self.data[17]])
        }

        #[inline(always)]
        pub fn y_superscript_x_size(&self) -> i16 {
            i16::from_be_bytes([self.data[18], self.data[19]])
        }

        #[inline(always)]
        pub fn y_superscript_y_size(&self) -> i16 {
            i16::from_be_bytes([self.data[20], self.data[21]])
        }

        #[inline(always)]
        pub fn y_superscript_x_offset(&self) -> i16 {
            i16::from_be_bytes([self.data[22], self.data[23]])
        }

        #[inline(always)]
        pub fn y_superscript_y_offset(&self) -> i16 {
            i16::from_be_bytes([self.data[24], self.data[25]])
        }

        #[inline(always)]
        pub fn y_strikeout_size(&self) -> i16 {
            i16::from_be_bytes([self.data[26], self.data[27]])
        }

        #[inline(always)]
        pub fn y_strikeout_position(&self) -> i16 {
            i16::from_be_bytes([self.data[28], self.data[29]])
        }

        #[inline(always)]
        pub fn fs_selection(&self) -> u16 {
            u16::from_be_bytes([self.data[62], self.data[63]])
        }

        #[inline(always)]
        pub fn s_typo_ascender(&self) -> i16 {
            i16::from_be_bytes([self.data[68], self.data[69]])
        }

        #[inline(always)]
        pub fn s_typo_descender(&self) -> i16 {
            i16::from_be_bytes([self.data[70], self.data[71]])
        }

        #[inline(always)]
        pub fn s_typo_line_gap(&self) -> i16 {
            i16::from_be_bytes([self.data[72], self.data[73]])
        }
    }
}

pub mod name {
    #[derive(Clone, Copy)]
    pub struct NameRecord {
        data: [u8; 12],
    }

    impl NameRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 12;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            NameRecord { data }
        }

        #[inline(always)]
        pub fn platform_id(&self) -> u16 {
            u16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn encoding_id(&self) -> u16 {
            u16::from_be_bytes([self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn language_id(&self) -> u16 {
            u16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn name_id(&self) -> u16 {
            u16::from_be_bytes([self.data[6], self.data[7]])
        }

        #[inline(always)]
        pub fn length(&self) -> u16 {
            u16::from_be_bytes([self.data[8], self.data[9]])
        }

        #[inline(always)]
        pub fn offset(&self) -> u16 {
            u16::from_be_bytes([self.data[10], self.data[11]])
        }
    }
}

pub mod gdef {
    use crate::parser::FromData;
    use crate::GlyphId;
    use core::ops::RangeInclusive;

    #[derive(Clone, Copy)]
    pub struct ClassRangeRecord {
        data: [u8; 6],
    }

    impl ClassRangeRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 6;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            ClassRangeRecord { data }
        }

        #[inline(always)]
        pub fn range(&self) -> RangeInclusive<GlyphId> {
            GlyphId(u16::from_be_bytes([self.data[0], self.data[1]]))
                ..=GlyphId(u16::from_be_bytes([self.data[2], self.data[3]]))
        }

        #[inline(always)]
        pub fn class(&self) -> u16 {
            u16::from_be_bytes([self.data[4], self.data[5]])
        }
    }

    impl FromData for ClassRangeRecord {
        const SIZE: usize = ClassRangeRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct RangeRecord {
        data: [u8; 6],
    }

    impl RangeRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 6;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            RangeRecord { data }
        }

        #[inline(always)]
        pub fn range(&self) -> RangeInclusive<GlyphId> {
            GlyphId(u16::from_be_bytes([self.data[0], self.data[1]]))
                ..=GlyphId(u16::from_be_bytes([self.data[2], self.data[3]]))
        }
    }

    impl FromData for RangeRecord {
        const SIZE: usize = RangeRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod gsubgpos {
    use crate::parser::{FromData, Offset16, Offset32};
    use crate::Tag;

    #[derive(Clone, Copy)]
    pub struct Record {
        data: [u8; 6],
    }

    impl Record {
        #[allow(dead_code)]
        pub const SIZE: usize = 6;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            Record { data }
        }

        #[inline(always)]
        pub fn tag(&self) -> Tag {
            use core::convert::TryInto;
            // Unwrap is safe, because an array and a slice have the same size.
            Tag::from_bytes(&self.data[0..4].try_into().unwrap())
        }

        #[inline(always)]
        pub fn offset(&self) -> Offset16 {
            Offset16(u16::from_be_bytes([self.data[4], self.data[5]]))
        }
    }

    impl FromData for Record {
        const SIZE: usize = Record::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct Condition {
        data: [u8; 8],
    }

    impl Condition {
        #[allow(dead_code)]
        pub const SIZE: usize = 8;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            Condition { data }
        }

        #[inline(always)]
        pub fn format(&self) -> u16 {
            u16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn axis_index(&self) -> u16 {
            u16::from_be_bytes([self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn filter_range_min_value(&self) -> i16 {
            i16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn filter_range_max_value(&self) -> i16 {
            i16::from_be_bytes([self.data[6], self.data[7]])
        }
    }

    impl FromData for Condition {
        const SIZE: usize = Condition::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct FeatureVariationRecord {
        data: [u8; 8],
    }

    impl FeatureVariationRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 8;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            FeatureVariationRecord { data }
        }

        #[inline(always)]
        pub fn condition_set_offset(&self) -> Offset32 {
            Offset32(u32::from_be_bytes([
                self.data[0],
                self.data[1],
                self.data[2],
                self.data[3],
            ]))
        }

        #[inline(always)]
        pub fn feature_table_substitution_offset(&self) -> Offset32 {
            Offset32(u32::from_be_bytes([
                self.data[4],
                self.data[5],
                self.data[6],
                self.data[7],
            ]))
        }
    }

    impl FromData for FeatureVariationRecord {
        const SIZE: usize = FeatureVariationRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod avar {
    use crate::parser::FromData;

    #[derive(Clone, Copy)]
    pub struct AxisValueMapRecord {
        data: [u8; 4],
    }

    impl AxisValueMapRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 4;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            AxisValueMapRecord { data }
        }

        #[inline(always)]
        pub fn from_coordinate(&self) -> i16 {
            i16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn to_coordinate(&self) -> i16 {
            i16::from_be_bytes([self.data[2], self.data[3]])
        }
    }

    impl FromData for AxisValueMapRecord {
        const SIZE: usize = AxisValueMapRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod fvar {
    use crate::parser::FromData;
    use crate::Tag;

    #[derive(Clone, Copy)]
    pub struct VariationAxisRecord {
        data: [u8; 20],
    }

    impl VariationAxisRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 20;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            VariationAxisRecord { data }
        }

        #[inline(always)]
        pub fn axis_tag(&self) -> Tag {
            use core::convert::TryInto;
            // Unwrap is safe, because an array and a slice have the same size.
            Tag::from_bytes(&self.data[0..4].try_into().unwrap())
        }

        #[inline(always)]
        pub fn min_value(&self) -> f32 {
            (i32::from_be_bytes([self.data[4], self.data[5], self.data[6], self.data[7]]) as f32
                / 65536.0)
        }

        #[inline(always)]
        pub fn def_value(&self) -> f32 {
            (i32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]) as f32
                / 65536.0)
        }

        #[inline(always)]
        pub fn max_value(&self) -> f32 {
            (i32::from_be_bytes([self.data[12], self.data[13], self.data[14], self.data[15]])
                as f32
                / 65536.0)
        }

        #[inline(always)]
        pub fn flags(&self) -> u16 {
            u16::from_be_bytes([self.data[16], self.data[17]])
        }

        #[inline(always)]
        pub fn axis_name_id(&self) -> u16 {
            u16::from_be_bytes([self.data[18], self.data[19]])
        }
    }

    impl FromData for VariationAxisRecord {
        const SIZE: usize = VariationAxisRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod vorg {
    use crate::parser::FromData;
    use crate::GlyphId;

    #[derive(Clone, Copy)]
    pub struct VertOriginYMetrics {
        data: [u8; 4],
    }

    impl VertOriginYMetrics {
        #[allow(dead_code)]
        pub const SIZE: usize = 4;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            VertOriginYMetrics { data }
        }

        #[inline(always)]
        pub fn glyph_index(&self) -> GlyphId {
            GlyphId(u16::from_be_bytes([self.data[0], self.data[1]]))
        }

        #[inline(always)]
        pub fn vert_origin_y(&self) -> i16 {
            i16::from_be_bytes([self.data[2], self.data[3]])
        }
    }

    impl FromData for VertOriginYMetrics {
        const SIZE: usize = VertOriginYMetrics::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}

pub mod mvar {
    use crate::parser::FromData;
    use crate::Tag;

    #[derive(Clone, Copy)]
    pub struct ValueRecord {
        data: [u8; 8],
    }

    impl ValueRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 8;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            ValueRecord { data }
        }

        #[inline(always)]
        pub fn value_tag(&self) -> Tag {
            use core::convert::TryInto;
            // Unwrap is safe, because an array and a slice have the same size.
            Tag::from_bytes(&self.data[0..4].try_into().unwrap())
        }

        #[inline(always)]
        pub fn delta_set_outer_index(&self) -> u16 {
            u16::from_be_bytes([self.data[4], self.data[5]])
        }

        #[inline(always)]
        pub fn delta_set_inner_index(&self) -> u16 {
            u16::from_be_bytes([self.data[6], self.data[7]])
        }
    }

    impl FromData for ValueRecord {
        const SIZE: usize = ValueRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }

    #[derive(Clone, Copy)]
    pub struct RegionAxisCoordinatesRecord {
        data: [u8; 6],
    }

    impl RegionAxisCoordinatesRecord {
        #[allow(dead_code)]
        pub const SIZE: usize = 6;

        #[inline(always)]
        pub fn new(input: &[u8]) -> Self {
            let mut data = [0u8; Self::SIZE];
            data.clone_from_slice(input);
            RegionAxisCoordinatesRecord { data }
        }

        #[inline(always)]
        pub fn start_coord(&self) -> i16 {
            i16::from_be_bytes([self.data[0], self.data[1]])
        }

        #[inline(always)]
        pub fn peak_coord(&self) -> i16 {
            i16::from_be_bytes([self.data[2], self.data[3]])
        }

        #[inline(always)]
        pub fn end_coord(&self) -> i16 {
            i16::from_be_bytes([self.data[4], self.data[5]])
        }
    }

    impl FromData for RegionAxisCoordinatesRecord {
        const SIZE: usize = RegionAxisCoordinatesRecord::SIZE;

        #[inline]
        fn parse(data: &[u8]) -> Self {
            Self::new(data)
        }
    }
}
