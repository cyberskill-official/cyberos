//! Minimal PDF export.

use crate::error::ViewError;
use crate::views::ViewResponse;

/// Render a small text PDF containing every row kind.
pub fn render_pdf(view: &ViewResponse) -> Result<Vec<u8>, ViewError> {
    let mut lines = vec![
        format!("CyberOS Compliance View: {}", view.regulation),
        format!("Tenant: {}", view.tenant_id),
        format!("Rows: {}", view.rows.len()),
    ];
    for row in &view.rows {
        lines.push(format!("{} {}", row.ts.to_rfc3339(), row.kind));
    }
    let text = escape_pdf_text(&lines.join("\\n"));
    let stream = format!("BT /F1 10 Tf 50 760 Td ({text}) Tj ET");
    let objects = [
        "1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj".to_string(),
        "2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj".to_string(),
        "3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >> endobj".to_string(),
        "4 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj".to_string(),
        format!(
            "5 0 obj << /Length {} >> stream\n{}\nendstream endobj",
            stream.len(),
            stream
        ),
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
        pdf.push(b'\n');
    }
    let xref_offset = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    Ok(pdf)
}

fn escape_pdf_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
