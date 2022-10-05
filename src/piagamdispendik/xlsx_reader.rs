use serde::Deserialize;
use serde_xml_rs::from_reader;
use std::char;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use zip;

pub fn parse_xlsx(
    data: &Vec<u8>,
    date_columns: Option<Vec<usize>>,
) -> Result<HashMap<usize, HashMap<usize, String>>, String> {
    let (strings, sheet) = match parse_xlsx_file_to_parts(data) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };
    let map = match get_strings_map(strings) {
        Some(m) => m,
        None => return Err("Data extracting error".to_owned()),
    };
    get_parsed_xlsx(map, sheet, date_columns)
}

pub fn parse_xlsx_file_to_parts(data: &Vec<u8>) -> Result<(String, String), String> {
    let reader = Cursor::new(data);
    let mut zip = match zip::ZipArchive::new(reader) {
        Ok(z) => z,
        Err(err) => return Err(format!("{:?}", err)),
    };

    let mut strings_content = String::new();
    let mut sheet_content = String::new();

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if file.name() == "xl/sharedStrings.xml" {
            match file.read_to_string(&mut strings_content) {
                Ok(_) => (),
                Err(err) => return Err(format!("Can't read strings file: {:?}", err)),
            }
        } else {
            if file.name() == "xl/worksheets/sheet1.xml" {
                match file.read_to_string(&mut sheet_content) {
                    Ok(_) => (),
                    Err(err) => return Err(format!("Can't read sheet file: {:?}", err)),
                }
            }
        }
    }
    Ok((strings_content, sheet_content))
}

pub fn get_strings_map(strings: String) -> Option<HashMap<usize, String>> {
    #[derive(Debug, Deserialize)]
    struct T {
        #[serde(rename = "$value")]
        val: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct R {
        t: Option<T>,
    }

    #[derive(Debug, Deserialize)]
    struct Si {
        t: Option<Vec<T>>,
        r: Option<Vec<R>>,
    }

    #[derive(Debug, Deserialize)]
    struct Sst {
        si: Vec<Si>,
    }

    let sst: Sst = match from_reader(strings.as_bytes()) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let mut map: HashMap<usize, String> = HashMap::new();
    let mut i = 0;
    for si in sst.si.iter() {
        if let Some(ref sits) = si.t {
            if let Some(ref sit) = sits.get(0) {
                map.insert(i, sit.val.clone().unwrap_or("".to_owned()));
            }
        } else {
            if let Some(ref sirs) = si.r {
                if let Some(ref sir) = sirs.get(0) {
                    if let Some(ref sirt) = sir.t {
                        map.insert(i, sirt.val.clone().unwrap_or("".to_owned()));
                    }
                }
            }
        }
        i = i + 1;
    }
    Some(map)
}

#[derive(Deserialize)]
struct CellValue {
    #[serde(rename = "$value")]
    v: Option<String>,
}

#[derive(Deserialize)]
struct Cell {
    r: Option<String>,
    s: Option<String>,
    t: Option<String>,
    v: Option<CellValue>,
}

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "c", default)]
    pub cells: Option<Vec<Cell>>,
}

#[derive(Deserialize)]
struct SheetData {
    #[serde(rename = "row", default)]
    pub rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Worksheet {
    #[serde(rename = "sheetData", default)]
    pub sheet: Vec<SheetData>,
}

pub fn get_parsed_xlsx(
    strings_map: HashMap<usize, String>,
    sheet_content: String,
    date_columns: Option<Vec<usize>>,
) -> Result<HashMap<usize, HashMap<usize, String>>, String> {
    let worksheet: Worksheet = match from_reader(sheet_content.as_bytes()) {
        Ok(ws) => ws,
        Err(err) => return Err(format!("XML parsing error: {:?}", err)),
    };
    let known_date_columns: Vec<usize> = date_columns.unwrap_or(Vec::new());
    let sd = &worksheet.sheet[0];
    let mut table: HashMap<usize, HashMap<usize, String>> = HashMap::with_capacity(sd.rows.len());
    let mut ir: usize = 0;
    for row in sd.rows.iter() {
        if let Some(ref cells) = row.cells {
            let mut tr: HashMap<usize, String> = HashMap::with_capacity(cells.len());
            let mut i: usize = 0;
            for cell in cells.iter() {
                if let Some(ref cell_r) = cell.r {
                    let pre_i = i;
                    i = 0;
                    while excel_str_cell(ir + 1, i) != cell_r.as_str() {
                        i += 1;
                        if i > 16384 {
                            // https://support.office.com/en-us/article/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
                            i = pre_i;
                            break;
                        }
                    }
                }
                if let Some(ref cv) = cell.v {
                    if let Some(ref value) = cv.v {
                        let mut value_found = false;
                        if known_date_columns.contains(&i) {
                            if let Some(ref s) = cell.s {
                                if s == "10" || s == "14" || s == "15" {
                                    // when parsing dates in format "05/15/2015 7 PM" we need to add this offset
                                    if let Some(dt) = excel_date(value, Some(1462.0)) {
                                        tr.insert(i, dt);
                                        value_found = true;
                                    }
                                } else {
                                    if let Some(dt) = excel_date(value, None) {
                                        tr.insert(i, dt);
                                        value_found = true;
                                    }
                                }
                            } else {
                                if let Some(dt) = excel_date(value, None) {
                                    tr.insert(i, dt);
                                    value_found = true;
                                }
                            }
                        }
                        if !value_found {
                            let t = cell.t.clone().unwrap_or("".to_owned());
                            if t == "s" {
                                let val = match value.parse::<usize>() {
                                    Ok(map_index) => {
                                        if strings_map.contains_key(&map_index) {
                                            strings_map[&map_index].clone()
                                        } else {
                                            value.to_owned()
                                        }
                                    }
                                    Err(_) => value.to_owned(),
                                };
                                tr.insert(i, val);
                            } else {
                                tr.insert(i, value.to_owned());
                            }
                        }
                    }
                }
                i = i + 1;
            }
            table.insert(ir, tr);
            ir = ir + 1;
        }
    }
    Ok(table)
}

pub fn excel_date(src: &str, days_offset: Option<f64>) -> Option<String> {
    let mut days: f64 = match src.parse::<f64>() {
        Ok(i) => {
            if i != 0.0 {
                i + days_offset.unwrap_or(0.0)
            } else {
                return None;
            }
        }
        Err(_) => return None,
    };
    let d: isize;
    let m: isize;
    let y: isize;
    if days == 60.0 {
        d = 29;
        m = 2;
        y = 1900;
    } else {
        if days < 60.0 {
            // Because of the 29-02-1900 bug, any serial date
            // under 60 is one off... Compensate.
            days += 1.0;
        }
        // Modified Julian to DMY calculation with an addition of 2415019
        let mut l = (days as isize) + 68569 + 2415019;
        let n = (4 * l) / 146097;
        l = l - ((146097 * n + 3) / 4);
        let i = (4000 * (l + 1)) / 1461001;
        l = l - ((1461 * i) / 4) + 31;
        let j = (80 * l) / 2447;
        d = l - ((2447 * j) / 80);
        l = j / 11;
        m = j + 2 - (12 * l);
        y = 100 * (n - 49) + i + l;
    }
    let date = format!("{}-{:02}-{:02}", y, m, d);
    if date == "1900-01-01" || date == "1900-01-02" {
        None
    } else {
        Some(date)
    }
}

pub fn excel_str_cell(row: usize, cell: usize) -> String {
    if cell == 0 {
        return format!("A{}", row);
    }
    let mut dividend = cell + 1;
    let mut column_name = String::new();
    let mut modulo;

    while dividend > 0 {
        modulo = (dividend - 1) % 26;
        column_name = format!(
            "{}{}",
            char::from_u32((65 + modulo) as u32).unwrap_or('A'),
            column_name
        );
        dividend = (dividend - modulo) / 26;
    }

    format!("{}{}", column_name, row)
}
