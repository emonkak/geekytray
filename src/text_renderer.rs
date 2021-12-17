use std::collections::hash_map;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::mem;
use std::os::raw::*;
use std::ptr;
use std::str::CharIndices;
use x11::xft;
use x11::xlib;
use x11::xrender;

use color::Color;
use font::{FontDescriptor, FontFamily, FontStretch, FontStyle};
use fontconfig as fc;
use geometrics::{Rectangle, Size};

const SERIF_FAMILY: &'static str = "Serif\0";
const SANS_SERIF_FAMILY: &'static str = "Sans\0";
const MONOSPACE_FAMILY: &'static str = "Monospace\0";

pub struct TextRenderer {
    display: *mut xlib::Display,
    font_caches: HashMap<FontKey, *mut xft::XftFont>,
}

impl TextRenderer {
    pub fn new(display: *mut xlib::Display) -> Self {
        Self {
            display,
            font_caches: HashMap::new(),
        }
    }

    pub fn render_single_line(
        &mut self,
        display: *mut xlib::Display,
        draw: *mut xft::XftDraw,
        text: &Text,
        bounds: Rectangle,
    ) {
        let origin_x = match text.horizontal_align {
            HorizontalAlign::Left => bounds.x,
            HorizontalAlign::Right => bounds.x + bounds.width,
            HorizontalAlign::Center => (bounds.x + bounds.width / 2.0) - (self.measure_single_line(display, text).width / 2.0),
        };
        let origin_y = match text.vertical_align {
            VerticalAlign::Top => bounds.y,
            VerticalAlign::Middle => bounds.y + bounds.height / 2.0 - text.font_size / 2.0,
            VerticalAlign::Bottom => bounds.y + bounds.height - text.font_size,
        };

        let mut x_offset = 0.0;

        for chunk in ChunkIter::new(text.content, &text.font_set) {
            let font = if let Some(font) =
                self.open_font(display, chunk.font, text.font_size, text.font_set.pattern)
            {
                font
            } else {
                continue;
            };

            let extents = unsafe {
                let mut extents = mem::MaybeUninit::<xrender::XGlyphInfo>::uninit();
                xft::XftTextExtentsUtf8(
                    display,
                    font,
                    chunk.text.as_ptr(),
                    chunk.text.len() as i32,
                    extents.as_mut_ptr(),
                );
                extents.assume_init()
            };

            let ascent = unsafe { (*font).ascent } as f32;
            let y_adjustment = if text.font_size > ascent {
                (text.font_size - ascent) / 2.0
            } else {
                0.0
            };

            unsafe {
                xft::XftDrawStringUtf8(
                    draw,
                    &mut text.color.as_xft_color(),
                    font,
                    (origin_x + x_offset + extents.x as f32).round() as i32,
                    (origin_y + y_adjustment + extents.height as f32).round() as i32,
                    chunk.text.as_ptr(),
                    chunk.text.len() as i32,
                );
            }

            x_offset += extents.width as f32;
        }
    }

    pub fn measure_single_line(&mut self, display: *mut xlib::Display, text: &Text) -> Size {
        let mut measured_size = Size {
            width: 0.0,
            height: 0.0,
        };

        for chunk in ChunkIter::new(text.content, &text.font_set) {
            let font = if let Some(font) =
                self.open_font(display, chunk.font, text.font_size, text.font_set.pattern)
            {
                font
            } else {
                continue;
            };

            let extents = unsafe {
                let mut extents = mem::MaybeUninit::<xrender::XGlyphInfo>::uninit();
                xft::XftTextExtentsUtf8(
                    display,
                    font,
                    chunk.text.as_ptr(),
                    chunk.text.len() as i32,
                    extents.as_mut_ptr(),
                );
                extents.assume_init()
            };

            measured_size.width += extents.width as f32;
            measured_size.height += measured_size.height.max(extents.height as f32);
        }

        measured_size
    }

    fn open_font(
        &mut self,
        display: *mut xlib::Display,
        font: *mut fc::FcPattern,
        font_size: f32,
        fontset_pattern: *mut fc::FcPattern,
    ) -> Option<*mut xft::XftFont> {
        unsafe {
            let pattern = fc::FcFontRenderPrepare(ptr::null_mut(), fontset_pattern, font);

            fc::FcPatternDel(pattern, fc::FC_PIXEL_SIZE.as_ptr() as *mut c_char);
            fc::FcPatternAddDouble(
                pattern,
                fc::FC_PIXEL_SIZE.as_ptr() as *mut c_char,
                font_size as f64,
            );

            match self.font_caches.entry(FontKey { pattern }) {
                hash_map::Entry::Occupied(entry) => {
                    fc::FcPatternDestroy(pattern);
                    Some(*entry.get())
                }
                hash_map::Entry::Vacant(entry) => {
                    let font = xft::XftFontOpenPattern(display, pattern.cast());
                    if font.is_null() {
                        fc::FcPatternDestroy(pattern);
                        return None;
                    }
                    entry.insert(font);
                    Some(font)
                }
            }
        }
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        for (key, font) in self.font_caches.drain() {
            unsafe {
                xft::XftFontClose(self.display, font);
                fc::FcPatternDestroy(key.pattern);
            }
        }
    }
}

#[derive(Debug)]
pub struct FontSet {
    pattern: *mut fc::FcPattern,
    fontset: *mut fc::FcFontSet,
    charsets: Vec<*mut fc::FcCharSet>,
    coverage: *mut fc::FcCharSet,
}

impl FontSet {
    pub fn new(font_descriptor: FontDescriptor) -> Option<FontSet> {
        unsafe {
            let pattern = create_pattern(&font_descriptor);

            fc::FcConfigSubstitute(ptr::null_mut(), pattern, fc::FcMatchKind::Pattern);
            fc::FcDefaultSubstitute(pattern);

            let mut result: fc::FcResult = fc::FcResult::NoMatch;
            let fontset = fc::FcFontSort(ptr::null_mut(), pattern, 1, ptr::null_mut(), &mut result);

            if result != fc::FcResult::Match || (*fontset).nfont == 0 {
                return None;
            }

            let mut coverage = fc::FcCharSetNew();
            let mut charsets = Vec::with_capacity((*fontset).nfont as usize);

            for i in 0..(*fontset).nfont {
                let font = *(*fontset).fonts.offset(i as isize);

                let mut charset: *mut fc::FcCharSet = ptr::null_mut();
                let result = fc::FcPatternGetCharSet(
                    font,
                    fc::FC_CHARSET.as_ptr() as *mut c_char,
                    0,
                    &mut charset,
                );

                if result == fc::FcResult::Match {
                    coverage = fc::FcCharSetUnion(coverage, charset);
                }

                charsets.push(charset);
            }

            Some(Self {
                pattern,
                fontset,
                charsets,
                coverage,
            })
        }
    }

    fn default_font(&self) -> *mut fc::FcPattern {
        unsafe { *(*self.fontset).fonts.offset(0) }
    }

    fn match_font(&self, c: char) -> Option<*mut fc::FcPattern> {
        unsafe {
            if fc::FcCharSetHasChar(self.coverage, c as u32) == 0 {
                return None;
            }

            for i in 0..(*self.fontset).nfont {
                let font = *(*self.fontset).fonts.offset(i as isize);
                let charset = self.charsets[i as usize];

                if !charset.is_null() && fc::FcCharSetHasChar(charset, c as u32) != 0 {
                    return Some(font);
                }
            }
        }

        None
    }
}

impl Drop for FontSet {
    fn drop(&mut self) {
        unsafe {
            for charset in self.charsets.iter() {
                fc::FcCharSetDestroy(*charset);
            }
            fc::FcCharSetDestroy(self.coverage);
            fc::FcFontSetDestroy(self.fontset);
            fc::FcPatternDestroy(self.pattern);
        }
    }
}

#[derive(Debug)]
pub struct FontKey {
    pattern: *mut fc::FcPattern,
}

impl PartialEq for FontKey {
    fn eq(&self, other: &Self) -> bool {
        unsafe { fc::FcPatternEqual(self.pattern, other.pattern) != 0 }
    }
}

impl Eq for FontKey {}

impl Hash for FontKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = unsafe { fc::FcPatternHash(self.pattern) };
        state.write_u32(hash);
    }
}

#[derive(Debug)]
pub struct Text<'a> {
    pub content: &'a str,
    pub color: &'a Color,
    pub font_size: f32,
    pub font_set: &'a FontSet,
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
}

#[derive(Debug)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

struct Chunk<'a> {
    text: &'a str,
    font: *mut fc::FcPattern,
}

struct ChunkIter<'a> {
    fontset: &'a FontSet,
    current_font: Option<*mut fc::FcPattern>,
    current_index: usize,
    inner_iter: CharIndices<'a>,
    source: &'a str,
}

impl<'a> ChunkIter<'a> {
    fn new(source: &'a str, fontset: &'a FontSet) -> Self {
        Self {
            fontset,
            current_font: None,
            current_index: 0,
            inner_iter: source.char_indices(),
            source,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.inner_iter.next() {
            let matched_font = self.fontset.match_font(c);
            if i == 0 {
                self.current_font = matched_font;
            } else if self.current_font != matched_font {
                let result = Some(Chunk {
                    text: &self.source[self.current_index..i],
                    font: self.current_font.unwrap_or(self.fontset.default_font()),
                });
                self.current_font = matched_font;
                self.current_index = i;
                return result;
            }
        }

        if self.current_index < self.source.len() {
            let result = Some(Chunk {
                text: &self.source[self.current_index..],
                font: self.current_font.unwrap_or(self.fontset.default_font()),
            });
            self.current_font = None;
            self.current_index = self.source.len();
            return result;
        }

        None
    }
}

unsafe fn create_pattern(descriptor: &FontDescriptor) -> *mut fc::FcPattern {
    let pattern = fc::FcPatternCreate();

    match &descriptor.family {
        FontFamily::Name(name) => {
            if let Ok(name_str) = CString::new(name.as_str()) {
                fc::FcPatternAddString(
                    pattern,
                    fc::FC_FAMILY.as_ptr() as *mut c_char,
                    name_str.as_ptr() as *mut c_uchar,
                );
            }
        }
        FontFamily::Serif => {
            fc::FcPatternAddString(
                pattern,
                fc::FC_FAMILY.as_ptr() as *mut c_char,
                SERIF_FAMILY.as_ptr(),
            );
        }
        FontFamily::SansSerif => {
            fc::FcPatternAddString(
                pattern,
                fc::FC_FAMILY.as_ptr() as *mut c_char,
                SANS_SERIF_FAMILY.as_ptr(),
            );
        }
        FontFamily::Monospace => {
            fc::FcPatternAddString(
                pattern,
                fc::FC_FAMILY.as_ptr() as *mut c_char,
                MONOSPACE_FAMILY.as_ptr(),
            );
        }
    };

    fc::FcPatternAddDouble(
        pattern,
        fc::FC_WEIGHT.as_ptr() as *mut c_char,
        descriptor.weight.0 as f64,
    );

    let slant = match descriptor.style {
        FontStyle::Italic => fc::FC_SLANT_ITALIC,
        FontStyle::Normal => fc::FC_SLANT_ROMAN,
        FontStyle::Oblique => fc::FC_SLANT_OBLIQUE,
    };
    fc::FcPatternAddInteger(pattern, fc::FC_SLANT.as_ptr() as *mut c_char, slant);

    let width = match descriptor.stretch {
        FontStretch::UltraCondensed => fc::FC_WIDTH_ULTRACONDENSED,
        FontStretch::ExtraCondensed => fc::FC_WIDTH_EXTRACONDENSED,
        FontStretch::Condensed => fc::FC_WIDTH_CONDENSED,
        FontStretch::SemiCondensed => fc::FC_WIDTH_SEMICONDENSED,
        FontStretch::Normal => fc::FC_WIDTH_NORMAL,
        FontStretch::SemiExpanded => fc::FC_WIDTH_SEMIEXPANDED,
        FontStretch::Expanded => fc::FC_WIDTH_EXPANDED,
        FontStretch::ExtraExpanded => fc::FC_WIDTH_EXTRAEXPANDED,
        FontStretch::UltraExpanded => fc::FC_WIDTH_ULTRAEXPANDED,
    };
    fc::FcPatternAddInteger(pattern, fc::FC_WIDTH.as_ptr() as *mut c_char, width);

    pattern
}
