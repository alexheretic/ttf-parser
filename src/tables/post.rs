// https://docs.microsoft.com/en-us/typography/opentype/spec/post

use crate::{LineMetrics, GlyphId};
use crate::parser::{Stream, LazyArray16};
use crate::raw::post as raw;


// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6post.html
const MACINTOSH_NAMES: &[&str] = &[
    ".notdef",
    ".null",
    "nonmarkingreturn",
    "space",
    "exclam",
    "quotedbl",
    "numbersign",
    "dollar",
    "percent",
    "ampersand",
    "quotesingle",
    "parenleft",
    "parenright",
    "asterisk",
    "plus",
    "comma",
    "hyphen",
    "period",
    "slash",
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "colon",
    "semicolon",
    "less",
    "equal",
    "greater",
    "question",
    "at",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "bracketleft",
    "backslash",
    "bracketright",
    "asciicircum",
    "underscore",
    "grave",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "braceleft",
    "bar",
    "braceright",
    "asciitilde",
    "Adieresis",
    "Aring",
    "Ccedilla",
    "Eacute",
    "Ntilde",
    "Odieresis",
    "Udieresis",
    "aacute",
    "agrave",
    "acircumflex",
    "adieresis",
    "atilde",
    "aring",
    "ccedilla",
    "eacute",
    "egrave",
    "ecircumflex",
    "edieresis",
    "iacute",
    "igrave",
    "icircumflex",
    "idieresis",
    "ntilde",
    "oacute",
    "ograve",
    "ocircumflex",
    "odieresis",
    "otilde",
    "uacute",
    "ugrave",
    "ucircumflex",
    "udieresis",
    "dagger",
    "degree",
    "cent",
    "sterling",
    "section",
    "bullet",
    "paragraph",
    "germandbls",
    "registered",
    "copyright",
    "trademark",
    "acute",
    "dieresis",
    "notequal",
    "AE",
    "Oslash",
    "infinity",
    "plusminus",
    "lessequal",
    "greaterequal",
    "yen",
    "mu",
    "partialdiff",
    "summation",
    "product",
    "pi",
    "integral",
    "ordfeminine",
    "ordmasculine",
    "Omega",
    "ae",
    "oslash",
    "questiondown",
    "exclamdown",
    "logicalnot",
    "radical",
    "florin",
    "approxequal",
    "Delta",
    "guillemotleft",
    "guillemotright",
    "ellipsis",
    "nonbreakingspace",
    "Agrave",
    "Atilde",
    "Otilde",
    "OE",
    "oe",
    "endash",
    "emdash",
    "quotedblleft",
    "quotedblright",
    "quoteleft",
    "quoteright",
    "divide",
    "lozenge",
    "ydieresis",
    "Ydieresis",
    "fraction",
    "currency",
    "guilsinglleft",
    "guilsinglright",
    "fi",
    "fl",
    "daggerdbl",
    "periodcentered",
    "quotesinglbase",
    "quotedblbase",
    "perthousand",
    "Acircumflex",
    "Ecircumflex",
    "Aacute",
    "Edieresis",
    "Egrave",
    "Iacute",
    "Icircumflex",
    "Idieresis",
    "Igrave",
    "Oacute",
    "Ocircumflex",
    "apple",
    "Ograve",
    "Uacute",
    "Ucircumflex",
    "Ugrave",
    "dotlessi",
    "circumflex",
    "tilde",
    "macron",
    "breve",
    "dotaccent",
    "ring",
    "cedilla",
    "hungarumlaut",
    "ogonek",
    "caron",
    "Lslash",
    "lslash",
    "Scaron",
    "scaron",
    "Zcaron",
    "zcaron",
    "brokenbar",
    "Eth",
    "eth",
    "Yacute",
    "yacute",
    "Thorn",
    "thorn",
    "minus",
    "multiply",
    "onesuperior",
    "twosuperior",
    "threesuperior",
    "onehalf",
    "onequarter",
    "threequarters",
    "franc",
    "Gbreve",
    "gbreve",
    "Idotaccent",
    "Scedilla",
    "scedilla",
    "Cacute",
    "cacute",
    "Ccaron",
    "ccaron",
    "dcroat",
];


#[derive(Clone, Copy)]
pub struct Table<'a> {
    underline: LineMetrics,
    name_indexes: LazyArray16<'a, u16>,
    names: &'a [u8],
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < raw::TABLE_SIZE {
            return None;
        }

        let version: u32 = Stream::new(data).read()?;
        if !(version == 0x00010000 || version == 0x00020000 ||
            version == 0x00025000 || version == 0x00030000 ||
            version == 0x00040000)
        {
            return None;
        }

        let underline = LineMetrics {
            position: Stream::read_at(data, raw::UNDERLINE_POSITION_OFFSET)?,
            thickness: Stream::read_at(data, raw::UNDERLINE_THICKNESS_OFFSET)?,
        };

        let mut name_indexes = LazyArray16::default();
        let mut names: &[u8] = &[];

        // Only version 2.0 of the table has data at the end.
        if version == 0x00020000 {
            let mut s = Stream::new_at(data, raw::TABLE_SIZE)?;
            let count: u16 = s.read()?;
            name_indexes = s.read_array16(count)?;
            names = s.tail()?;
        }

        Some(Table {
            underline,
            name_indexes,
            names,
        })
    }

    #[inline]
    pub fn underline_metrics(&self) -> LineMetrics {
        self.underline
    }

    #[inline]
    pub fn glyph_name(&self, glyph_id: GlyphId) -> Option<&'a str> {
        let mut index = self.name_indexes.get(glyph_id.0)?;

        // 'If the name index is between 0 and 257, treat the name index
        // as a glyph index in the Macintosh standard order.'
        if usize::from(index) < MACINTOSH_NAMES.len() {
            Some(MACINTOSH_NAMES[usize::from(index)])
        } else {
            // 'If the name index is between 258 and 65535, then subtract 258 and use that
            // to index into the list of Pascal strings at the end of the table.'
            index -= MACINTOSH_NAMES.len() as u16;

            let mut s = Stream::new(self.names);
            let mut i = 0;
            while !s.at_end() && i < core::u16::MAX {
                let len: u8 = s.read()?;

                if i == index {
                    if len == 0 {
                        // Empty name is an error.
                        break;
                    } else {
                        let name = s.read_bytes(usize::from(len))?;
                        return core::str::from_utf8(name).ok();
                    }
                } else {
                    s.advance(usize::from(len));
                }

                i += 1;
            }

            None
        }
    }
}
