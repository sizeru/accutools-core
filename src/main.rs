use printpdf::*;
use scraper::{Html, Selector};
use std::{env, fs::File, fmt, error::Error, io::{Read, BufWriter}, char::{MAX, decode_utf16}};

const MAX_DESC_LENGTH: usize = 23;
macro_rules! lpad {
    ($arg:expr) => {{
        format!("{:>12}", $arg)
    }}
}

#[derive(Debug)]
struct ReceiptInfo {
    title: String,
    date: String,
    company_info: String,
    customer_info: String,
    transaction_number: String,
    order_id: String,
    invoice_number: String,
    item_lines: Vec<ItemLine>,
    delivery_tickets: String,
    weigh_tickets: String,
    totals: Vec<Amount>,
    payments: Vec<Amount>,
    amount_due: String,
    employee: String,
    slogan: String,
}

impl ReceiptInfo {
    const fn new() -> Self {
        Self {
            // Initial spans
            title: String::new(),
            date: String::new(),
            // Table one
            company_info: String::new(),
            customer_info: String::new(),
            transaction_number: String::new(),
            order_id: String::new(),
            invoice_number: String::new(),
            // Table three
            item_lines: Vec::new(),
            delivery_tickets: String::new(),
            weigh_tickets: String::new(),
            // Table five
            totals: Vec::new(),
            payments: Vec::new(),
            // Table seven
            amount_due: String::new(),
            // Table Eight
            employee: String::new(),
            // Table Nine
            slogan: String::new(),
        }
    }
}

#[derive(Debug)]
struct ItemLine {
    code: String,
    description: String,
    quantity: String,
    price: String,
    amount: String,
}

impl ItemLine {
    const fn new() -> Self {
        Self {
            code: String::new(),
            description: String::new(),
            quantity: String::new(),
            price: String::new(),
            amount: String::new(),
        }
    }
}

#[derive(Debug)]
struct Amount {
    name: String,
    value: String,
}

impl Amount {
    const fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }
}

impl<'a> Cleanup for scraper::element_ref::Text<'a> {
    fn cleanup(&mut self) -> String {
        let folded = self
            .fold(
                String::new(),
                |acc, string| {
                    format!("{acc}{} ", string.trim())
                }
            );
        return folded.trim().to_owned();
    }

    fn cleanup_amount(&mut self) -> String {
        let folded = self
            .fold(
                String::new(),
                |acc, string| {
                    format!("{acc}{} ", string.trim())
                }
            );
        let amount = folded.trim();
        if amount.starts_with('$') {
            return amount[1..].to_owned();
        } else {
            return amount.to_owned();
        }
    }

    fn cleanup_multiple_lines(&mut self) -> String {
        // First combine
        let folded = self
            .fold(
                String::new(),
                |acc, x| {
                    format!("{acc}{x} ")
                }
        );
        // Second split lines
        let mut lines = Vec::new();
        folded.lines().for_each(|line| 
            lines.push(line
                .split_whitespace()
                .fold(String::new(), |acc, token| format!("{acc}{token} "))
            )
        );
        // Trim Lines
        let mut trimmed_lines = Vec::new();
        lines.iter().for_each(
            |line| trimmed_lines.push(line.trim())
        );
        // Combine non-empty lines
        let result = trimmed_lines
            .iter()
            .filter(|&line| !line.is_empty())
            .fold(String::new(), |acc, &line| format!("{acc}{line}\n"));
        return result.trim_end().to_owned();
    }
}

trait Cleanup {
    fn cleanup(&mut self) -> String;
    fn cleanup_multiple_lines(&mut self) -> String;
    fn cleanup_amount(&mut self) -> String;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_file = args.get(1).unwrap();
    let receipt = parse_html(input_file)?;
    let doc = gen_pdf(&receipt)?; 

    let output_file = if let Some(output_file) = args.get(2) {
        output_file
    } else {
        "receipt.pdf"
    };
    doc.save(&mut BufWriter::new(File::create(output_file).unwrap())).unwrap();
    Ok(())
}

fn gen_pdf(receipt: &ReceiptInfo) -> Result<PdfDocumentReference, Box<dyn Error>> {
    // Create and initialize document
    // 8.5" x 11" = 215.9mm x 279.4mm = 612pt x 792pt
    let (doc, page1, layer1) = PdfDocument::new("PDF_Document_title", Pt(612.0).into(), Pt(792.0).into(), "Layer 1");
    let font_regular = doc.add_external_font(
        File::open("fonts/NotoSans-Regular.ttf")?
    )?;
    let font_bold = doc.add_external_font(
        File::open("fonts/NotoSans-Bold.ttf")?
    )?;
    let font_mono = doc.add_external_font(
        File::open("fonts/NotoSansMono-Regular.tff")?
    )?;
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let left_margin: Mm = Pt(54.0).into();
    let right_margin: Mm = Pt(558.0).into();

    // Add title
    current_layer.use_text("Customer Invoice", 14.0, Pt(260.0).into(), Pt(750.0).into(), &font_bold);

    // Add company header
    let company = "***REMOVED***";
    let company_info = "***REMOVED*** • ***REMOVED*** • ***REMOVED***";
    current_layer.use_text(company, 28.0, Pt(225.0).into(), Pt(712.0).into(), &font_bold);
    current_layer.use_text(company_info, 18.0, Pt(228.0).into(), Pt(690.0).into(), &font_regular);

    // Add logo
    let mut svg_file = File::open("logo.svg")?;
    let mut buf = String::new();
    svg_file.read_to_string(&mut buf).unwrap();
    let logo_svg = printpdf::Svg::parse(&buf).unwrap();
    let logo_transform = SvgTransform {
        translate_x: Some(Pt(55.0)),
        translate_y: Some(Pt(682.0)),
        rotate: None,
        scale_x: Some(0.65),
        scale_y: Some(0.65),
        dpi: None,
    };
    logo_svg.add_to_layer(&current_layer, logo_transform);
    

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
    let text_bottom = headers_bottom_border + Pt(20.0).into();
    current_layer.use_text("Date/Time:"      , font_size, header_positions[0] + spacing, text_bottom, &font_bold);
    // current_layer.use_text("Order ID:"      , font_size, header_positions[1] + spacing, text_bottom, &font_bold);
    current_layer.use_text("Transaction ID:", font_size, header_positions[1] + spacing, text_bottom, &font_bold);
    current_layer.use_text("Invoice Number:", font_size, header_positions[2] + spacing, text_bottom, &font_bold);
    let font_size = 12.0;
    let text_bottom = headers_bottom_border + Pt(4.0).into();
    current_layer.use_text(&receipt.date,      font_size, header_positions[0] + spacing, text_bottom, &font_regular);
    // current_layer.use_text(order_id,           font_size, header_positions[1] + spacing, headers_bottom_border, &font_regular);
    current_layer.use_text(&receipt.transaction_number, font_size, header_positions[1] + spacing, text_bottom, &font_regular);
    current_layer.use_text(&receipt.invoice_number,     font_size + 6.0, header_positions[2] + spacing, text_bottom - Pt(1.0).into(), &font_bold);

    
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
            // let desc_lines = split_into_lines("Interior-crocodile-alligator I drive a chevrolet-movie-theater.", 28);
            let item_num = str::parse::<usize>(&line.code)?;
            let uom = if item_num >= 2000 && item_num < 2100 {
                "EA" // item is a block
            } else {
                "TON" // item is not a block
            };
            let qty = if uom.eq("EA") && line.quantity.ends_with(".000") { 
                format!("{:>6}    ", &line.quantity[..line.quantity.len()-4])
            } else {
                format!("{:>10}", line.quantity)
            };
            current_layer.use_text(&line.code,                 font_size, left_margin  + spacing, cursor_y, &font_mono);
            current_layer.use_text(&desc_lines[0],             font_size, li_vlines[0] + spacing, cursor_y, &font_mono);
            current_layer.use_text(uom,                        font_size, li_vlines[1] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&qty,     font_size, li_vlines[2] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&lpad!(&line.price),   font_size, li_vlines[3] + spacing, cursor_y, &font_mono);
            current_layer.use_text(&lpad!(&line.amount), font_size, li_vlines[4] + spacing, cursor_y, &font_mono);
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
        current_layer.use_text(&amount.name, font_size, x1, current_y, font);
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

fn parse_html(filename: &str) -> Result<ReceiptInfo, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    {
        let mut mail_file = File::open(filename)?;
        let _ = mail_file.read_to_string(&mut buf)?;
    }
    let start_delimiter = "<Html>";
    let end_delimiter = "</Html>";
    let start_index = buf.find(start_delimiter).unwrap();
    let end_index = buf.find(end_delimiter).unwrap() + end_delimiter.len();
    let html_doc = &buf[start_index..end_index];
    let doc = Html::parse_document(html_doc);

    // Create all selectors
    let body_selector = Selector::parse("body").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let table_selector = Selector::parse("table").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();

    let mut receipt_info = ReceiptInfo::new();
    // Everything should be in the body. This is a safety check
    let body = doc.select(&body_selector).next().unwrap();

    // First two strong tags are title and datetime
    let mut span_elements = body.select(&span_selector);
    receipt_info.title = span_elements.next().unwrap().text().cleanup();
    receipt_info.date = span_elements.next().unwrap().text().cleanup();
    drop(span_elements);

    // Everything else in document is in tables
    {
        let mut tables = body.select(&table_selector);
        {
            // Table one is Company name, Customer name, and order metadata
            let first_table = tables.next().unwrap();
            let mut rows = first_table.select(&tr_selector);
            {
                let company_and_customer_row = rows.next().unwrap();
                let mut tds = company_and_customer_row.select(&td_selector);
                receipt_info.company_info = tds
                    .next()
                    .unwrap()
                    .text()
                    .cleanup_multiple_lines();
                receipt_info.customer_info = tds
                    .next()
                    .unwrap()
                    .text()
                    .cleanup_multiple_lines();
            }
            let _ = rows.next().unwrap(); // blank
            {
                let metadata = rows.next().unwrap();
                let mut tds = metadata.select(&td_selector);
                let tnum = tds.next().unwrap().text().cleanup();
                let tnum_prefix = "TransactionNumber: ";
                receipt_info.transaction_number = if tnum.starts_with(tnum_prefix) {
                    tnum[tnum_prefix.len()..].to_owned()
                } else {
                    tnum
                };

                let order_id = tds.next().unwrap().text().cleanup();
                let oid_prefix = "OrderId: ";
                receipt_info.order_id = if order_id.starts_with(oid_prefix) {
                    order_id[oid_prefix.len()..].to_owned()
                } else {
                    order_id
                };

                let invnum = tds.next().unwrap().text().cleanup();
                let invnum_prefix = "Invoice#: ";
                receipt_info.invoice_number = if invnum.starts_with(invnum_prefix) {
                    invnum[invnum_prefix.len()..].to_owned()
                } else {
                    invnum
                };
            }
        }
        // Table two contains table headers. Not used.
        let _ = tables.next().unwrap();
        {
            // Table three contains item lines
            let table_three = tables.next().unwrap();
            let mut dt_nums = Vec::new();
            let mut wt_nums = Vec::new();
            for row in table_three.select(&tr_selector) {
                let mut tds = row.select(&td_selector);
                let code        = tds.next().unwrap().text().next().unwrap().to_owned();
                let description = tds.next().unwrap().text().next().unwrap().to_owned();
                let quantity    = tds.next().unwrap().text().next().unwrap().to_owned();
                let price       = tds.next().unwrap().text().cleanup_amount();
                let amount      = tds.next().unwrap().text().cleanup_amount();
                if code.eq("2300") {
                    dt_nums.push(description);
                } else if code.eq("2301") {
                    wt_nums.push(description);
                } else {
                    let item_line = ItemLine {
                        code,
                        description,
                        quantity,
                        price,
                        amount
                    };
                    receipt_info.item_lines.push(item_line);
                }
            }
            // Fix DT and WT nums
            dt_nums.iter().for_each(|string| {
                let dt_line = string
                    .chars()
                    .filter(|char| char.is_digit(10) || char.is_ascii_punctuation() || char.is_whitespace())
                    .fold(String::new(), |acc, add| format!("{acc}{add}"));
                receipt_info.delivery_tickets.push_str(&format!("{} ", dt_line.trim()));
            });
            receipt_info.delivery_tickets.pop();
            wt_nums.iter().for_each(|string| {
                let wt_line = string
                    .chars()
                    .filter(|char| char.is_digit(10) || char.is_ascii_punctuation() || char.is_whitespace())
                    .fold(String::new(), |acc, add| format!("{acc}{add}"));
                receipt_info.weigh_tickets.push_str(&format!("{} ", wt_line.trim()));
            });
            receipt_info.weigh_tickets.pop();
        }
        // Table 4 is empty
        let _ = tables.next().unwrap();
        {
            // Table 5 is subtotal, tax, total
            let table_five = tables.next().unwrap();
            for row in table_five.select(&tr_selector) {
                let mut tds = row.select(&td_selector);
                receipt_info.totals.push(
                    Amount {
                        name: tds.next().unwrap().text().next().unwrap().to_owned(),
                        value: tds.next().unwrap().text().cleanup_amount(),
                    }
                )
                
            }
        }
        {
            // Table 6 is Payments
            let table_six = tables.next().unwrap();
            for row in table_six.select(&tr_selector) {
                let mut tds = row.select(&td_selector);
                receipt_info.payments.push(
                    Amount {
                        name:  tds.next().unwrap().text().cleanup(),
                        value: tds.next().unwrap().text().cleanup_amount(),
                    }
                )
            }
        }
        {
            // Table seven is Amount Due from customer
            let table_seven = tables.next().unwrap();
            let mut tds = table_seven.select(&td_selector);
            let _empty = tds.next();
            let _name = tds.next();
            let amount = tds.next().unwrap().text().cleanup_amount();
        }
        {
            // Table eight is Employee Name
            let table_eight = tables.next().unwrap();
            let mut tds = table_eight.select(&td_selector);
            let _employee_label = tds.next();
            receipt_info.employee = tds.next().unwrap().text().cleanup();
        }
        {
            // Table nine is Footer With Slogan
            let table_nine = tables.next().unwrap();
            receipt_info.slogan = table_nine.select(&td_selector).next().unwrap().text().next().unwrap().to_owned();
        }
    }
    dbg!(&receipt_info);
    return Ok(receipt_info);
}

// Split any text which goes over a maximimum number of characters into separate
// lines
fn split_into_lines(string: &str, max_length: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if string.is_empty() {
        return Vec::new();
    }

    lines.push(string.to_owned());
    while lines.last().unwrap().len() > max_length {
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