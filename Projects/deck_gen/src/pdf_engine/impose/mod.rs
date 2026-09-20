//! Stamp card PDFs onto A4 sheets with crop marks and duplex mirroring.

use std::io::Cursor;

use lopdf::{dictionary, Document, Object};

use crate::error::{Error, Result};
use crate::pdf_engine::layout::{mm_to_pt, CardSize, Duplex, SheetLayout};

mod form;
mod import;
mod sheet;

use import::Importer;
use sheet::build_sheet;

pub fn impose_duplex_bytes(
    faces_pdf: &[u8],
    back_pdf: &[u8],
    card: CardSize,
    duplex: Duplex,
    layout: SheetLayout,
) -> Result<Vec<u8>> {
    let faces = Document::load_from(Cursor::new(faces_pdf))?;
    let back = Document::load_from(Cursor::new(back_pdf))?;
    let face_pages = page_ids(&faces);
    let back_pages = page_ids(&back);
    if face_pages.is_empty() {
        return Err(Error::msg("empty face PDF"));
    }
    if back_pages.is_empty() {
        return Err(Error::msg("empty back PDF"));
    }

    let face_count = face_pages.len() as i32;
    let sheet_count = (face_count + layout.cards_per_sheet() - 1) / layout.cards_per_sheet();
    let page_w = mm_to_pt(layout.page_width_mm);
    let page_h = mm_to_pt(layout.page_height_mm);

    let mut dest_doc = Document::with_version("1.5");
    let pages_id = dest_doc.new_object_id();
    let mut kids: Vec<Object> = Vec::new();
    let mut face_import = Importer::new(&faces);
    let mut back_import = Importer::new(&back);

    for sheet_index in 0..sheet_count {
        let slots = layout.slots(sheet_index, face_count, duplex);
        kids.push(Object::Reference(build_sheet(
            &mut dest_doc,
            &mut face_import,
            pages_id,
            &face_pages,
            face_count as usize,
            &slots,
            card,
            layout,
            page_w,
            page_h,
            false,
        )?));
        kids.push(Object::Reference(build_sheet(
            &mut dest_doc,
            &mut back_import,
            pages_id,
            &back_pages,
            face_count as usize,
            &slots,
            card,
            layout,
            page_w,
            page_h,
            true,
        )?));
    }

    dest_doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Count" => kids.len() as i32,
            "Kids" => kids,
        }),
    );
    let catalog_id = dest_doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    dest_doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    dest_doc.save_to(&mut out)?;
    Ok(out)
}

fn page_ids(doc: &Document) -> Vec<lopdf::ObjectId> {
    doc.get_pages().into_iter().map(|(_, id)| id).collect()
}
