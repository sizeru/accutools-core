use printpdf::{PdfDocument, PdfDocumentReference, Mm, PdfLayerReference, Point, Line, Pt, SvgTransform, Svg};
use std::{fs, sync::Arc};
use anyhow::{Error, Result, anyhow};
use number_to_words::number_to_words;

const MAX_DESC_LENGTH: usize = 23;
macro_rules! lpad {
    ($arg:expr) => {{
        format!("${:>11}", $arg)
    }}
}

#[derive(Debug)]
pub enum DocType {
    Invoice,
    Receipt,
    Quote,
}

#[derive(Debug)]
pub struct ReceiptInfo {
    pub title: String,
    pub date: String,
    pub company_name: String,
    pub company_info_line: String,
    pub customer_info: String,
    pub transaction_number: String,
    pub order_id: String,
    pub vat_number: String,
    pub doc_number: String,
    pub doc_type: DocType,
    pub item_lines: Vec<ItemLine>,
    pub delivery_tickets: String,
    pub weigh_tickets: String,
    pub totals: Vec<Amount>,
    pub payments: Vec<Amount>,
    pub amount_due: String,
    pub employee: String,
    pub slogan: String,
}

#[derive(Debug)]
pub struct ItemLine {
    pub code: String,
    pub description: String,
    pub quantity: String,
    pub price: String,
    pub amount: String,
}

#[derive(Debug)]
pub struct Amount {
    pub name: String,
    pub value: String,
}

trait QuickShapes {
    // fn add_closed_shape<T>(&mut self, points: Vec<T>);
    fn add_box(&self, x1: Mm, y1: Mm, x2: Mm, y2: Mm);
    fn add_line(&self, x1: Mm, y1: Mm, x2: Mm, y2: Mm);
}

impl QuickShapes for PdfLayerReference {
    fn add_box(&self, x1: Mm, y1: Mm, x2: Mm, y2: Mm) {
        self.add_shape(Line {
            points: vec![
                (Point::new(x1, y1), false),
                (Point::new(x2, y1), false),
                (Point::new(x2, y2), false),
                (Point::new(x1, y2), false),
            ],
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        });
    }

    fn add_line(&self, x1: Mm, y1: Mm, x2: Mm, y2: Mm) {
        self.add_shape(Line {
            points: vec![
                (Point::new(x1, y1), false),
                (Point::new(x2, y2), false),
            ],
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        });
    }
}

pub struct PdfResources {
    font_regular: Arc<[u8]>,
    font_bold: Arc<[u8]>,
    font_mono: Arc<[u8]>,
    logo: Svg,
}

impl ReceiptInfo {
    pub fn pre_pass(&mut self) -> Result<(), Error> {
        let receipt_payment_pos = self.payments
            .iter()
            .position(|tender| tender.name.eq("Pay on Account"));
        if let Some(index) = receipt_payment_pos {
            let tender = self.payments.remove(index);
            let value_as_float = str::parse::<f64>(&tender.value)?.abs(); 
            let number_in_words = number_to_words(value_as_float, false);
            self.item_lines.push(
                ItemLine {
                    code: String::from("0000"),
                    description: format!("Received as cash deposit the sum of {number_in_words} dollars for materials."),
                    quantity: String::new(),
                    price: String::new(),
                    amount: tender.value.clone(),
                }
            );
            self.totals.clear();
            self.totals.push(
                Amount {
                    name: String::from("Total:"),
                    value: format!("{value_as_float:.2}"),
                }
            )
        }
        return Ok(());
    }
}
impl PdfResources {
    pub fn load(data_dir: &str) -> Result<Self, Error> {
        let font_regular_file = format!("{data_dir}/fonts/NotoSans-Regular.ttf");
        let font_regular = match fs::read(&font_regular_file) {
            Ok(bytes) => bytes,
            Err(e) => return Err(anyhow!(format!("Could not read the font from the file: `{}`. Reason: `{e}`", &font_regular_file)).into()),
        };
        let font_bold_file = format!("{data_dir}/fonts/NotoSans-Bold.ttf");
        let font_bold = match fs::read(&font_bold_file) {
            Ok(bytes) => bytes,
            Err(e) => return Err(anyhow!(format!("Could not read the font from the file: `{}`. Reason: `{e}`", &font_bold_file)).into()),
        };
        let font_mono_file = format!("{data_dir}/fonts/NotoSansMono-Regular.ttf");
        let font_mono = match fs::read(&font_mono_file) {
            Ok(bytes) => bytes,
            Err(e) => return Err(anyhow!(format!("Could not read the font from the file: `{}`. Reason: `{e}`", &font_mono_file)).into()),
        };
        let logo = {
            let svg_file = format!("{data_dir}/logo.svg");
            let svg = match fs::read_to_string(&svg_file) {
                Ok(file_as_string) => file_as_string,
                Err(e) => return Err(anyhow!(format!("Could not read the logo from the file: `{}`. Reason: `{e}`", &svg_file)).into()),
            };
            match Svg::parse(&svg) {
                Ok(svg) => svg,
                Err(e) => return Err(anyhow!(format!("Could not parse the svg loaded from: `{}`. Reason: {e}", &svg_file)).into()),
            }
        };
        // Converting from Vec to Arc doesn't reallocate the memory. Party!
        // This would be a safe thing to use raw pointers on, but I don't want
        // to implement that right now!
        return Ok(Self { 
            font_regular: Arc::from(font_regular),
            font_bold: Arc::from(font_bold),
            font_mono: Arc::from(font_mono),
            logo,
        });
    }
}

pub fn gen_pdf(receipt: &ReceiptInfo, resources: &PdfResources) -> Result<PdfDocumentReference, Error> {
    // Create and initialize document
    // 8.5" x 11" = 215.9mm x 279.4mm = 612pt x 792pt
    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", Pt(612.0).into(), Pt(792.0).into(), "Layer 1");
    let font_regular = doc.add_external_font(
        resources.font_regular.as_ref()
    )?;
    let font_bold = doc.add_external_font(
        resources.font_bold.as_ref()
    )?;
    let font_mono = doc.add_external_font(
        resources.font_mono.as_ref()
    )?;
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let left_margin: Mm = Pt(54.0).into();
    let right_margin: Mm = Pt(558.0).into();

    // Add title
    current_layer.use_text(&receipt.title, 14.0, Pt(260.0).into(), Pt(750.0).into(), &font_bold);

    // Add company header
    current_layer.use_text(&receipt.company_name, 28.0, Pt(225.0).into(), Pt(712.0).into(), &font_bold);
    current_layer.use_text(&receipt.company_info_line, 18.0, Pt(228.0).into(), Pt(690.0).into(), &font_regular);

    // Add logo
    let logo_transform = SvgTransform {
        translate_x: Some(Pt(55.0)),
        translate_y: Some(Pt(682.0)),
        rotate: None,
        scale_x: Some(0.65),
        scale_y: Some(0.65),
        dpi: None,
    };
    resources.logo.clone().add_to_layer(&current_layer, logo_transform);
    

    // Box for headers1
    // Pt 680 to 600 with 18pt font leaves space for four max lines
    let headers_bottom_border: Mm = Pt(640.0).into();
    // current_layer.add_box(left_margin, headers_bottom_border, right_margin, headers_bottom_border + Pt(headers_size).into());
    let spacing: Mm = Pt(5.0).into();
    let font_size = 8.0;
    let header_positions = [
        left_margin, 
        Pt(222.0).into(),
        Pt(390.0).into(),
    ];
    let doctype = match receipt.doc_type {
        DocType::Invoice => "Invoice Number:",
        DocType::Receipt => "Receipt Number:",
        DocType::Quote => "Quote Number:",
    };
    let text_bottom = headers_bottom_border + Pt(20.0).into();
    current_layer.use_text("Date/Time:"      , font_size, header_positions[0] + spacing, text_bottom, &font_bold);
    // current_layer.use_text("Order ID:"      , font_size, header_positions[1] + spacing, text_bottom, &font_bold);
    current_layer.use_text("VAT Number:", font_size, header_positions[1] + spacing, text_bottom, &font_bold);
    current_layer.use_text(doctype, font_size, header_positions[2] + spacing, text_bottom, &font_bold);
    let font_size = 12.0;
    let text_bottom = headers_bottom_border + Pt(4.0).into();
    current_layer.use_text(&receipt.date,      font_size, header_positions[0] + spacing, text_bottom, &font_regular);
    // current_layer.use_text(order_id,           font_size, header_positions[1] + spacing, headers_bottom_border, &font_regular);
    current_layer.use_text(&receipt.vat_number, font_size, header_positions[1] + spacing, text_bottom, &font_regular);
    current_layer.use_text(&receipt.doc_number,     font_size + 6.0, header_positions[2] + spacing, text_bottom - Pt(1.0).into(), &font_bold);

    
    // Box for headers2
    current_layer.add_box(left_margin, Pt(530.0).into(), right_margin, Pt(630.0).into());
    //Pt 264 to 524 Leaves space for 16 possible line items per page
    // Fill out customer info
    let mut current_y: Mm = Pt(618.0).into();
    current_layer.use_text("Sold to:", 8.0, left_margin + spacing, current_y, &font_bold);
    let line_height = Pt(16.0).into();
    receipt.customer_info.split("\n").for_each(
        |line| {
            current_y -= line_height;
            current_layer.use_text(line, font_size, left_margin + spacing, current_y, &font_regular);
        }
    );

    // Insert info
    current_y = Pt(618.0).into();
    let left_border: Mm = Into::<Mm>::into(Pt(390.0)) + spacing;
    current_layer.use_text("Clerk:", 8.0, left_border, current_y, &font_bold);
    current_layer.use_text(&receipt.employee, font_size, left_border, current_y - Pt(16.0).into(), &font_regular);
    current_layer.use_text("Delivery Ticket #:", 8.0, left_border, current_y - Pt(32.0).into(), &font_bold);
    current_layer.use_text(&receipt.delivery_tickets, font_size, left_border, current_y - Pt(48.0).into(), &font_regular);
    current_layer.use_text("Weigh Ticket #:", 8.0, left_border, current_y - Pt(64.0).into(), &font_bold);
    current_layer.use_text(&receipt.weigh_tickets, font_size, left_border, current_y - Pt(80.0).into(), &font_regular);

    let li_top: Mm = Pt(514.0).into();
    let li_bottom: Mm = Pt(254.0).into();
    current_layer.add_box(left_margin, li_bottom, right_margin, li_top);

    // vertical lines to divide line item on invoice
    let li_vlines: [Mm; 5] = [
        Pt(104.0).into(), // Name | Desc
        Pt(282.0).into(), // Desc | U/M
        Pt(322.0).into(), // U/M | Qty
        Pt(393.0).into(), // Qty | Price
        Pt(476.0).into(), // Price | Total
    ];
    for x in li_vlines {
        current_layer.add_line(x, li_bottom, x, li_top);
    }
    // Populate line items and subtotals
    {
        // Add headers
        let font_size = 12.0;
        let line_height = 20.0;
        let line_height_mm = Pt(line_height).into();
        let spacing: Mm = Pt(5.0).into();
        let mut bottom_border = li_top - line_height_mm;
        let mut cursor_y = bottom_border + spacing;
        current_layer.add_line(left_margin, bottom_border, right_margin, bottom_border);
        current_layer.use_text("Item"       , font_size, left_margin    + spacing, cursor_y, &font_regular);
        current_layer.use_text("Description", font_size, li_vlines[0] + spacing, cursor_y, &font_regular);
        current_layer.use_text("U/M"        , font_size, li_vlines[1] + spacing, cursor_y, &font_regular);
        current_layer.use_text("Quantity"   , font_size, li_vlines[2] + spacing, cursor_y, &font_regular);
        current_layer.use_text("Unit Price" , font_size, li_vlines[3] + spacing, cursor_y, &font_regular);
        current_layer.use_text("Total"      , font_size, li_vlines[4] + spacing, cursor_y, &font_regular);

        // Add content
        bottom_border -= line_height_mm;
        cursor_y = bottom_border + spacing;
        let font_size = 10.0;
        let line_height_mm: Mm = Pt(15.0).into();
        for line in &receipt.item_lines {
            let desc_lines = split_into_lines(&line.description, MAX_DESC_LENGTH);
            let item_num = str::parse::<usize>(&line.code)?;
            let uom = if item_num >= 2000 && item_num < 2100 {
                "EA" // item is a block
            } else {
                "TON" // item is not a block
            };
            let qty = if uom.eq("EA") && line.quantity.ends_with(".00") { 
                format!("{:>7}    ", &line.quantity[..line.quantity.len()-3])
            } else {
                format!("{:>10}", line.quantity)
            };
            let price = if line.price.is_empty() {
                String::new()
            } else {
                lpad!(&line.price)
            };
            let amount = if line.amount.is_empty() {
                String::new()
            } else {
                lpad!(&line.amount)
            };
            current_layer.use_text(&line.code,      font_size, left_margin  + spacing, cursor_y, &font_mono);
            current_layer.use_text(&desc_lines[0],  font_size, li_vlines[0] + spacing, cursor_y, &font_mono);
            current_layer.use_text(uom,             font_size, li_vlines[1] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&qty,            font_size, li_vlines[2] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&price,          font_size, li_vlines[3] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&amount,         font_size, li_vlines[4] + spacing, cursor_y, &font_mono);
            // Add additional description lines
            for i in 1..desc_lines.len() {
                bottom_border -= line_height_mm;
                cursor_y = bottom_border + spacing;
                current_layer.use_text(&desc_lines[i], font_size, li_vlines[0] + spacing, cursor_y, &font_mono);
            }
            bottom_border -= line_height_mm;
            cursor_y = bottom_border + spacing;
        }
    }

    // add totals below table on right side
    let mut current_y = li_bottom;
    let x1 = li_vlines[3] + spacing;
    let x2 = li_vlines[4] + spacing;
    for amount in &receipt.totals {
        current_y -= line_height;
        let font = if amount.name.eq("Total:") {
            &font_bold
        } else {
            &font_regular
        };
        let name = if amount.name.eq("Tax:") {
            "VAT:"
        } else {
            &amount.name
        };
        current_layer.use_text(name, font_size, x1, current_y, font);
        current_layer.use_text(&lpad!(amount.value), 10.0, x2, current_y, &font_mono);
    }

    // Add tenders below table on left side
    let mut current_y = li_bottom - Pt(40.0).into();
    let x1 = left_margin + spacing;
    let x2: Mm = Pt(200.0).into();
    current_y -= line_height;
    current_layer.use_text("Tender", font_size, x1, current_y, &font_regular);
    current_y -= Pt(4.0).into();
    current_layer.add_line(x1, current_y, x2 + Pt(80.0).into(), current_y);
    for amount in &receipt.payments {
        current_y -= line_height;
        current_layer.use_text(&amount.name, 10.0, x1, current_y, &font_regular);
        current_layer.use_text(&lpad!(amount.value), 10.0, x2, current_y, &font_mono);
    }

    //Pt 54 to 94 for signature box 
    current_layer.add_box(
        Pt(350.0).into(), Pt(84.0).into(), right_margin, Pt(84.0).into()
    );
    // Add signature line
    current_layer.use_text("Received By", 10.0, Pt(350.0).into(), Pt(74.0).into(), &font_regular);

    // Add slogan
    current_layer.use_text(&receipt.slogan, 9.0, Pt(258.0).into(), Pt(54.0).into(), &font_regular);
    return Ok(doc);
}

// Split any text which goes over a maximimum number of characters into separate
// lines
fn split_into_lines(string: &str, max_length: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if string.is_empty() {
        return Vec::new();
    }

    lines.push(string.to_owned());
    while unsafe { lines.last().unwrap_unchecked().len() } > max_length {
        let last_line = unsafe { lines.pop().unwrap_unchecked() };
        let final_whitespace = &last_line[..max_length+1]
            .chars()
            .enumerate()
            .filter(|(_, char)| char.eq(&' ') || char.eq(&'-'))
            .last();
        if let Some((index, _)) = final_whitespace {
            let (first_str, last_str)= last_line.split_at(*index+1);
            lines.push(first_str.to_owned());
            lines.push(format!(" {last_str}"));
        } else {
            let (first_str, last_str)= last_line.split_at(max_length+1);
            lines.push(format!("{first_str}-"));
            lines.push(format!(" {last_str}"));
        }
    }
    return lines;
}