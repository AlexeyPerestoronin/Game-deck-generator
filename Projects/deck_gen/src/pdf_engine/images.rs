//! Build a multi-page card-size PDF from JPEG page images (browser raster path).

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

use super::layout::{mm_to_pt, CardSize};
use crate::error::{Error, Result};

/// One raster page: JPEG bytes plus pixel size.
pub struct JpegPage {
    pub jpeg: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

/// Embed each JPEG as a page whose media box is the card size in points.
pub fn pdf_from_jpeg_pages(pages: &[JpegPage], card: CardSize) -> Result<Vec<u8>> {
    if pages.is_empty() {
        return Err(Error::msg("no JPEG pages to write"));
    }
    let page_w = mm_to_pt(card.width_mm);
    let page_h = mm_to_pt(card.height_mm);
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut kids: Vec<Object> = Vec::new();

    for page in pages {
        if page.width_px == 0 || page.height_px == 0 {
            return Err(Error::msg("JPEG page has zero size"));
        }
        let img_id = doc.add_object(Object::Stream(Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => page.width_px as i64,
                "Height" => page.height_px as i64,
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8,
                "Filter" => "DCTDecode",
            },
            page.jpeg.clone(),
        )));
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        page_w.into(),
                        0.into(),
                        0.into(),
                        page_h.into(),
                        0.into(),
                        0.into(),
                    ],
                ),
                Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]),
                Operation::new("Q", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode()?));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), page_w.into(), page_h.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "XObject" => dictionary! { "Im0" => img_id }
            },
        });
        kids.push(Object::Reference(page_id));
    }

    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Count" => kids.len() as i32,
            "Kids" => kids,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    doc.save_to(&mut out)?;
    Ok(out)
}
