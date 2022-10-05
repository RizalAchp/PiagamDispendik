#![allow(dead_code)]

use crate::lang;
use nwd::NwgPartial;

#[derive(Default, NwgPartial)]
pub struct ValuesUi {
    #[nwg_layout(min_size: [20,20], max_column: Some(10))]
    layout: nwg::GridLayout,

    #[nwg_control(text: &lang::tr("ui-values-label"))]
    #[nwg_layout_item(layout: layout, col: 8, row: 0)]
    label: nwg::Label,

    #[nwg_control(list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT, 
    )]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 8, row: 0)]
    data_view: nwg::ListView,
}

impl ValuesUi {
    pub fn insert_header(&self, headers: &Option<Vec<String>>) {
        match headers {
            Some(h) => {h.into_iter().for_each(|header| self.data_view.insert_column(header.clone())); ()},
            None => (),
        }
    }
    
    pub fn load_data(&self, _value: &Vec<Vec<String>>) {
        let dv = &self.data_view;

        dv.insert_column("Name");
        dv.insert_column(nwg::InsertListViewColumn{
            index: Some(1),
            fmt: Some(nwg::ListViewColumnFlags::RIGHT),
            width: Some(20),
            text: Some("test".into())
        });
        dv.set_headers_enabled(true);

        // Passing a str to this method will automatically push the item at the end of the list in the first column
        dv.insert_item("Cat");
        dv.insert_item(nwg::InsertListViewItem { 
            index: Some(0),
            column_index: 1,
            text: Some("Felis".into()),
            image: None
        });

        // To insert a new row, use the index 0.
        dv.insert_item(nwg::InsertListViewItem {
            index: Some(0),
            column_index: 0,
            text: Some("Moose".into()),
            image: Some(1),
        });

        dv.insert_item(nwg::InsertListViewItem {
            index: Some(0),
            column_index: 1,
            text: Some("Alces".into()),
            image: None,
        });

        // Insert multiple item on a single row. 
        dv.insert_items_row(None, &["Dog", "Canis"]);

        // Insert many item at one
        dv.insert_items(&["Duck", "Horse", "Boomalope"]);
        dv.insert_items(&[
            nwg::InsertListViewItem { index: Some(3), column_index: 1, text: Some("Anas".into()), image: None },
            nwg::InsertListViewItem { index: Some(4), column_index: 1, text: Some("Equus".into()), image: None },
        ]);

        // Update items
        dv.update_item(2, nwg::InsertListViewItem { image: Some(1), ..Default::default() });
        dv.update_item(4, nwg::InsertListViewItem { image: Some(1), ..Default::default() });
    }
    
    pub fn reset_language(&self) {
        self.label.set_text(&lang::tr("ui-values-label"))
    }
}
    
