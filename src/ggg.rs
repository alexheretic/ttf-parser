//! Common types for GDEF, GPOS and GSUB tables.

use core::convert::TryFrom;

use crate::{GlyphId, Result, Error, Tag};
use crate::parser::*;
use crate::raw;
use crate::raw::gsubgpos::{Record, Condition, FeatureVariationRecord};


/// A generic interface over GSUB/GPOS tables.
pub trait GlyphPosSubTable {
    /// Returns an iterator over GSUB/GPOS table scripts.
    fn scripts(&self) -> Result<Scripts>;

    /// Returns a `Script` at `index`.
    ///
    /// Just a shorthand for:
    ///
    /// ```ignore
    /// table.scripts()?.nth(index.0 as usize).transpose()
    /// ```
    fn script_at(&self, index: ScriptIndex) -> Result<Option<Script>> {
        self.scripts()?.nth(index.0 as usize).transpose()
    }

    /// Returns an iterator over GSUB/GPOS table features.
    fn features(&self) -> Result<Features>;

    /// Returns a `Feature` at `index`.
    ///
    /// Just a shorthand for:
    ///
    /// ```ignore
    /// table.features()?.nth(index.0 as usize).transpose()
    /// ```
    fn feature_at(&self, index: FeatureIndex) -> Result<Option<Feature>> {
        self.features()?.nth(index.0 as usize).transpose()
    }

    /// Returns an iterator over GSUB/GPOS table lookups.
    fn lookups(&self) -> Result<Lookups>;

    /// Returns a `Lookup` at `index`.
    ///
    /// Just a shorthand for:
    ///
    /// ```ignore
    /// table.lookups()?.nth(index.0 as usize).transpose()
    /// ```
    fn lookup_at(&self, index: LookupIndex) -> Result<Option<Lookup>> {
        self.lookups()?.nth(index.0 as usize).transpose()
    }

    /// Returns an iterator over GSUB/GPOS table feature variations.
    ///
    /// Iterator will be empty when font doesn't have Feature Variations data.
    fn feature_variations(&self) -> Result<FeatureVariations>;

    /// Returns a `feature_variations` at `index`.
    ///
    /// Just a shorthand for:
    ///
    /// ```ignore
    /// table.feature_variations()?.nth(index.0 as usize).transpose()
    /// ```
    fn feature_variation_at(&self, index: FeatureVariationIndex) -> Result<Option<FeatureVariation>> {
        self.feature_variations()?.nth(index.0 as usize).transpose()
    }
}

pub(crate) fn parse_scripts(table: raw::gsubgpos::Table) -> Result<Scripts> {
    let data = table.data.try_slice_from(table.script_list_offset())?;
    let mut s = Stream::new(data);
    let count: u16 = s.read()?;
    Ok(Scripts {
        data,
        records: s.read_array(count)?,
        index: 0,
    })
}

pub(crate) fn parse_features(table: raw::gsubgpos::Table) -> Result<Features> {
    let data = table.data.try_slice_from(table.feature_list_offset())?;
    let mut s = Stream::new(data);
    let count: u16 = s.read()?;
    Ok(Features {
        data,
        records: s.read_array(count)?,
        index: 0,
    })
}

pub(crate) fn parse_lookups(table: raw::gsubgpos::Table) -> Result<Lookups> {
    let data = table.data.try_slice_from(table.lookup_list_offset())?;
    let mut s = Stream::new(data);
    let count: u16 = s.read()?;
    Ok(Lookups {
        data,
        records: s.read_array(count)?,
        index: 0,
    })
}

pub(crate) fn parse_feature_variations(table: raw::gsubgpos::Table) -> Result<FeatureVariations> {
    if !(table.major_version() == 1 && table.minor_version() == 1) {
        return Ok(FeatureVariations { data: &[], records: LazyArray::new(&[]), index: 0 });
    }

    let offset: Option<Offset32>
        = Stream::read_at(table.data, raw::gsubgpos::FEATURE_VARIATIONS_OFFSET_OFFSET)?;

    let offset = match offset {
        Some(v) => v,
        None => return Ok(FeatureVariations { data: &[], records: LazyArray::new(&[]), index: 0 }),
    };

    let data = table.data.try_slice_from(offset)?;
    let mut s = Stream::new(data);
    s.skip::<u16>(); // majorVersion
    s.skip::<u16>(); // minorVersion
    Ok(FeatureVariations {
        data,
        records: s.read_array32()?,
        index: 0,
    })
}


/// A type-safe wrapper for script index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct ScriptIndex(pub u16);


/// A type-safe wrapper for language index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct LanguageIndex(pub u16);


/// A type-safe wrapper for feature index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct FeatureIndex(pub u16);

impl FromData for FeatureIndex {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        FeatureIndex(u16::parse(data))
    }
}


/// A type-safe wrapper for feature index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct FeatureVariationIndex(pub u32);


/// A type-safe wrapper for lookup index used by GSUB/GPOS tables.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct LookupIndex(pub u16);

impl FromData for LookupIndex {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        LookupIndex(u16::parse(data))
    }
}


/// An iterator over GSUB/GPOS table scripts.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Scripts<'a> {
    data: &'a [u8], // GSUB/GPOS data from beginning of ScriptList.
    records: LazyArray16<'a, Record>,
    index: u16,
}

impl<'a> Iterator for Scripts<'a> {
    type Item = Result<Script<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        fn parse(data: &[u8], record: Record) -> Result<Script> {
            let data = data.try_slice_from(record.offset())?;
            let mut s = Stream::new(data);
            let default_lang: Option<Offset16> = s.read()?;
            let records = s.read_array16()?;
            Ok(Script {
                data,
                script: record.tag(),
                default_lang_offset: default_lang,
                records,
            })
        }

        let record = self.records.get(u16::try_from(n).ok()?)?;
        Some(parse(self.data, record))
    }
}


/// A font script.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Script<'a> {
    data: &'a [u8], // GSUB/GPOS data from beginning of ScriptTable.
    script: Tag,
    default_lang_offset: Option<Offset16>,
    records: LazyArray16<'a, Record>,
}

impl<'a> Script<'a> {
    /// Returns scrips's tag.
    #[inline]
    pub fn tag(&self) -> Tag {
        self.script
    }

    /// Returns scrips's default language.
    pub fn default_language(&self) -> Option<Language> {
        let data = self.data.try_slice_from(self.default_lang_offset?).ok()?;
        parse_lang_sys_table(data, None).ok()
    }

    /// Returns an iterator over scrips's languages.
    pub fn languages(&self) -> Languages {
        Languages {
            data: self.data,
            records: self.records,
            index: 0,
        }
    }

    /// Returns a `Language` at `index`.
    ///
    /// Just a shorthand for:
    ///
    /// ```ignore
    /// script.languages().nth(index.0 as usize)
    /// ```
    pub fn language_at(&self, index: LanguageIndex) -> Option<Language> {
        self.languages().nth(index.0 as usize)
    }

    /// Returns a `Language` by `tag`.
    ///
    /// Uses binary search and not an iterator internally.
    pub fn language_by_tag(&self, tag: Tag) -> Option<(LanguageIndex, Language)> {
        let (idx, _) = self.records.binary_search_by(|r| r.tag().cmp(&tag))?;
        let lang = self.language_at(LanguageIndex(idx))?;
        Some((LanguageIndex(idx), lang))
    }
}


/// An iterator over GSUB/GPOS table script languages.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Languages<'a> {
    data: &'a [u8], // GSUB/GPOS data from beginning of ScriptTable.
    records: LazyArray16<'a, Record>,
    index: u32,
}

impl<'a> Iterator for Languages<'a> {
    type Item = Language<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let record = self.records.get(u16::try_from(n).ok()?)?;
        let data = self.data.try_slice_from(record.offset()).ok()?;
        parse_lang_sys_table(data, Some(record.tag())).ok()
    }
}

fn parse_lang_sys_table(data: &[u8], tag: Option<Tag>) -> Result<Language> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // lookupOrder (reserved)

    let required_feature_index = match s.read::<u16>()? {
        0xFFFF => None, // no required features
        n => Some(FeatureIndex(n)),
    };

    let count: u16 = s.read()?;
    Ok(Language {
        tag: tag.unwrap_or_else(|| Tag::from_bytes(b"DFLT")),
        required_feature_index,
        feature_indices: s.read_array(count)?,
    })
}

/// A font language.
#[derive(Clone, Copy, Debug)]
pub struct Language<'a> {
    /// Language tag.
    pub tag: Tag,
    /// Required feature index.
    pub required_feature_index: Option<FeatureIndex>,
    /// List of feature indices associated with this language.
    pub feature_indices: LazyArray16<'a, FeatureIndex>,
}


/// An iterator over GSUB/GPOS table features.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Features<'a> {
    data: &'a [u8], // Data from beginning of FeatureList.
    records: LazyArray16<'a, Record>,
    index: u16,
}

impl<'a> Iterator for Features<'a> {
    type Item = Result<Feature<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        fn parse(data: &[u8], record: Record) -> Result<Feature> {
            let data = data.try_slice_from(record.offset())?;
            let mut s = Stream::new(data);
            s.skip::<Offset16>(); // featureParams
            Ok(Feature {
                tag: record.tag(),
                lookup_indices: s.read_array16()?,
            })
        }

        let record = self.records.get(u16::try_from(n).ok()?)?;
        Some(parse(self.data, record))
    }
}


/// A font feature.
#[derive(Clone, Copy, Debug)]
pub struct Feature<'a> {
    /// Feature tag.
    pub tag: Tag,
    /// List of lookup indices associated with this feature.
    pub lookup_indices: LazyArray16<'a, LookupIndex>,
}


/// An iterator over GSUB/GPOS table lookups.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct Lookups<'a> {
    data: &'a [u8], // Data from beginning of LookupList.
    records: LazyArray16<'a, Record>,
    index: u16,
}

impl<'a> Iterator for Lookups<'a> {
    type Item = Result<Lookup<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        fn parse(data: &[u8], record: Record) -> Result<Lookup> {
            let data = data.try_slice_from(record.offset())?;
            let mut s = Stream::new(data);
            let lookup_type: u16 = s.read()?;
            let lookup_flag: u16 = s.read()?;
            let offsets = s.read_offsets16(data)?;
            let mark_filtering_set: u16 = s.read()?;
            Ok(Lookup {
                lookup_type,
                lookup_flag,
                offsets,
                mark_filtering_set,
            })
        }

        let record = self.records.get(u16::try_from(n).ok()?)?;
        Some(parse(self.data, record))
    }
}


/// A font lookup.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Lookup<'a> {
    lookup_type: u16,
    lookup_flag: u16,
    offsets: Offsets16<'a>,
    mark_filtering_set: u16, // TODO: optional
}


/// An iterator over GSUB/GPOS table features.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy)]
pub struct FeatureVariations<'a> {
    data: &'a [u8], // Data from beginning of FeatureVariationsList.
    records: LazyArray32<'a, FeatureVariationRecord>,
    index: u32,
}

impl<'a> Iterator for FeatureVariations<'a> {
    type Item = Result<FeatureVariation<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let record = self.records.get(u32::try_from(n).ok()?)?;
        Some(Ok(FeatureVariation {
            data: self.data,
            condition_set_offset: record.condition_set_offset(),
            feature_table_substitution_offset: record.feature_table_substitution_offset(),
        }))
    }
}


/// A font feature variation.
#[derive(Clone, Copy, Debug)]
pub struct FeatureVariation<'a> {
    data: &'a [u8], // Data from beginning of FeatureVariations.
    condition_set_offset: Offset32,
    feature_table_substitution_offset: Offset32,
}

impl<'a> FeatureVariation<'a> {
    /// Evaluates variation using specified `coordinates`.
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    pub fn evaluate(&self, coordinates: &[i32]) -> bool {
        for condition in try_or!(self.condition_set(), false).filter_map(Result::ok) {
            if !condition.evaluate(coordinates) {
                return false;
            }
        }

        true
    }

    fn condition_set(&self) -> Result<ConditionSet<'a>> {
        let data = self.data.try_slice_from(self.condition_set_offset)?;
        Ok(ConditionSet {
            data,
            offsets: Stream::new(data).read_array16()?,
            index: 0,
        })
    }

    /// Returns an iterator over feature variation substitutions.
    pub fn substitutions(&self) -> Result<FeatureSubstitutions<'a>> {
        let data = self.data.try_slice_from(self.feature_table_substitution_offset)?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // majorVersion
        s.skip::<u16>(); // minorVersion
        Ok(FeatureSubstitutions {
            data,
            records: s.read_array16()?,
            index: 0,
        })
    }
}


#[derive(Clone, Copy, Debug)]
struct ConditionSet<'a> {
    data: &'a [u8], // Data from beginning of ConditionSet.
    offsets: LazyArray16<'a, Offset32>,
    index: u16,
}

impl<'a> Iterator for ConditionSet<'a> {
    type Item = Result<Condition>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.offsets.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        fn parse(data: &[u8], offset: Offset32) -> Result<Condition> {
            let condition: Condition = Stream::read_at(data, offset.to_usize())?;
            if condition.format() != 1 {
                return Err(Error::UnsupportedTableVersion);
            }

            Ok(condition)
        }


        let offset = self.offsets.get(u16::try_from(n).ok()?)?;
        Some(parse(self.data, offset))
    }
}


impl Condition {
    fn evaluate(&self, coordinates: &[i32]) -> bool {
        let coord = coordinates.get(self.axis_index() as usize).cloned().unwrap_or(0);
        self.filter_range_min_value() as i32 <= coord && coord <= self.filter_range_max_value() as i32
    }
}


/// An iterator over GSUB/GPOS table features.
#[derive(Clone, Copy, Debug)]
pub struct FeatureSubstitutions<'a> {
    data: &'a [u8], // Data from beginning of FeatureVariationsList.
    records: LazyArray16<'a, FeatureTableSubstitutionRecord>,
    index: u16,
}

impl<'a> Iterator for FeatureSubstitutions<'a> {
    type Item = FeatureSubstitution<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.checked_add(1)?;
        self.nth(self.index as usize - 1)
    }

    fn count(self) -> usize {
        self.records.len().to_usize()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let record = self.records.get(u16::try_from(n).ok()?)?;
        Some(FeatureSubstitution {
            data: self.data,
            index: record.index,
            table_offset: record.table_offset,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct FeatureTableSubstitutionRecord {
    index: FeatureIndex,
    table_offset: Offset32,
}

impl FromData for FeatureTableSubstitutionRecord {
    const SIZE: usize = 6;

    fn parse(data: &[u8]) -> Self {
        let mut s = SafeStream::new(data);
        FeatureTableSubstitutionRecord {
            index: s.read(),
            table_offset: s.read(),
        }
    }
}


/// A font feature substitution.
#[derive(Clone, Copy, Debug)]
pub struct FeatureSubstitution<'a> {
    data: &'a [u8], // Data from beginning of FeatureTableSubstitution.
    index: FeatureIndex,
    table_offset: Offset32,
}

impl<'a> FeatureSubstitution<'a> {
    /// Returns feature index.
    pub fn index(&self) -> FeatureIndex {
        self.index
    }

    /// Returns substitution's feature.
    pub fn feature(&self) -> Result<Feature<'a>> {
        let data = self.data.try_slice_from(self.table_offset)?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // featureParams (reserved)
        let count: u16 = s.read()?;
        Ok(Feature {
            tag: Tag(0),
            lookup_indices: s.read_array(count)?,
        })
    }
}


/// A [Coverage Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoverageTable<'a> {
    data: &'a [u8],
}

impl<'a> CoverageTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CoverageTable { data }
    }

    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        let mut s = Stream::new(self.data);
        let format: u16 = match s.read() {
            Ok(v) => v,
            Err(_) => return false,
        };

        match format {
            1 => {
                s.read_array16::<GlyphId>().unwrap().binary_search(&glyph_id).is_some()
            }
            2 => {
                let records = s.read_array16::<crate::raw::gdef::RangeRecord>().unwrap();
                records.into_iter().any(|r| r.range().contains(&glyph_id))
            }
            _ => false,
        }
    }
}


/// A value of [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Class(pub u16);

impl FromData for Class {
    fn parse(data: &[u8]) -> Self {
        Class(SafeStream::new(data).read())
    }
}


/// A [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
pub(crate) struct ClassDefinitionTable<'a> {
    data: &'a [u8],
}

impl<'a> ClassDefinitionTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ClassDefinitionTable { data }
    }

    /// Any glyph not included in the range of covered glyph IDs automatically belongs to Class 0.
    pub fn get(&self, glyph_id: GlyphId) -> Result<Class> {
        let mut s = Stream::new(self.data);
        let format: u16 = s.read()?;
        match format {
            1 => {
                let start_glyph_id: GlyphId = s.read()?;

                // Prevent overflow.
                if glyph_id < start_glyph_id {
                    return Ok(Class(0));
                }

                let classes = s.read_array16::<Class>()?;
                Ok(classes.get(glyph_id.0 - start_glyph_id.0).unwrap_or(Class(0)))
            }
            2 => {
                let records = s.read_array16::<crate::raw::gdef::ClassRangeRecord>()?;
                Ok(match records.into_iter().find(|r| r.range().contains(&glyph_id)) {
                    Some(record) => Class(record.class()),
                    None => Class(0),
                })
            }
            _ => Ok(Class(0)),
        }
    }
}