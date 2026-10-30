use crate::models::{Bill, User};

/// The bill as a one-page PDF, for `viewer`.
pub fn render_bill_pdf(bill: &Bill, customer: &User, viewer: &User) -> Vec<u8> {
    build_pdf(&build_bill_pdf_lines(bill, customer, viewer))
}

pub fn build_bill_pdf_lines(bill: &Bill, customer: &User, viewer: &User) -> Vec<String> {
    vec![
        "Electricity Bill".to_string(),
        format!("Bill ID: {}", bill.id),
        format!("Customer: {}", customer.name),
        format!("Customer Display: {}", customer.display_name()),
        format!("Meter: {}", customer.meter_number),
        format!("Billing Month: {}", bill.month),
        format!("Units: {:.2}", bill.units),
        format!("Rate per Unit: {:.2}", bill.rate_per_unit.0 as f64 / 100.0),
        format!("Fixed Charge: {:.2}", bill.fixed_charge.0 as f64 / 100.0),
        format!("Amount Due: {:.2}", bill.amount_due.0 as f64 / 100.0),
        format!("Due Date: {}", bill.due_date),
        format!("Status: {}", bill.status),
        format!("Viewer: {}", viewer.name),
    ]
}

pub fn render_bill_pdf_from_lines(lines: &[String]) -> Vec<u8> {
    build_pdf(lines)
}

fn build_pdf(lines: &[String]) -> Vec<u8> {
    let content = build_content_stream(lines);
    let mut objects = Vec::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objects.push(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec());
    objects.push(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    objects.push(format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content).into_bytes());

    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    let mut xref = Vec::new();
    for (i, obj) in objects.iter().enumerate() {
        xref.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(obj);
        out.extend_from_slice(b"\nendobj\n");
    }
    let startxref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for off in xref {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objects.len() + 1, startxref).as_bytes());
    out
}

fn build_content_stream(lines: &[String]) -> String {
    let mut content = String::from("BT\n/F1 16 Tf\n50 780 Td\n");
    for (idx, line) in lines.iter().enumerate() {
        let escaped = escape_pdf_text(line);
        if idx == 0 {
            content.push_str(&format!("({}) Tj\n", escaped));
        } else {
            content.push_str(&format!("0 -22 Td\n({}) Tj\n", escaped));
        }
    }
    content.push_str("ET");
    content
}

fn escape_pdf_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' | '\r' | '\t' => " ".to_string(),
            c if c.is_ascii_graphic() || c == ' ' => c.to_string(),
            _ => "?".to_string(),
        })
        .collect()
}
