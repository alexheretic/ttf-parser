// https://docs.microsoft.com/en-us/typography/opentype/spec/name

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use crate::parser::LazyArray16;

use crate::parser::Stream;
use crate::raw::name as raw;


/// A list of [name ID](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids)'s.
pub mod name_id {
    #![allow(missing_docs)]

    pub const COPYRIGHT_NOTICE: u16                     = 0;
    pub const FAMILY: u16                               = 1;
    pub const SUBFAMILY: u16                            = 2;
    pub const UNIQUE_ID: u16                            = 3;
    pub const FULL_NAME: u16                            = 4;
    pub const VERSION: u16                              = 5;
    pub const POST_SCRIPT_NAME: u16                     = 6;
    pub const TRADEMARK: u16                            = 7;
    pub const MANUFACTURER: u16                         = 8;
    pub const DESIGNER: u16                             = 9;
    pub const DESCRIPTION: u16                          = 10;
    pub const VENDOR_URL: u16                           = 11;
    pub const DESIGNER_URL: u16                         = 12;
    pub const LICENSE: u16                              = 13;
    pub const LICENSE_URL: u16                          = 14;
    //        RESERVED                                  = 15
    pub const TYPOGRAPHIC_FAMILY: u16                   = 16;
    pub const TYPOGRAPHIC_SUBFAMILY: u16                = 17;
    pub const COMPATIBLE_FULL: u16                      = 18;
    pub const SAMPLE_TEXT: u16                          = 19;
    pub const POST_SCRIPT_CID: u16                      = 20;
    pub const WWS_FAMILY: u16                           = 21;
    pub const WWS_SUBFAMILY: u16                        = 22;
    pub const LIGHT_BACKGROUND_PALETTE: u16             = 23;
    pub const DARK_BACKGROUND_PALETTE: u16              = 24;
    pub const VARIATIONS_POST_SCRIPT_NAME_PREFIX: u16   = 25;
}


/// A [platform ID](https://docs.microsoft.com/en-us/typography/opentype/spec/name#platform-ids).
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum PlatformId {
    Unicode,
    Macintosh,
    Iso,
    Windows,
    Custom,
}

impl PlatformId {
    pub(crate) fn from_u16(n: u16) -> Option<Self> {
        match n {
            0 => Some(PlatformId::Unicode),
            1 => Some(PlatformId::Macintosh),
            2 => Some(PlatformId::Iso),
            3 => Some(PlatformId::Windows),
            4 => Some(PlatformId::Custom),
            _ => None,
        }
    }
}


#[cfg(feature = "std")]
#[inline]
fn is_unicode_encoding(platform_id: PlatformId, encoding_id: u16) -> bool {
    // https://docs.microsoft.com/en-us/typography/opentype/spec/name#windows-encoding-ids
    const WINDOWS_UNICODE_BMP_ENCODING_ID: u16 = 1;

    match platform_id {
        PlatformId::Unicode => true,
        PlatformId::Windows if encoding_id == WINDOWS_UNICODE_BMP_ENCODING_ID => true,
        _ => false,
    }
}


/// A [Name Record](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records).
#[derive(Clone, Copy)]
pub struct Name<'a> {
    data: raw::NameRecord,
    strings: &'a [u8],
}

impl<'a> Name<'a> {
    /// Parses the platform ID.
    pub fn platform_id(&self) -> Option<PlatformId> {
        PlatformId::from_u16(self.data.platform_id())
    }

    /// Parses the platform-specific encoding ID.
    pub fn encoding_id(&self) -> u16 {
        self.data.encoding_id()
    }

    /// Parses the language ID.
    pub fn language_id(&self) -> u16 {
        self.data.language_id()
    }

    /// Parses the [Name ID](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids).
    ///
    /// A predefined list of ID's can be found in the [`name_id`](name_id/index.html) module.
    pub fn name_id(&self) -> u16 {
        self.data.name_id()
    }

    /// Parses the Name's data as bytes.
    ///
    /// Can be empty.
    pub fn name(&self) -> &'a [u8] {
        let start = usize::from(self.data.offset());
        let end = start + usize::from(self.data.length());
        self.strings.get(start..end).unwrap_or(&[])
    }

    /// Parses the Name's data as a UTF-8 string.
    ///
    /// Only Unicode names are supported. And since they are stored as UTF-16BE,
    /// we can't return `&str` and have to allocate a `String`.
    ///
    /// Supports:
    /// - Unicode Platform ID
    /// - Windows Platform ID + Unicode BMP
    #[cfg(feature = "std")]
    #[inline(never)]
    pub fn name_utf8(&self) -> Option<String> {
        if self.is_unicode() {
            self.name_from_utf16_be()
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    pub(crate) fn is_unicode(&self) -> bool {
        is_unicode_encoding(self.platform_id().unwrap(), self.encoding_id())
    }

    #[cfg(feature = "std")]
    #[inline(never)]
    pub(crate) fn name_from_utf16_be(&self) -> Option<String> {
        let mut name: Vec<u16> = Vec::new();
        for c in LazyArray16::<u16>::new(self.name()) {
            name.push(c);
        }

        String::from_utf16(&name).ok()
    }
}

#[cfg(feature = "std")]
impl<'a> core::fmt::Debug for Name<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // TODO: https://github.com/rust-lang/rust/issues/50264

        let name = self.name_utf8();
        f.debug_struct("Name")
            .field("name", &name.as_ref().map(core::ops::Deref::deref)
                                .unwrap_or("unsupported encoding"))
            .field("platform_id", &self.platform_id())
            .field("encoding_id", &self.encoding_id())
            .field("language_id", &self.language_id())
            .field("name_id", &self.name_id())
            .finish()
    }
}

#[cfg(not(feature = "std"))]
impl<'a> core::fmt::Debug for Name<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Name")
            .field("name", &self.name())
            .field("platform_id", &self.platform_id())
            .field("encoding_id", &self.encoding_id())
            .field("language_id", &self.language_id())
            .field("name_id", &self.name_id())
            .finish()
    }
}


/// An iterator over font's names.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Names<'a> {
    names: &'a [u8],
    storage: &'a [u8],
    index: u16,
    total: u16,
}

impl Default for Names<'_> {
    fn default() -> Self {
        Names {
            names: &[],
            storage: &[],
            index: 0,
            total: 0,
        }
    }
}

impl<'a> Names<'a> {
    fn new(names: &'a [u8], storage: &'a [u8], total: u16) -> Self {
        Names {
            names,
            storage,
            index: 0,
            total,
        }
    }
}

impl<'a> Iterator for Names<'a> {
    type Item = Name<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.total {
            self.index += 1;
            self.nth(usize::from(self.index) - 1)
        } else {
            None
        }
    }

    fn count(self) -> usize {
        usize::from(self.total)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let start = raw::NameRecord::SIZE * n;
        let end = start + raw::NameRecord::SIZE;
        let data = self.names.get(start..end)?;

        Some(Name {
            data: raw::NameRecord::new(data)?,
            strings: self.storage,
        })
    }
}


#[inline(never)]
pub(crate) fn parse(data: &[u8]) -> Option<Names> {
    // https://docs.microsoft.com/en-us/typography/opentype/spec/name#naming-table-format-1
    const LANG_TAG_RECORD_SIZE: u16 = 4;

    let mut s = Stream::new(data);
    let format: u16 = s.read()?;
    let count: u16 = s.read()?;
    s.skip::<u16>(); // offset

    if format == 0 {
        let names_data = s.read_bytes(raw::NameRecord::SIZE * usize::from(count))?;
        Some(Names::new(names_data, s.tail()?, count))
    } else if format == 1 {
        let lang_tag_count: u16 = s.read()?;
        let lang_tag_len = lang_tag_count.checked_mul(LANG_TAG_RECORD_SIZE)?;

        s.advance(usize::from(lang_tag_len)); // langTagRecords
        let names_data = s.read_bytes(raw::NameRecord::SIZE * usize::from(count))?;
        Some(Names::new(names_data, s.tail()?, count))
    } else {
        None
    }
}
