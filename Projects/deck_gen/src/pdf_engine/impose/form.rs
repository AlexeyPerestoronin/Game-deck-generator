//! Turn a source PDF page into a Form XObject in the destination document.

use lopdf::{dictionary, Document, Object, ObjectId, Stream};

use super::import::Importer;
use crate::error::{Error, Result};

pub fn page_to_form(
    dest: &mut Document,
    importer: &mut Importer<'_>,
    page_id: ObjectId,
) -> Result<ObjectId> {
    let page = importer
        .src
        .get_dictionary(page_id)
        .map_err(|err| Error::msg(err.to_string()))?
        .clone();
    let bbox = page
        .get(b"CropBox")
        .or_else(|_| page.get(b"MediaBox"))
        .map_err(|err| Error::msg(err.to_string()))?
        .clone();
    let bbox = importer.rewrite(dest, bbox)?;
    let mut form_dict = dictionary! {
        "Type" => "XObject",
        "Subtype" => "Form",
        "FormType" => 1,
        "BBox" => bbox,
    };
    if let Ok(resources) = page.get(b"Resources") {
        form_dict.set("Resources", importer.rewrite(dest, resources.clone())?);
    }
    let contents = page
        .get(b"Contents")
        .map_err(|err| Error::msg(err.to_string()))?;
    let bytes = content_bytes(importer.src, contents)?;
    Ok(dest.add_object(Object::Stream(Stream::new(form_dict, bytes))))
}

fn content_bytes(doc: &Document, obj: &Object) -> Result<Vec<u8>> {
    match obj {
        Object::Reference(id) => content_bytes(doc, doc.get_object(*id)?),
        Object::Stream(stream) => Ok(stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone())),
        Object::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                out.extend(content_bytes(doc, item)?);
                out.push(b'\n');
            }
            Ok(out)
        }
        other => Err(Error::msg(format!("unexpected page Contents: {other:?}"))),
    }
}
