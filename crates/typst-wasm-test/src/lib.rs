use wasm_bindgen::prelude::*;
use typst::compile;
use typst::Library;
use typst::LibraryExt;
use typst::layout::PagedDocument;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::World;
use typst::utils::LazyHash;
use std::sync::OnceLock;

// Embed fonts - New Computer Modern Math for math, New Computer Modern for text
const NEW_CM_MATH_REGULAR: &[u8] = include_bytes!("fonts/NewCMMath-Regular.otf");
const NEW_CM_REGULAR: &[u8] = include_bytes!("fonts/NewCM10-Regular.otf");
const NEW_CM_BOOK: &[u8] = include_bytes!("fonts/NewCM10-Book.otf");
const NEW_CM_BOLD: &[u8] = include_bytes!("fonts/NewCM10-Bold.otf");
const NEW_CM_ITALIC: &[u8] = include_bytes!("fonts/NewCM10-Italic.otf");

static LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();
static FONTS: OnceLock<(Vec<Font>, LazyHash<FontBook>)> = OnceLock::new();

fn load_fonts() -> (Vec<Font>, LazyHash<FontBook>) {
    let mut fonts = Vec::new();
    let mut book = FontBook::new();

    // Load embedded fonts
    for font_data in [NEW_CM_MATH_REGULAR, NEW_CM_REGULAR, NEW_CM_BOOK, NEW_CM_BOLD, NEW_CM_ITALIC] {
        let buffer = Bytes::new(font_data.to_vec());
        for font in Font::iter(buffer) {
            book.push(font.info().clone());
            fonts.push(font);
        }
    }

    (fonts, LazyHash::new(book))
}

struct MiniWorld {
    library: &'static LazyHash<Library>,
    main: Source,
    fonts: &'static Vec<Font>,
    book: &'static LazyHash<FontBook>,
}

impl MiniWorld {
    fn new(source: &str) -> Self {
        let library = LIBRARY.get_or_init(|| LazyHash::new(Library::default()));
        let (fonts, book) = FONTS.get_or_init(load_fonts);
        let main = Source::detached(source);
        Self {
            library,
            main,
            fonts,
            book,
        }
    }
}

impl World for MiniWorld {
    fn library(&self) -> &LazyHash<Library> {
        self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(typst::diag::FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(typst::diag::FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

#[wasm_bindgen]
pub fn init() -> String {
    let _ = LIBRARY.get_or_init(|| LazyHash::new(Library::default()));
    let (fonts, _) = FONTS.get_or_init(load_fonts);
    let names: Vec<_> = fonts.iter().map(|f| f.info().family.as_str()).collect();
    format!("Loaded {} fonts: {:?}", fonts.len(), names)
}

#[wasm_bindgen]
pub fn compile_to_svg(source: &str) -> Result<String, String> {
    let world = MiniWorld::new(source);

    match compile::<PagedDocument>(&world).output {
        Ok(doc) => {
            if let Some(page) = doc.pages.first() {
                Ok(typst_svg::svg(page))
            } else {
                Err("No pages generated".to_string())
            }
        }
        Err(errors) => {
            let msg = errors.iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            Err(msg)
        }
    }
}
