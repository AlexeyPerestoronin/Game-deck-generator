//! A4 n-up geometry. Origin for millimetre coordinates is the page top-left.

use crate::conf::PrintSettings;

#[derive(Clone, Copy, Debug)]
pub struct CardSize {
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duplex {
    LongEdge,
    ShortEdge,
}

impl Duplex {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "long-edge" => Ok(Self::LongEdge),
            "short-edge" => Ok(Self::ShortEdge),
            other => Err(format!(
                "duplex must be 'long-edge' or 'short-edge', got {other:?}"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SheetLayout {
    pub page_width_mm: f64,
    pub page_height_mm: f64,
    pub cols: i32,
    pub rows: i32,
    pub gap_x_mm: f64,
    pub gap_y_mm: f64,
    pub crop_mark_mm: f64,
    pub crop_mark_gap_mm: f64,
    pub crop_mark_thickness_pt: f64,
}

impl SheetLayout {
    pub fn from_print(print: &PrintSettings) -> Self {
        Self {
            page_width_mm: print.page_width_mm,
            page_height_mm: print.page_height_mm,
            cols: print.cols,
            rows: print.rows,
            gap_x_mm: print.gap_x_mm,
            gap_y_mm: print.gap_y_mm,
            crop_mark_mm: print.crop_mark_mm,
            crop_mark_gap_mm: print.crop_mark_gap_mm,
            crop_mark_thickness_pt: print.crop_mark_thickness_pt,
        }
    }

    pub fn cards_per_sheet(&self) -> i32 {
        self.cols * self.rows
    }

    pub fn margins_mm(&self, card: CardSize) -> (f64, f64) {
        let grid_w = f64::from(self.cols) * card.width_mm + f64::from(self.cols - 1) * self.gap_x_mm;
        let grid_h = f64::from(self.rows) * card.height_mm + f64::from(self.rows - 1) * self.gap_y_mm;
        (
            (self.page_width_mm - grid_w) / 2.0,
            (self.page_height_mm - grid_h) / 2.0,
        )
    }

    pub fn cell_origin_mm(&self, col: i32, row: i32, card: CardSize) -> (f64, f64) {
        let (margin_x, margin_y) = self.margins_mm(card);
        (
            margin_x + f64::from(col) * (card.width_mm + self.gap_x_mm),
            margin_y + f64::from(row) * (card.height_mm + self.gap_y_mm),
        )
    }

    pub fn slots(&self, sheet: i32, face_count: i32, duplex: Duplex) -> Vec<Slot> {
        let start = sheet * self.cards_per_sheet();
        let end = (start + self.cards_per_sheet()).min(face_count);
        (start..end)
            .map(|card_index| {
                let local = card_index - start;
                let row = local / self.cols;
                let col = local % self.cols;
                let (back_col, back_row) = match duplex {
                    Duplex::LongEdge => (self.cols - 1 - col, row),
                    Duplex::ShortEdge => (col, self.rows - 1 - row),
                };
                Slot {
                    card_index,
                    col,
                    row,
                    back_col,
                    back_row,
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub card_index: i32,
    pub col: i32,
    pub row: i32,
    pub back_col: i32,
    pub back_row: i32,
}

pub fn mm_to_pt(value: f64) -> f64 {
    value * 72.0 / 25.4
}

/// Convert a top-left y (mm) into PDF user space (origin bottom-left, points).
pub fn pdf_y_top_left_mm(page_height_mm: f64, top_mm: f64, height_mm: f64) -> f64 {
    mm_to_pt(page_height_mm - top_mm - height_mm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_edge_mirrors_columns() {
        let layout = SheetLayout {
            page_width_mm: 210.0,
            page_height_mm: 297.0,
            cols: 3,
            rows: 3,
            gap_x_mm: 5.0,
            gap_y_mm: 5.0,
            crop_mark_mm: 2.0,
            crop_mark_gap_mm: 0.8,
            crop_mark_thickness_pt: 0.3,
        };
        let slots = layout.slots(0, 2, Duplex::LongEdge);
        assert_eq!(slots[0].col, 0);
        assert_eq!(slots[0].back_col, 2);
        assert_eq!(slots[1].col, 1);
        assert_eq!(slots[1].back_col, 1);
    }
}
