use freetype::face::{Face, LoadFlag};
use freetype::Library;
use glow::{HasContext, NativeTexture};

use thiserror::Error;

use std::collections::hash_map::{Entry, HashMap};

use super::GlError;

#[allow(unused)]
pub struct CachedCharacter {
    pub texture: NativeTexture,
    pub advance_x: i32,
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Error, Debug)]
enum GlyphCacheCreationErrorRepr {
    #[error("failed to create font library")]
    CreateLibrary(freetype::Error),
    #[error("failed to create font face")]
    CreateFace(freetype::Error),
    #[error("failed to set font size")]
    SetSize(freetype::Error),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct GlyphCacheCreationError(#[from] GlyphCacheCreationErrorRepr);

pub struct GlyphCache {
    character_map: HashMap<char, CachedCharacter>,
    pixel_size: u32,
    face: Face<&'static [u8]>,
}

#[derive(Error, Debug)]
enum GetCharacterErrorRepr {
    #[error("failed to load character")]
    LoadChar(freetype::Error),
    #[error("failed to create texture")]
    CreateTexture(GlError),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct GetCharacterError(#[from] GetCharacterErrorRepr);

impl GlyphCache {
    pub fn new(pixel_size: u32) -> Result<GlyphCache, GlyphCacheCreationError> {
        let lib = Library::init().map_err(GlyphCacheCreationErrorRepr::CreateLibrary)?;

        const HACK_TTF: &[u8] = include_bytes!("../res/Hack-Regular.ttf");

        let face = lib
            .new_memory_face2(HACK_TTF, 0)
            .map_err(GlyphCacheCreationErrorRepr::CreateFace)?;

        face.set_pixel_sizes(pixel_size, pixel_size)
            .map_err(GlyphCacheCreationErrorRepr::SetSize)?;

        Ok(GlyphCache {
            character_map: HashMap::new(),
            pixel_size,
            face,
        })
    }

    pub fn pixel_size(&self) -> u32 {
        self.pixel_size
    }

    pub fn get_character(
        &mut self,
        gl: &glow::Context,
        c: char,
    ) -> Result<&CachedCharacter, GetCharacterError> {
        let entry = self.character_map.entry(c);
        let entry = match entry {
            Entry::Occupied(v) => {
                return Ok(v.into_mut());
            }
            Entry::Vacant(v) => v,
        };

        self.face
            .load_char(c as usize, LoadFlag::RENDER)
            .map_err(GetCharacterErrorRepr::LoadChar)?;
        let glyph = self.face.glyph();
        if let Err(e) = glyph.render_glyph(freetype::RenderMode::Sdf) {
            println!("Failed to render glyph with sdf for {}: {e}", c);
        }
        let glyph_bitmap = glyph.bitmap();

        let texture = unsafe {
            let texture = crate::gl_util::create_tex_default_params(gl)
                .map_err(GetCharacterErrorRepr::CreateTexture)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                glyph_bitmap.pitch(),
                glyph_bitmap.rows(),
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(glyph_bitmap.buffer()),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            texture
        };

        let inserted = entry.insert(CachedCharacter {
            texture,
            advance_x: glyph.advance().x as i32,
            left: glyph.bitmap_left(),
            top: glyph.bitmap_top(),
            width: glyph_bitmap.width(),
            height: glyph_bitmap.rows(),
        });
        Ok(inserted)
    }
}
