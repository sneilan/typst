// WASM test to measure typst dependency size
// This actually exercises the compile pipeline to ensure all code is included

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

static LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();

/// A minimal world for compiling typst content
struct MiniWorld {
    library: &'static LazyHash<Library>,
    main: Source,
    fonts: Vec<Font>,
    book: LazyHash<FontBook>,
}

impl MiniWorld {
    fn new(source: &str) -> Self {
        let library = LIBRARY.get_or_init(|| LazyHash::new(Library::default()));
        let main = Source::detached(source);
        Self {
            library,
            main,
            fonts: vec![],
            book: LazyHash::new(FontBook::new()),
        }
    }
}

impl World for MiniWorld {
    fn library(&self) -> &LazyHash<Library> {
        self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
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

#[no_mangle]
pub extern "C" fn compile_math(ptr: *const u8, len: usize) -> i32 {
    let source = unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Wrap in math mode
    let typst_source = format!("$ {} $", source);
    let world = MiniWorld::new(&typst_source);

    match compile::<PagedDocument>(&world).output {
        Ok(doc) => doc.pages.len() as i32,
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn init_typst() -> u32 {
    // Initialize the library
    let _ = LIBRARY.get_or_init(|| LazyHash::new(Library::default()));
    1
}
