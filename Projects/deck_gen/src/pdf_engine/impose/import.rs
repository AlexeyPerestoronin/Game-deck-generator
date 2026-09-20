//! Copy PDF objects from a source document into the destination, remapping ids.

use std::collections::BTreeMap;

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::error::{Error, Result};

pub struct Importer<'a> {
    pub src: &'a Document,
    map: BTreeMap<ObjectId, ObjectId>,
}

impl<'a> Importer<'a> {
    pub fn new(src: &'a Document) -> Self {
        Self {
            src,
            map: BTreeMap::new(),
        }
    }

    pub fn page_size(&self, page_id: ObjectId) -> Result<(f64, f64)> {
        let page = self.src.get_dictionary(page_id)?;
        let box_obj = page.get(b"CropBox").or_else(|_| page.get(b"MediaBox"))?;
        let box_obj = resolve(self.src, box_obj)?;
        let arr = box_obj.as_array().map_err(|err| Error::msg(err.to_string()))?;
        if arr.len() < 4 {
            return Err(Error::msg("MediaBox must have 4 numbers"));
        }
        let nums: Vec<f64> = arr.iter().take(4).map(object_f64).collect::<Result<Vec<_>>>()?;
        Ok((nums[2] - nums[0], nums[3] - nums[1]))
    }

    pub fn rewrite(&mut self, dest: &mut Document, obj: Object) -> Result<Object> {
        match obj {
            Object::Reference(id) => Ok(Object::Reference(self.import(dest, id)?)),
            Object::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.rewrite(dest, item)?);
                }
                Ok(Object::Array(out))
            }
            Object::Dictionary(dict) => Ok(Object::Dictionary(self.rewrite_dict(dest, dict)?)),
            Object::Stream(mut stream) => {
                stream.dict = self.rewrite_dict(dest, stream.dict)?;
                Ok(Object::Stream(stream))
            }
            other => Ok(other),
        }
    }

    fn import(&mut self, dest: &mut Document, id: ObjectId) -> Result<ObjectId> {
        if let Some(&existing) = self.map.get(&id) {
            return Ok(existing);
        }
        let new_id = dest.new_object_id();
        self.map.insert(id, new_id);
        let obj = self.src.get_object(id)?.clone();
        let rewritten = self.rewrite(dest, obj)?;
        dest.objects.insert(new_id, rewritten);
        Ok(new_id)
    }

    fn rewrite_dict(&mut self, dest: &mut Document, dict: Dictionary) -> Result<Dictionary> {
        let mut out = Dictionary::new();
        for (key, value) in dict.into_iter() {
            out.set(key, self.rewrite(dest, value)?);
        }
        Ok(out)
    }
}

fn resolve<'a>(doc: &'a Document, obj: &'a Object) -> Result<&'a Object> {
    match obj {
        Object::Reference(id) => Ok(doc.get_object(*id)?),
        other => Ok(other),
    }
}

fn object_f64(obj: &Object) -> Result<f64> {
    match obj {
        Object::Integer(v) => Ok(*v as f64),
        Object::Real(v) => Ok(*v as f64),
        Object::Reference(_) => Err(Error::msg("unresolved number in MediaBox")),
        other => Err(Error::msg(format!("expected number, got {other:?}"))),
    }
}
