// Useful links:
// http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf
// http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5177.Type2.pdf
// https://github.com/opentypejs/opentype.js/blob/master/src/tables/cff.js

use core::ops::Range;

use crate::parser::{Stream, TryFromData, TrySlice, U24, FromData};
use crate::{Font, GlyphId, TableName, OutlineBuilder, Rect, Result, Error};

// Limits according to the Adobe Technical Note #5176, chapter 4 DICT Data.
const MAX_OPERANDS_LEN: usize = 48;

// Limits according to the Adobe Technical Note #5177 Appendix B.
const STACK_LIMIT: u8 = 10;
const MAX_ARGUMENTS_STACK_LEN: usize = 48;

const FLOAT_STACK_LEN: usize = 64;
const END_OF_FLOAT_FLAG: u8 = 0xf;

const TWO_BYTE_OPERATOR_MARK: u8 = 12;

/// Enumerates some operators defined in the Adobe Technical Note #5177.
mod operator {
    pub const HORIZONTAL_STEM: u8           = 1;
    pub const VERTICAL_STEM: u8             = 3;
    pub const VERTICAL_MOVE_TO: u8          = 4;
    pub const RELATIVE_LINE_TO: u8          = 5;
    pub const HORIZONTAL_LINE_TO: u8        = 6;
    pub const VERTICAL_LINE_TO: u8          = 7;
    pub const RELATIVE_CURVE_TO: u8         = 8;
    pub const CALL_LOCAL_SUBROUTINE: u8     = 10;
    pub const RETURN: u8                    = 11;
    pub const ENDCHAR: u8                   = 14;
    pub const HORIZONTAL_STEM_HINT_MASK: u8 = 18;
    pub const HINT_MASK: u8                 = 19;
    pub const COUNTER_MASK: u8              = 20;
    pub const RELATIVE_MOVE_TO: u8          = 21;
    pub const HORIZONTAL_MOVE_TO: u8        = 22;
    pub const VERTICAL_STEM_HINT_MASK: u8   = 23;
    pub const RELATIVE_CURVE_LINE: u8       = 24;
    pub const RELATIVE_LINE_CURVE: u8       = 25;
    pub const VV_CURVE_TO: u8               = 26;
    pub const HH_CURVE_TO: u8               = 27;
    pub const SHORT_INT: u8                 = 28;
    pub const CALL_GLOBAL_SUBROUTINE: u8    = 29;
    pub const VH_CURVE_TO: u8               = 30;
    pub const HV_CURVE_TO: u8               = 31;
    pub const HFLEX: u8                     = 34;
    pub const FLEX: u8                      = 35;
    pub const HFLEX1: u8                    = 36;
    pub const FLEX1: u8                     = 37;
}

/// Enumerates some operators defined in the Adobe Technical Note #5176,
/// Table 9 Top DICT Operator Entries
mod top_dict_operator {
    pub const CHAR_STRINGS_OFFSET: u16          = 17;
    pub const PRIVATE_DICT_SIZE_AND_OFFSET: u16 = 18;
}

/// Enumerates some operators defined in the Adobe Technical Note #5176,
/// Table 23 Private DICT Operators
mod private_dict_operator {
    pub const LOCAL_SUBROUTINES_OFFSET: u16 = 19;
}


/// A list of errors that can occur during a CFF table parsing.
#[derive(Clone, Copy, Debug)]
pub enum CFFError {
    /// The CFF table doesn't have any char strings.
    NoCharStrings,

    /// An invalid operand occurred.
    InvalidOperand,

    /// An invalid operator occurred.
    InvalidOperator,

    /// An unsupported operator occurred.
    UnsupportedOperator,

    /// Failed to parse a float number.
    InvalidFloat,

    /// The `OffSize` value must be in 1..4 range.
    ///
    /// Adobe Technical Note #5176, Table 2 CFF Data Types
    InvalidOffsetSize,

    /// Subroutines nesting is limited by 10.
    ///
    /// Adobe Technical Note #5177 Appendix B.
    NestingLimitReached,

    /// An arguments stack size is limited by 48 values.
    ///
    /// Adobe Technical Note #5177 Appendix B.
    ArgumentsStackLimitReached,

    /// Each operand expects a specific amount of arguments on the stack.
    ///
    /// Usually indicates an implementation error and should not occur on valid fonts.
    InvalidArgumentsStackLength,

    /// The `ItemVariationData` record should have format #1.
    ///
    /// CFF2 only.
    InvalidItemVariationDataFormat,

    /// No `ItemVariationData` with required index.
    ///
    /// CFF2 only.
    InvalidItemVariationDataIndex,
}

impl core::fmt::Display for CFFError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            CFFError::NoCharStrings => {
                write!(f, "table doesn't have any char strings")
            }
            CFFError::InvalidOperand => {
                write!(f, "an invalid operand occurred")
            }
            CFFError::InvalidOperator => {
                write!(f, "an invalid operator occurred")
            }
            CFFError::UnsupportedOperator => {
                write!(f, "an unsupported operator occurred")
            }
            CFFError::InvalidFloat => {
                write!(f, "failed to parse a float number")
            }
            CFFError::InvalidOffsetSize => {
                write!(f, "OffSize with an invalid value occurred")
            }
            CFFError::NestingLimitReached => {
                write!(f, "subroutines nesting limit reached")
            }
            CFFError::ArgumentsStackLimitReached => {
                write!(f, "arguments stack limit reached")
            }
            CFFError::InvalidArgumentsStackLength => {
                write!(f, "an invalid amount of items are in an arguments stack")
            }
            CFFError::InvalidItemVariationDataFormat => {
                write!(f, "invalid ItemVariationData format")
            }
            CFFError::InvalidItemVariationDataIndex => {
                write!(f, "no ItemVariationData with required index")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CFFError {}


#[derive(Clone, Copy, Default)]
pub struct Metadata<'a> {
    global_subrs: DataIndex<'a>,
    local_subrs: DataIndex<'a>,
    char_strings: DataIndex<'a>,
}

pub(crate) fn parse_metadata(data: &[u8]) -> Result<Metadata> {
    let mut s = Stream::new(data);

    // Parse Header.
    let major: u8 = s.read()?;
    s.skip::<u8>(); // minor
    let header_size: u8 = s.read()?;
    s.skip::<u8>(); // Absolute offset

    if major != 1 {
        return Err(Error::UnsupportedTableVersion(TableName::CompactFontFormat, major as u16));
    }

    // Jump to Name INDEX. It's not necessarily right after the header.
    if header_size > s.offset() as u8 {
        s.skip_len(header_size as u32 - s.offset() as u32);
    }

    // Skip Name INDEX.
    skip_index(&mut s)?;

    let (char_strings_offset, private_dict_range) = parse_top_dict(&mut s)?;

    // Must be set, otherwise there are nothing to parse.
    if char_strings_offset == 0 {
        return Err(CFFError::NoCharStrings.into());
    }

    let subroutines_offset = if let Some(range) = private_dict_range.clone() {
        parse_private_dict(data.try_slice(range)?)?
    } else {
        None
    };

    // Skip String INDEX.
    skip_index(&mut s)?;

    // Parse Global Subroutines INDEX.
    let mut metadata = Metadata::default();
    metadata.global_subrs = parse_index(&mut s)?;

    match (private_dict_range, subroutines_offset) {
        (Some(private_dict_range), Some(subroutines_offset)) => {
            // 'The local subroutines offset is relative to the beginning
            // of the Private DICT data.'
            if let Some(start) = private_dict_range.start.checked_add(subroutines_offset) {
                let data = data.try_slice(start..data.len())?;
                let mut s = Stream::new(data);
                metadata.local_subrs = parse_index(&mut s)?;
            }
        }
        _ => {}
    }

    metadata.char_strings = {
        let mut s = Stream::new_at(data, char_strings_offset);
        parse_index(&mut s)?
    };

    Ok(metadata)
}


impl<'a> Font<'a> {
    pub(crate) fn cff_glyph_outline(
        &self,
        metadata: &Metadata,
        glyph_id: GlyphId,
        builder: &mut dyn OutlineBuilder,
    ) -> Result<Rect> {
        parse_char_string(metadata, glyph_id, builder)
    }
}

fn parse_top_dict(s: &mut Stream) -> Result<(usize, Option<Range<usize>>)> {
    let mut char_strings_offset = 0;
    let mut private_dict_range = None;

    let index = parse_index(s)?;

    // The Top DICT INDEX should have only one dictionary.
    let data = match index.get(0) {
        Some(v) => v,
        None => return Err(CFFError::NoCharStrings.into()),
    };

    let mut dict_parser = DictionaryParser::new(data);
    while let Some(operator) = dict_parser.parse_next() {
        match operator.value() {
            top_dict_operator::CHAR_STRINGS_OFFSET => {
                dict_parser.parse_operands()?;
                let operands = dict_parser.operands();

                if operands.len() == 1 {
                    char_strings_offset = operands[0].as_i32() as usize;
                }
            }
            top_dict_operator::PRIVATE_DICT_SIZE_AND_OFFSET => {
                dict_parser.parse_operands()?;
                let operands = dict_parser.operands();

                if operands.len() == 2 {
                    let len = operands[0].as_i32() as usize;
                    let start = operands[1].as_i32() as usize;
                    if let Some(end) = start.checked_add(len) {
                        private_dict_range = Some(start..end);
                    }
                }
            }
            _ => {}
        }

        if char_strings_offset != 0 && private_dict_range.is_some() {
            break;
        }
    }

    Ok((char_strings_offset, private_dict_range))
}

fn parse_private_dict(data: &[u8]) -> Result<Option<usize>> {
    let mut subroutines_offset = None;
    let mut dict_parser = DictionaryParser::new(data);
    while let Some(operator) = dict_parser.parse_next() {
        if operator.value() == private_dict_operator::LOCAL_SUBROUTINES_OFFSET {
            dict_parser.parse_operands()?;
            let operands = dict_parser.operands();

            if operands.len() == 1 {
                subroutines_offset = Some(operands[0].as_i32() as usize);
            }

            break;
        }
    }

    Ok(subroutines_offset)
}

struct CharStringParserContext<'a> {
    metadata: &'a Metadata<'a>,
    is_first_move_to: bool,
    width_parsed: bool,
    stems_len: u32,
}

fn parse_char_string(
    metadata: &Metadata,
    glyph_id: GlyphId,
    builder: &mut dyn OutlineBuilder,
) -> Result<Rect> {
    let data = metadata.char_strings.get(glyph_id.0).ok_or(Error::NoGlyph)?;

    let mut ctx = CharStringParserContext {
        metadata,
        is_first_move_to: true,
        width_parsed: false,
        stems_len: 0,
    };

    let mut inner_builder = Builder {
        builder,
        bbox: RectF {
            x_min: core::f32::MAX,
            y_min: core::f32::MAX,
            x_max: core::f32::MIN,
            y_max: core::f32::MIN,
        }
    };

    let mut stack = ArgumentsStack {
        data: &mut [0.0; MAX_ARGUMENTS_STACK_LEN],
        len: 0,
        max_len: MAX_ARGUMENTS_STACK_LEN,
    };
    let _ = _parse_char_string(&mut ctx, data, 0.0, 0.0, &mut stack, 0, &mut inner_builder)?;

    let bbox = inner_builder.bbox;
    Ok(Rect {
        x_min: bbox.x_min as i16,
        y_min: bbox.y_min as i16,
        x_max: bbox.x_max as i16,
        y_max: bbox.y_max as i16,
    })
}


pub struct RectF {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

pub trait OutlineBuilderInner {
    fn update_bbox(&mut self, x: f32, y: f32);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    fn close(&mut self);
}

pub struct Builder<'a> {
    pub builder: &'a mut dyn OutlineBuilder,
    pub bbox: RectF,
}

impl<'a> OutlineBuilderInner for Builder<'a> {
    #[inline]
    fn update_bbox(&mut self, x: f32, y: f32) {
        self.bbox.x_min = self.bbox.x_min.min(x);
        self.bbox.y_min = self.bbox.y_min.min(y);

        self.bbox.x_max = self.bbox.x_max.max(x);
        self.bbox.y_max = self.bbox.y_max.max(y);
    }

    #[inline]
    fn move_to(&mut self, x: f32, y: f32) {
        self.update_bbox(x, y);
        self.builder.move_to(x, y);
    }

    #[inline]
    fn line_to(&mut self, x: f32, y: f32) {
        self.update_bbox(x, y);
        self.builder.line_to(x, y);
    }

    #[inline]
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.update_bbox(x1, y1);
        self.update_bbox(x2, y2);
        self.update_bbox(x, y);
        self.builder.curve_to(x1, y1, x2, y2, x, y);
    }

    #[inline]
    fn close(&mut self) {
        self.builder.close();
    }
}

fn _parse_char_string(
    ctx: &mut CharStringParserContext,
    char_string: &[u8],
    mut x: f32,
    mut y: f32,
    stack: &mut ArgumentsStack,
    depth: u8,
    builder: &mut Builder,
) -> Result<(f32, f32)> {
    let mut s = Stream::new(char_string);

    while !s.at_end() {
        let op: u8 = s.read()?;
        match op {
            0 | 2 | 9 | 13 | 15 | 16 | 17 => {
                // Reserved.
                return Err(CFFError::InvalidOperator.into());
            }
            operator::HORIZONTAL_STEM |
            operator::VERTICAL_STEM |
            operator::HORIZONTAL_STEM_HINT_MASK |
            operator::VERTICAL_STEM_HINT_MASK => {
                // y dy {dya dyb}* hstem
                // x dx {dxa dxb}* vstem
                // y dy {dya dyb}* hstemhm
                // x dx {dxa dxb}* vstemhm

                // If the stack length is uneven, than the first value is a `width`.
                let len = if stack.len().is_odd() && !ctx.width_parsed {
                    ctx.width_parsed = true;
                    stack.len() - 1
                } else {
                    stack.len()
                };

                ctx.stems_len += len as u32 >> 1;

                // We are ignoring the hint operators.
                stack.clear();
            }
            operator::VERTICAL_MOVE_TO => {
                // dy1

                let mut i = 0;
                if stack.len() == 2 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                y += stack.at(i);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::RELATIVE_LINE_TO => {
                // {dxa dya}+

                if stack.len().is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() {
                    x += stack.at(i + 0);
                    y += stack.at(i + 1);
                    builder.line_to(x, y);
                    i += 2;
                }

                stack.clear();
            }
            operator::HORIZONTAL_LINE_TO => {
                // dx1 {dya dxb}*
                //     {dxa dyb}+

                let mut i = 0;
                while i < stack.len() {
                    x += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);

                    if i == stack.len() {
                        break;
                    }

                    y += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);
                }

                stack.clear();
            }
            operator::VERTICAL_LINE_TO => {
                // dy1 {dxa dyb}*
                //     {dya dxb}+

                let mut i = 0;
                while i < stack.len() {
                    y += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);

                    if i == stack.len() {
                        break;
                    }

                    x += stack.at(i);
                    i += 1;
                    builder.line_to(x, y);
                }

                stack.clear();
            }
            operator::RELATIVE_CURVE_TO => {
                // {dxa dya dxb dyb dxc dyc}+

                if stack.len() % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y + stack.at(i + 1);
                    let x2 = x1 + stack.at(i + 2);
                    let y2 = y1 + stack.at(i + 3);
                    x = x2 + stack.at(i + 4);
                    y = y2 + stack.at(i + 5);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 6;
                }

                stack.clear();
            }
            operator::CALL_LOCAL_SUBROUTINE => {
                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached.into());
                }

                let subroutine_bias = calc_subroutine_bias(ctx.metadata.local_subrs.len() as u16);
                let index = stack.pop() as i32 + subroutine_bias as i32;
                let char_string = ctx.metadata.local_subrs.get(index as u16).ok_or(Error::NoGlyph)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            operator::RETURN => {
                break;
            }
            TWO_BYTE_OPERATOR_MARK => {
                // flex
                let op2: u8 = s.read()?;
                match op2 {
                    operator::HFLEX => {
                        // dx1 dx2 dy2 dx3 dx4 dx5 dx6

                        if stack.len() != 7 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y;
                        let dx2 = dx1 + stack.at(1);
                        let dy2 = dy1 + stack.at(2);
                        let dx3 = dx2 + stack.at(3);
                        let dy3 = dy2;
                        let dx4 = dx3 + stack.at(4);
                        let dy4 = dy2;
                        let dx5 = dx4 + stack.at(5);
                        let dy5 = y;
                        x = dx5 + stack.at(6);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    operator::FLEX => {
                        // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 dx6 dy6 fd

                        if stack.len() != 13 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2 + stack.at(5);
                        let dx4 = dx3 + stack.at(6);
                        let dy4 = dy3 + stack.at(7);
                        let dx5 = dx4 + stack.at(8);
                        let dy5 = dy4 + stack.at(9);
                        x = dx5 + stack.at(10);
                        y = dy5 + stack.at(11);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    operator::HFLEX1 => {
                        // dx1 dy1 dx2 dy2 dx3 dx4 dx5 dy5 dx6

                        if stack.len() != 9 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2;
                        let dx4 = dx3 + stack.at(5);
                        let dy4 = dy2;
                        let dx5 = dx4 + stack.at(6);
                        let dy5 = dy4 + stack.at(7);
                        x = dx5 + stack.at(8);
                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    operator::FLEX1 => {
                        // dx1 dy1 dx2 dy2 dx3 dy3 dx4 dy4 dx5 dy5 d6

                        if stack.len() != 11 {
                            return Err(CFFError::InvalidArgumentsStackLength.into());
                        }

                        let dx1 = x + stack.at(0);
                        let dy1 = y + stack.at(1);
                        let dx2 = dx1 + stack.at(2);
                        let dy2 = dy1 + stack.at(3);
                        let dx3 = dx2 + stack.at(4);
                        let dy3 = dy2 + stack.at(5);
                        let dx4 = dx3 + stack.at(6);
                        let dy4 = dy3 + stack.at(7);
                        let dx5 = dx4 + stack.at(8);
                        let dy5 = dy4 + stack.at(9);

                        if f32_abs(dx5 - x) > f32_abs(dy5 - y) {
                            x = dx5 + stack.at(10);
                        } else {
                            y = dy5 + stack.at(10);
                        }

                        builder.curve_to(dx1, dy1, dx2, dy2, dx3, dy3);
                        builder.curve_to(dx4, dy4, dx5, dy5, x, y);

                        stack.clear();
                    }
                    _ => {
                        return Err(CFFError::UnsupportedOperator.into());
                    }
                }
            }
            operator::ENDCHAR => {
                if !stack.is_empty() && !ctx.width_parsed {
                    stack.clear();
                    ctx.width_parsed = true;
                }

                if !ctx.is_first_move_to {
                    ctx.is_first_move_to = true;
                    builder.close();
                }
            }
            operator::HINT_MASK | operator::COUNTER_MASK => {
                let mut len = stack.len();

                // We are ignoring the hint operators.
                stack.clear();

                // If the stack length is uneven, than the first value is a `width`.
                if len.is_odd() && !ctx.width_parsed {
                    len -= 1;
                    ctx.width_parsed = true;
                }

                ctx.stems_len += len as u32 >> 1;

                s.skip_len((ctx.stems_len + 7) >> 3);
            }
            operator::RELATIVE_MOVE_TO => {
                // dx1 dy1

                let mut i = 0;
                if stack.len() == 3 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 2 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                x += stack.at(i + 0);
                y += stack.at(i + 1);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::HORIZONTAL_MOVE_TO => {
                // dx1

                let mut i = 0;
                if stack.len() == 2 && !ctx.width_parsed {
                    i += 1;
                    ctx.width_parsed = true;
                } else if stack.len() != 1 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if ctx.is_first_move_to {
                    ctx.is_first_move_to = false;
                } else {
                    builder.close();
                }

                x += stack.at(i);
                builder.move_to(x, y);

                stack.clear();
            }
            operator::RELATIVE_CURVE_LINE => {
                // {dxa dya dxb dyb dxc dyc}+ dxd dyd

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if (stack.len() - 2) % 6 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() - 2 {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y + stack.at(i + 1);
                    let x2 = x1 + stack.at(i + 2);
                    let y2 = y1 + stack.at(i + 3);
                    x = x2 + stack.at(i + 4);
                    y = y2 + stack.at(i + 5);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 6;
                }

                x += stack.at(i + 0);
                y += stack.at(i + 1);
                builder.line_to(x, y);

                stack.clear();
            }
            operator::RELATIVE_LINE_CURVE => {
                // {dxa dya}+ dxb dyb dxc dyc dxd dyd

                if stack.len() < 8 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if (stack.len() - 6).is_odd() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                let mut i = 0;
                while i < stack.len() - 6 {
                    x += stack.at(i + 0);
                    y += stack.at(i + 1);

                    builder.line_to(x, y);
                    i += 2;
                }

                let x1 = x + stack.at(i + 0);
                let y1 = y + stack.at(i + 1);
                let x2 = x1 + stack.at(i + 2);
                let y2 = y1 + stack.at(i + 3);
                x = x2 + stack.at(i + 4);
                y = y2 + stack.at(i + 5);
                builder.curve_to(x1, y1, x2, y2, x, y);

                stack.clear();
            }
            operator::VV_CURVE_TO => {
                // dx1? {dya dxb dyb dyc}+

                let mut i = 0;

                // The odd argument count indicates an X position.
                if stack.len().is_odd() {
                    x += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                while i < stack.len() {
                    let x1 = x;
                    let y1 = y + stack.at(i + 0);
                    let x2 = x1 + stack.at(i + 1);
                    let y2 = y1 + stack.at(i + 2);
                    x = x2;
                    y = y2 + stack.at(i + 3);

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 4;
                }

                stack.clear();
            }
            operator::HH_CURVE_TO => {
                // dy1? {dxa dxb dyb dxc}+

                let mut i = 0;

                // The odd argument count indicates an Y position.
                if stack.len().is_odd() {
                    y += stack.at(0);
                    i += 1;
                }

                if (stack.len() - i) % 4 != 0 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                while i < stack.len() {
                    let x1 = x + stack.at(i + 0);
                    let y1 = y;
                    let x2 = x1 + stack.at(i + 1);
                    let y2 = y1 + stack.at(i + 2);
                    x = x2 + stack.at(i + 3);
                    y = y2;

                    builder.curve_to(x1, y1, x2, y2, x, y);
                    i += 4;
                }

                stack.clear();
            }
            operator::SHORT_INT => {
                let b1 = s.read::<u8>()? as i32;
                let b2 = s.read::<u8>()? as i32;
                let n = ((b1 << 24) | (b2 << 16)) >> 16;
                debug_assert!((-32768..=32767).contains(&n));
                stack.push(n as f32)?;
            }
            operator::CALL_GLOBAL_SUBROUTINE => {
                if stack.is_empty() {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                if depth == STACK_LIMIT {
                    return Err(CFFError::NestingLimitReached.into());
                }

                let subroutine_bias = calc_subroutine_bias(ctx.metadata.global_subrs.len() as u16);
                let index = stack.pop() as i32 + subroutine_bias as i32;
                let char_string = ctx.metadata.global_subrs.get(index as u16).ok_or(Error::NoGlyph)?;
                let pos = _parse_char_string(ctx, char_string, x, y, stack, depth + 1, builder)?;
                x = pos.0;
                y = pos.1;
            }
            operator::VH_CURVE_TO => {
                // dy1 dx2 dy2 dx3 {dxa dxb dyb dyc dyd dxe dye dxf}* dyf?
                //                 {dya dxb dyb dxc dxd dxe dye dyf}+ dxf?

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x;
                    let y1 = y + stack.pop();
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    x = x2 + stack.pop();
                    y = y2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                    if stack.is_empty() {
                        break;
                    }

                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x + stack.pop();
                    let y1 = y;
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    y = y2 + stack.pop();
                    x = x2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                }

                debug_assert!(stack.is_empty());
            }
            operator::HV_CURVE_TO => {
                // dx1 dx2 dy2 dy3 {dya dxb dyb dxc dxd dxe dye dyf}* dxf?
                //                 {dxa dxb dyb dyc dyd dxe dye dxf}+ dyf?

                if stack.len() < 4 {
                    return Err(CFFError::InvalidArgumentsStackLength.into());
                }

                stack.reverse();
                while !stack.is_empty() {
                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x + stack.pop();
                    let y1 = y;
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    y = y2 + stack.pop();
                    x = x2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                    if stack.is_empty() {
                        break;
                    }

                    if stack.len() < 4 {
                        return Err(CFFError::InvalidArgumentsStackLength.into());
                    }

                    let x1 = x;
                    let y1 = y + stack.pop();
                    let x2 = x1 + stack.pop();
                    let y2 = y1 + stack.pop();
                    x = x2 + stack.pop();
                    y = y2 + if stack.len() == 1 { stack.pop() } else { 0.0 };
                    builder.curve_to(x1, y1, x2, y2, x, y);
                }

                debug_assert!(stack.is_empty());
            }
            32..=246 => {
                let n = op as i32 - 139;
                stack.push(n as f32)?;
            }
            247..=250 => {
                let b1 = s.read::<u8>()? as i32;
                let n = (op as i32 - 247) * 256 + b1 + 108;
                debug_assert!((108..=1131).contains(&n));
                stack.push(n as f32)?;
            }
            251..=254 => {
                let b1 = s.read::<u8>()? as i32;
                let n = -(op as i32 - 251) * 256 - b1 - 108;
                debug_assert!((-1131..=-108).contains(&n));
                stack.push(n as f32)?;
            }
            255 => {
                let n = s.read::<u32>()? as i32 as f32 / 65536.0;
                stack.push(n)?;
            }
        }
    }

    Ok((x, y))
}

// Adobe Technical Note #5176, Chapter 16 "Local / Global Subrs INDEXes"
#[inline]
pub fn calc_subroutine_bias(len: u16) -> u16 {
    if len < 1240 {
        107
    } else if len < 33900 {
        1131
    } else {
        32768
    }
}

fn parse_index<'a>(s: &mut Stream<'a>) -> Result<DataIndex<'a>> {
    let count: u16 = s.read()?;
    if count != 0 && count != core::u16::MAX {
        parse_index_impl(count as u32, s)
    } else {
        Ok(DataIndex::default())
    }
}

pub fn parse_index_impl<'a>(count: u32, s: &mut Stream<'a>) -> Result<DataIndex<'a>> {
    let offset_size: OffsetSize = s.try_read()?;
    let offsets_len = (count + 1) * offset_size as u32;
    let offsets = VarOffsets {
        data: &s.read_bytes(offsets_len)?,
        offset_size,
    };

    match offsets.last() {
        Some(last_offset) => {
            let data = s.read_bytes(last_offset)?;
            Ok(DataIndex { data, offsets })
        }
        None => {
            Ok(DataIndex::default())
        }
    }
}

fn skip_index(s: &mut Stream) -> Result<()> {
    let count: u16 = s.read()?;
    if count != 0 && count != core::u16::MAX {
        let offset_size: OffsetSize = s.try_read()?;
        let offsets_len = (count + 1) as u32 * offset_size as u32;
        let offsets = VarOffsets {
            data: &s.read_bytes(offsets_len)?,
            offset_size,
        };

        if let Some(last_offset) = offsets.last() {
            s.skip_len(last_offset);
        }
    }

    Ok(())
}


#[derive(Clone, Copy)]
pub struct VarOffsets<'a> {
    pub data: &'a [u8],
    pub offset_size: OffsetSize,
}

impl<'a> VarOffsets<'a> {
    pub fn get(&self, index: u16) -> Option<u32> {
        if index >= self.len() {
            return None;
        }

        let start = index as usize * self.offset_size as usize;
        let end = start + self.offset_size as usize;
        let data = self.data.try_slice(start..end).ok()?;
        let n: u32 = match self.offset_size {
            OffsetSize::Size1 => u8::parse(data) as u32,
            OffsetSize::Size2 => u16::parse(data) as u32,
            OffsetSize::Size3 => U24::parse(data).0,
            OffsetSize::Size4 => u32::parse(data),
        };

        // Offset must be positive.
        if n == 0 {
            return None;
        }

        // Offsets are offset by one byte in the font,
        // so we have to shift them back.
        Some(n - 1)
    }

    #[inline]
    pub fn last(&self) -> Option<u32> {
        if !self.is_empty() {
            self.get(self.len() - 1)
        } else {
            None
        }
    }

    #[inline]
    pub fn len(&self) -> u16 {
        self.data.len() as u16 / self.offset_size as u16
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


#[derive(Clone, Copy)]
pub struct DataIndex<'a> {
    pub data: &'a [u8],
    pub offsets: VarOffsets<'a>,
}

impl<'a> Default for DataIndex<'a> {
    #[inline]
    fn default() -> Self {
        DataIndex {
            data: b"",
            offsets: VarOffsets { data: b"", offset_size: OffsetSize::Size1 },
        }
    }
}

impl<'a> IntoIterator for DataIndex<'a> {
    type Item = &'a [u8];
    type IntoIter = DataIndexIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        DataIndexIter {
            data: self,
            offset: 0,
        }
    }
}

impl<'a> DataIndex<'a> {
    #[inline]
    pub fn len(&self) -> u16 {
        if !self.offsets.is_empty() {
            // Last offset points to the byte after the `Object data`.
            // We should skip it.
            self.offsets.len() - 1
        } else {
            0
        }
    }

    pub fn get(&self, index: u16) -> Option<&'a [u8]> {
        // Check for overflow first.
        if index == core::u16::MAX {
            None
        } else if index + 1 < self.offsets.len() {
            let start = self.offsets.get(index)? as usize;
            let end = self.offsets.get(index + 1)? as usize;
            let data = self.data.try_slice(start..end).ok()?;
            Some(data)
        } else {
            None
        }
    }
}

pub struct DataIndexIter<'a> {
    data: DataIndex<'a>,
    offset: u16,
}

impl<'a> Iterator for DataIndexIter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.data.len() {
            return None;
        }

        let index = self.offset;
        self.offset += 1;
        self.data.get(index)
    }
}


#[derive(Clone, Copy, Debug)]
pub enum OffsetSize {
    Size1 = 1,
    Size2 = 2,
    Size3 = 3,
    Size4 = 4,
}

impl TryFromData for OffsetSize {
    #[inline]
    fn try_parse(data: &[u8]) -> Result<Self> {
        match u8::parse(data) {
            1 => Ok(OffsetSize::Size1),
            2 => Ok(OffsetSize::Size2),
            3 => Ok(OffsetSize::Size3),
            4 => Ok(OffsetSize::Size4),
            _ => Err(CFFError::InvalidOffsetSize.into()),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Operator(pub u16);

impl Operator {
    #[inline]
    pub fn value(&self) -> u16 { self.0 }
}


struct DictionaryParser<'a> {
    data: &'a [u8],
    // The current offset.
    offset: usize,
    // Offset to the last operands start.
    operands_offset: usize,
    // Actual operands.
    operands: [Number; MAX_OPERANDS_LEN], // 192B
    // An amount of operands in the `operands` array.
    operands_len: u8,
}

impl<'a> DictionaryParser<'a> {
    #[inline]
    fn new(data: &'a [u8]) -> Self {
        DictionaryParser {
            data,
            offset: 0,
            operands_offset: 0,
            operands: [Number::Integer(0); MAX_OPERANDS_LEN],
            operands_len: 0,
        }
    }

    #[inline(never)]
    fn parse_next(&mut self) -> Option<Operator> {
        let mut s = Stream::new_at(self.data, self.offset);
        self.operands_offset = self.offset;
        while !s.at_end() {
            let b: u8 = s.read().ok()?;
            // 0..=21 bytes are operators.
            if b <= 21 {
                let mut operator = b as u16;

                // Check that operator is two byte long.
                if b == TWO_BYTE_OPERATOR_MARK {
                    // Use a 1200 'prefix' to make two byte operators more readable.
                    // 12 3 => 1203
                    operator = 1200 + s.read::<u8>().ok()? as u16;
                }

                self.offset = s.offset();
                return Some(Operator(operator));
            } else {
                skip_number(b, &mut s)?;
            }
        }

        None
    }

    /// Parses operands of the current operator.
    ///
    /// In the DICT structure, operands are defined before an operator.
    /// So we are trying to find an operator first and the we can actually parse the operands.
    ///
    /// Since this methods is pretty expensive and we do not care about most of the operators,
    /// we can speed up parsing by parsing operands only for required operators.
    ///
    /// We still have to "skip" operands during operators search (see `skip_number()`),
    /// but it's still faster that a naive method.
    fn parse_operands(&mut self) -> Result<()> {
        let mut s = Stream::new_at(self.data, self.operands_offset);
        self.operands_len = 0;
        while !s.at_end() {
            let b: u8 = s.read()?;
            // 0..=21 bytes are operators.
            if b <= 21 {
                break;
            } else {
                let op = parse_number(b, &mut s)?;
                self.operands[self.operands_len as usize] = op;
                self.operands_len += 1;

                if self.operands_len >= MAX_OPERANDS_LEN as u8 {
                    break;
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn operands(&self) -> &[Number] {
        &self.operands[..self.operands_len as usize]
    }
}

// Adobe Technical Note #5177, Table 3 Operand Encoding
pub fn parse_number(b0: u8, s: &mut Stream) -> Result<Number> {
    match b0 {
        28 => {
            let n = s.read::<u16>()? as i32;
            Ok(Number::Integer(n))
        }
        29 => {
            let n = s.read::<u32>()? as i32;
            Ok(Number::Integer(n))
        }
        30 => {
            parse_float(s)
        }
        32..=246 => {
            let n = b0 as i32 - 139;
            Ok(Number::Integer(n))
        }
        247..=250 => {
            let b1 = s.read::<u8>()? as i32;
            let n = (b0 as i32 - 247) * 256 + b1 + 108;
            Ok(Number::Integer(n))
        }
        251..=254 => {
            let b1 = s.read::<u8>()? as i32;
            let n = -(b0 as i32 - 251) * 256 - b1 - 108;
            Ok(Number::Integer(n))
        }
        _ => Err(CFFError::InvalidOperand.into()),
    }
}

fn parse_float(s: &mut Stream) -> Result<Number> {
    let mut data = [0u8; FLOAT_STACK_LEN];
    let mut idx = 0;

    loop {
        let b1: u8 = s.read()?;
        let nibble1 = b1 >> 4;
        let nibble2 = b1 & 15;

        if nibble1 == END_OF_FLOAT_FLAG {
            break;
        }

        idx = parse_float_nibble(nibble1, idx, &mut data)?;

        if nibble2 == END_OF_FLOAT_FLAG {
            break;
        }

        idx = parse_float_nibble(nibble2, idx, &mut data)?;
    }

    let s = core::str::from_utf8(&data[..idx]).map_err(|_| CFFError::InvalidFloat)?;
    let n = s.parse().map_err(|_| CFFError::InvalidFloat)?;
    Ok(Number::Float(n))
}

// Adobe Technical Note #5176, Table 5 Nibble Definitions
fn parse_float_nibble(nibble: u8, mut idx: usize, data: &mut [u8]) -> Result<usize> {
    if idx == FLOAT_STACK_LEN {
        return Err(CFFError::InvalidFloat.into());
    }

    match nibble {
        0..=9 => {
            data[idx] = b'0' + nibble;
        }
        10 => {
            data[idx] = b'.';
        }
        11 => {
            data[idx] = b'E';
        }
        12 => {
            if idx + 1 == FLOAT_STACK_LEN {
                return Err(CFFError::InvalidFloat.into());
            }

            data[idx] = b'E';
            idx += 1;
            data[idx] = b'-';
        }
        13 => {
            return Err(CFFError::InvalidFloat.into());
        }
        14 => {
            data[idx] = b'-';
        }
        _ => {
            return Err(CFFError::InvalidFloat.into());
        }
    }

    idx += 1;
    Ok(idx)
}

// Just like `parse_number`, but doesn't actually parses the data.
pub fn skip_number(b0: u8, s: &mut Stream) -> Option<()> {
    match b0 {
        28 => s.skip::<u16>(),
        29 => s.skip::<u32>(),
        30 => {
            while !s.at_end() {
                let b1: u8 = s.read().ok()?;
                let nibble1 = b1 >> 4;
                let nibble2 = b1 & 15;
                if nibble1 == END_OF_FLOAT_FLAG || nibble2 == END_OF_FLOAT_FLAG {
                    break;
                }
            }
        }
        32..=246 => {}
        247..=250 => s.skip::<u8>(),
        251..=254 => s.skip::<u8>(),
        _ => return None,
    }

    Some(())
}


#[derive(Clone, Copy, Debug)]
pub enum Number {
    Integer(i32),
    Float(f32),
}

impl Number {
    #[inline]
    pub fn as_i32(&self) -> i32 {
        match *self {
            Number::Integer(n) => n,
            Number::Float(n) => n as i32,
        }
    }
}


pub struct ArgumentsStack<'a> {
    pub data: &'a mut [f32],
    pub len: usize,
    pub max_len: usize,
}

impl<'a> ArgumentsStack<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn push(&mut self, n: f32) -> Result<()> {
        if self.len == self.max_len {
            Err(CFFError::ArgumentsStackLimitReached.into())
        } else {
            self.data[self.len] = n;
            self.len += 1;
            Ok(())
        }
    }

    #[inline]
    pub fn at(&self, index: usize) -> f32 {
        self.data[index]
    }

    #[inline]
    pub fn pop(&mut self) -> f32 {
        debug_assert!(!self.is_empty());
        self.len -= 1;
        self.data[self.len]
    }

    #[inline]
    pub fn remove_last_n(&mut self, n: usize) {
        assert!(n < self.len);
        self.len -= n;
    }

    #[inline]
    pub fn reverse(&mut self) {
        if self.is_empty() {
            return;
        }

        // Reverse only the actual data and not the whole stack.
        let (first, _) = self.data.split_at_mut(self.len);
        first.reverse();
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl core::fmt::Debug for ArgumentsStack<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_list().entries(&self.data[..self.len]).finish()
    }
}


pub trait IsEven {
    fn is_even(&self) -> bool;
    fn is_odd(&self) -> bool;
}

impl IsEven for usize {
    #[inline]
    fn is_even(&self) -> bool { (*self) & 1 == 0 }

    #[inline]
    fn is_odd(&self) -> bool { !self.is_even() }
}

#[cfg(feature = "std")]
#[inline]
pub fn f32_abs(n: f32) -> f32 {
    n.abs()
}

#[cfg(not(feature = "std"))]
#[inline]
pub fn f32_abs(n: f32) -> f32 {
    if n.is_sign_negative() { -n } else { n }
}
