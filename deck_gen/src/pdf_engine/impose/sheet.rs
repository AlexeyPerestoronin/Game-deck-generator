//! One A4 sheet: place card XObjects and draw crop marks.

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};

use super::form::page_to_form;
use super::import::Importer;
use crate::error::{Error, Result};
use crate::pdf_engine::layout::{mm_to_pt, pdf_y_top_left_mm, CardSize, SheetLayout, Slot};

pub fn build_sheet(
    dest: &mut Document,
    importer: &mut Importer<'_>,
    pages_id: ObjectId,
    src_pages: &[ObjectId],
    face_count: usize,
    slots: &[Slot],
    card: CardSize,
    layout: SheetLayout,
    page_w: f64,
    page_h: f64,
    is_back: bool,
) -> Result<ObjectId> {
    let mut xobjects = Dictionary::new();
    let mut operations = vec![Operation::new("RG", vec![0.into(), 0.into(), 0.into()])];

    for (index, slot) in slots.iter().enumerate() {
        let page_index = source_page_index(is_back, src_pages.len(), face_count, slot.card_index);
        if page_index >= src_pages.len() {
            return Err(Error::msg(format!("PDF page {page_index} is out of range")));
        }
        let form_id = page_to_form(dest, importer, src_pages[page_index])?;
        let name = format!("Fm{index}");
        xobjects.set(name.as_bytes().to_vec(), Object::Reference(form_id));

        let (col, row) = if is_back {
            (slot.back_col, slot.back_row)
        } else {
            (slot.col, slot.row)
        };
        place_card(
            &mut operations,
            importer,
            src_pages[page_index],
            &name,
            layout,
            card,
            col,
            row,
        )?;
        push_crop_marks(&mut operations, &layout, col, row, card);
    }

    let content_id = dest.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode()?,
    ));
    let page_id = dest.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), page_w.into(), page_h.into()],
        "Contents" => content_id,
        "Resources" => dictionary! { "XObject" => xobjects },
    });
    Ok(page_id)
}

fn place_card(
    operations: &mut Vec<Operation>,
    importer: &Importer<'_>,
    src_page: ObjectId,
    xobject_name: &str,
    layout: SheetLayout,
    card: CardSize,
    col: i32,
    row: i32,
) -> Result<()> {
    let (x_mm, y_mm) = layout.cell_origin_mm(col, row, card);
    let dest_w = mm_to_pt(card.width_mm);
    let dest_h = mm_to_pt(card.height_mm);
    let tx = mm_to_pt(x_mm);
    let ty = pdf_y_top_left_mm(layout.page_height_mm, y_mm, card.height_mm);
    let (src_w, src_h) = importer.page_size(src_page)?;
    let sx = dest_w / src_w;
    let sy = dest_h / src_h;
    operations.push(Operation::new("q", vec![]));
    operations.push(Operation::new(
        "cm",
        vec![sx.into(), 0.into(), 0.into(), sy.into(), tx.into(), ty.into()],
    ));
    operations.push(Operation::new(
        "Do",
        vec![Object::Name(xobject_name.as_bytes().to_vec())],
    ));
    operations.push(Operation::new("Q", vec![]));
    Ok(())
}

fn source_page_index(is_back: bool, src_page_count: usize, face_count: usize, card_index: i32) -> usize {
    if !is_back {
        return card_index as usize;
    }
    if src_page_count == face_count {
        card_index as usize
    } else {
        0
    }
}

fn push_crop_marks(
    operations: &mut Vec<Operation>,
    layout: &SheetLayout,
    col: i32,
    row: i32,
    card: CardSize,
) {
    let (x_mm, y_mm) = layout.cell_origin_mm(col, row, card);
    let x0 = mm_to_pt(x_mm);
    let y0_top = mm_to_pt(y_mm);
    let x1 = mm_to_pt(x_mm + card.width_mm);
    let y1_top = mm_to_pt(y_mm + card.height_mm);
    let page_h = mm_to_pt(layout.page_height_mm);
    let y0 = page_h - y0_top;
    let y1 = page_h - y1_top;
    let top = y0.max(y1);
    let bottom = y0.min(y1);
    let tick = mm_to_pt(layout.crop_mark_mm);
    let gap = mm_to_pt(layout.crop_mark_gap_mm);
    operations.push(Operation::new(
        "w",
        vec![layout.crop_mark_thickness_pt.into()],
    ));
    let corners = [
        (x0, top, -1.0, 1.0),
        (x1, top, 1.0, 1.0),
        (x0, bottom, -1.0, -1.0),
        (x1, bottom, 1.0, -1.0),
    ];
    for (x, y, dx, dy) in corners {
        stroke_line(operations, x + dx * gap, y, x + dx * (gap + tick), y);
        stroke_line(operations, x, y + dy * gap, x, y + dy * (gap + tick));
    }
}

fn stroke_line(operations: &mut Vec<Operation>, x1: f64, y1: f64, x2: f64, y2: f64) {
    operations.push(Operation::new("m", vec![x1.into(), y1.into()]));
    operations.push(Operation::new("l", vec![x2.into(), y2.into()]));
    operations.push(Operation::new("S", vec![]));
}
