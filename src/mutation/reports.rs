use crate::mutation::types::{MutationReport, ReportFormat, TestOutcome};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use tracing::{info};
use plotters::prelude::*;
use plotters::style::RGBColor;
use serde_json;

const GREEN: RGBColor = RGBColor(0, 255, 0);
const RED: RGBColor = RGBColor(255, 0, 0);
const BLUE: RGBColor = RGBColor(0, 0, 255);
const YELLOW: RGBColor = RGBColor(255, 255, 0);
#[allow(dead_code)]
const GREY: RGBColor = RGBColor(128, 128, 128);

#[allow(dead_code)]
pub struct ReportGenerator;

#[allow(dead_code)]
impl ReportGenerator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn generate_report(&self, report: &MutationReport, format: ReportFormat, output_path: Option<&str>) -> Result<String, String> {
        match format {
            ReportFormat::JSON => self.generate_json_report(report, output_path),
            ReportFormat::CSV => self.generate_csv_report(report, output_path),
            ReportFormat::HTML => self.generate_html_report(report, output_path),
            ReportFormat::Markdown => self.generate_markdown_report(report, output_path),
            ReportFormat::Console => self.generate_console_report(report),
        }
    }

    #[allow(dead_code)]
    pub fn generate_mutation_chart(&self, report: &MutationReport, output_path: &str) -> Result<(), String> {
        let path = Path::new(output_path);
        
        let pie_chart_path = path.join("mutation_outcomes.png");
        self.create_pie_chart(report, pie_chart_path.to_str().unwrap())?;
        
        let bar_chart_path = path.join("mutation_types.png");
        self.create_bar_chart(report, bar_chart_path.to_str().unwrap())?;
        
        info!("Generated mutation charts at {}", output_path);
        Ok(())
    }

    #[allow(dead_code)]
    fn generate_json_report(&self, report: &MutationReport, output_path: Option<&str>) -> Result<String, String> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| format!("Failed to serialize report to JSON: {}", e))?;
            
        if let Some(path) = output_path {
            fs::write(path, &json)
                .map_err(|e| format!("Failed to write JSON report to {}: {}", path, e))?;
            info!("JSON report written to {}", path);
        }
        
        Ok(json)
    }

    #[allow(dead_code)]
    fn generate_csv_report(&self, report: &MutationReport, output_path: Option<&str>) -> Result<String, String> {
        let mut csv_content = String::from("mutation_type,original_code,test_result,execution_time_ms,line,column\n");
        
        for result in &report.results {
            let test_result = match result.test_result {
                TestOutcome::Killed { .. } => "killed",
                TestOutcome::Survived => "survived",
                TestOutcome::Timeout => "timeout",
                TestOutcome::Error => "error",
                TestOutcome::Skipped => "skipped",
            };
            
            let line = format!(
                "{:?},{},{},{},{},{}\n",
                result.candidate.mutation_type,
                result.candidate.original_code.replace(',', "\\,"),
                test_result,
                result.execution_time_ms,
                result.candidate.line,
                result.candidate.column
            );
            
            csv_content.push_str(&line);
        }
        
        if let Some(path) = output_path {
            fs::write(path, &csv_content)
                .map_err(|e| format!("Failed to write CSV report to {}: {}", path, e))?;
            info!("CSV report written to {}", path);
        }
        
        Ok(csv_content)
    }

    #[allow(dead_code)]
    fn generate_html_report(&self, report: &MutationReport, output_path: Option<&str>) -> Result<String, String> {
        let mut html = String::from(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Mutation Testing Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
        .summary { background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
        th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
        .killed { background-color: #d4edda; }
        .survived { background-color: #f8d7da; }
        .timeout { background-color: #fff3cd; }
        .error { background-color: #f5c6cb; }
        .skipped { background-color: #e2e3e5; }
        .score-high { color: green; }
        .score-medium { color: orange; }
        .score-low { color: red; }
    </style>
</head>
<body>
    <h1>Mutation Testing Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Mutations: "#);
        
        html.push_str(&format!("{}</p>", report.total_mutations));
        html.push_str(&format!("<p>Killed Mutations: {}</p>", report.killed_mutations));
        html.push_str(&format!("<p>Survived Mutations: {}</p>", report.survived_mutations));
        html.push_str(&format!("<p>Error Mutations: {}</p>", report.error_mutations));
        html.push_str(&format!("<p>Timeout Mutations: {}</p>", report.timeout_mutations));
        html.push_str(&format!("<p>Skipped Mutations: {}</p>", report.skipped_mutations));
        
        let score_class = if report.mutation_score >= 80.0 {
            "score-high"
        } else if report.mutation_score >= 60.0 {
            "score-medium"
        } else {
            "score-low"
        };
        
        html.push_str(&format!(
            r#"<p>Mutation Score: <span class="{}">{:.2}%</span></p>
            <p>Execution Time: {:.2} seconds</p>
        </div>"#,
            score_class, report.mutation_score, report.execution_time_seconds
        ));
        
        html.push_str(r#"
    <h2>Mutation Results</h2>
    <table>
        <thead>
            <tr>
                <th>Mutation Type</th>
                <th>Line</th>
                <th>Column</th>
                <th>Original Code</th>
                <th>Mutated Code</th>
                <th>Result</th>
                <th>Execution Time (ms)</th>
            </tr>
        </thead>
        <tbody>
"#);
        
        for result in &report.results {
            let row_class = match result.test_result {
                TestOutcome::Killed { .. } => "killed",
                TestOutcome::Survived => "survived",
                TestOutcome::Timeout => "timeout",
                TestOutcome::Error => "error",
                TestOutcome::Skipped => "skipped",
            };
            
            let test_result = match &result.test_result {
                TestOutcome::Killed { killing_tests } => format!("Killed (by {} tests)", killing_tests.len()),
                TestOutcome::Survived => "Survived".to_string(),
                TestOutcome::Timeout => "Timeout".to_string(),
                TestOutcome::Error => "Error".to_string(),
                TestOutcome::Skipped => "Skipped".to_string(),
            };
            
            html.push_str(&format!(
                r#"<tr class="{}">
                    <td>{:?}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><pre>{}</pre></td>
                    <td><pre>{}</pre></td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                row_class,
                result.candidate.mutation_type,
                result.candidate.line,
                result.candidate.column,
                html_escape(&result.candidate.original_code),
                html_escape(&result.mutated_code),
                test_result,
                result.execution_time_ms
            ));
        }
        
        html.push_str(r#"
        </tbody>
    </table>
</body>
</html>
"#);
        
        if let Some(path) = output_path {
            fs::write(path, &html)
                .map_err(|e| format!("Failed to write HTML report to {}: {}", path, e))?;
            info!("HTML report written to {}", path);
        }
        
        Ok(html)
    }

    #[allow(dead_code)]
    fn generate_markdown_report(&self, report: &MutationReport, output_path: Option<&str>) -> Result<String, String> {
        let mut md = String::from("# Mutation Testing Report\n\n");
        
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Total Mutations**: {}\n", report.total_mutations));
        md.push_str(&format!("- **Killed Mutations**: {}\n", report.killed_mutations));
        md.push_str(&format!("- **Survived Mutations**: {}\n", report.survived_mutations));
        md.push_str(&format!("- **Error Mutations**: {}\n", report.error_mutations));
        md.push_str(&format!("- **Timeout Mutations**: {}\n", report.timeout_mutations));
        md.push_str(&format!("- **Skipped Mutations**: {}\n", report.skipped_mutations));
        md.push_str(&format!("- **Mutation Score**: {:.2}%\n", report.mutation_score));
        md.push_str(&format!("- **Execution Time**: {:.2} seconds\n\n", report.execution_time_seconds));
        
        md.push_str("## Mutation Results\n\n");
        md.push_str("| Mutation Type | Line | Column | Original Code | Result | Execution Time (ms) |\n");
        md.push_str("|--------------|------|--------|--------------|--------|--------------------|\n");
        
        for result in &report.results {
            let test_result = match &result.test_result {
                TestOutcome::Killed { killing_tests } => format!("✅ Killed (by {} tests)", killing_tests.len()),
                TestOutcome::Survived => "❌ Survived".to_string(),
                TestOutcome::Timeout => "⏱️ Timeout".to_string(),
                TestOutcome::Error => "⚠️ Error".to_string(),
                TestOutcome::Skipped => "⏭️ Skipped".to_string(),
            };
            
            md.push_str(&format!(
                "| {:?} | {} | {} | `{}` | {} | {} |\n",
                result.candidate.mutation_type,
                result.candidate.line,
                result.candidate.column,
                result.candidate.original_code.replace('|', "\\|").replace('`', "\\`"),
                test_result,
                result.execution_time_ms
            ));
        }
        
        if let Some(path) = output_path {
            fs::write(path, &md)
                .map_err(|e| format!("Failed to write Markdown report to {}: {}", path, e))?;
            info!("Markdown report written to {}", path);
        }
        
        Ok(md)
    }

    #[allow(dead_code)]
    fn generate_console_report(&self, report: &MutationReport) -> Result<String, String> {
        let mut output = String::new();
        
        output.push_str("\n=== MUTATION TESTING REPORT ===\n\n");
        output.push_str(&format!("Total Mutations: {}\n", report.total_mutations));
        output.push_str(&format!("Killed Mutations: {}\n", report.killed_mutations));
        output.push_str(&format!("Survived Mutations: {}\n", report.survived_mutations));
        output.push_str(&format!("Error Mutations: {}\n", report.error_mutations));
        output.push_str(&format!("Timeout Mutations: {}\n", report.timeout_mutations));
        output.push_str(&format!("Skipped Mutations: {}\n", report.skipped_mutations));
        output.push_str(&format!("Mutation Score: {:.2}%\n", report.mutation_score));
        output.push_str(&format!("Execution Time: {:.2} seconds\n\n", report.execution_time_seconds));
        
        output.push_str("Survived Mutations (need better tests):\n");
        output.push_str("----------------------------------------\n");
        
        let mut has_survived = false;
        for result in &report.results {
            if matches!(result.test_result, TestOutcome::Survived) {
                has_survived = true;
                output.push_str(&format!(
                    "Line {}, Col {}: {:?} '{}'\n",
                    result.candidate.line,
                    result.candidate.column,
                    result.candidate.mutation_type,
                    result.candidate.original_code
                ));
                
                if let Some(suggested) = &result.suggested_improvement {
                    output.push_str(&format!("Suggestion: {}\n", suggested));
                }
                output.push_str("\n");
            }
        }
        
        if !has_survived {
            output.push_str("No survived mutations! Great test coverage.\n");
        }
        
        output.push_str("\n=== END OF REPORT ===\n");
        
        Ok(output)
    }

    #[allow(dead_code)]
    fn create_pie_chart(&self, report: &MutationReport, output_path: &str) -> Result<(), String> {
        let root = BitMapBackend::new(output_path, (800, 600))
            .into_drawing_area();
            
        root.fill(&WHITE)
            .map_err(|e| format!("Failed to create chart: {}", e))?;
            
        let mut chart = ChartBuilder::on(&root)
            .caption("Mutation Testing Results", ("sans-serif", 40))
            .build_cartesian_2d(0.0..1.0, 0.0..1.0)
            .map_err(|e| format!("Failed to build chart: {}", e))?;
            
        chart.configure_mesh()
            .disable_mesh()
            .draw()
            .map_err(|e| format!("Failed to configure chart: {}", e))?;
            
        let total = report.total_mutations as f64;
        if total == 0.0 {
            return Err("No mutations to visualize".to_string());
        }
        
        let killed = report.killed_mutations as f64 / total;
        let survived = report.survived_mutations as f64 / total;
        let error = report.error_mutations as f64 / total;
        let timeout = report.timeout_mutations as f64 / total;
        let skipped = report.skipped_mutations as f64 / total;
        
        let values = vec![
            ("Killed", killed, &GREEN),
            ("Survived", survived, &RED),
            ("Error", error, &YELLOW),
            ("Timeout", timeout, &BLUE),
            ("Skipped", skipped, &GREY),
        ];
        
        let mut start_angle = 0.0;
        for (_i, (label, value, color)) in values.iter().enumerate() {
            if *value > 0.0 {
                let end_angle = start_angle + value * 360.0;
                let y = 540 - (start_angle / 400.0 * 600.0) as i32;
                let rect = [(80, y), (160, y - 30)];
                root.draw(&Rectangle::new(rect, ShapeStyle::from(*color).filled()))
                    .map_err(|e| format!("Failed to draw chart: {}", e))?;
                let text_y = y - 15;
                let text = format!("{}: {:.1}% ({})", label, value * 100.0, if *label == "Killed" { report.killed_mutations } else if *label == "Survived" { report.survived_mutations } else if *label == "Error" { report.error_mutations } else if *label == "Timeout" { report.timeout_mutations } else { report.skipped_mutations });
                root.draw(&Text::new(
                    text,
                    (180, text_y),
                    ("sans-serif", 20)
                ))
                .map_err(|e| format!("Failed to draw chart: {}", e))?;
                start_angle = end_angle;
            }
        }
        root.draw(&Text::new(
            format!("Mutation Score: {:.1}%", report.mutation_score),
            (400, 180),
            ("sans-serif", 30)
        ))
        .map_err(|e| format!("Failed to draw chart: {}", e))?;
        
        root.present()
            .map_err(|e| format!("Failed to save chart: {}", e))?;
            
        Ok(())
    }

    #[allow(dead_code)]
    fn create_bar_chart(&self, report: &MutationReport, output_path: &str) -> Result<(), String> {
        // Count mutations by type
        let mut type_counts: HashMap<String, i32> = HashMap::new();
        
        for result in &report.results {
            let type_name = format!("{:?}", result.candidate.mutation_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        let mut types: Vec<String> = type_counts.keys().cloned().collect();
        types.sort();
        
        let root = BitMapBackend::new(output_path, (800, 600))
            .into_drawing_area();
            
        root.fill(&WHITE)
            .map_err(|e| format!("Failed to create chart: {}", e))?;
            
        let max_count = *type_counts.values().max().unwrap_or(&0) as f32;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Mutations by Type", ("sans-serif", 40))
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0i32..(types.len() as i32),
                0.0..max_count * 1.2
            )
            .map_err(|e| format!("Failed to build chart: {}", e))?;
            
        chart.configure_mesh()
            .x_labels(types.len())
            .y_desc("Count")
            .x_label_formatter(&|idx| {
                if (*idx as usize) < types.len() {
                    types[*idx as usize].clone()
                } else {
                    "".to_string()
                }
            })
            .draw()
            .map_err(|e| format!("Failed to configure chart: {}", e))?;
            
        // Draw bars
        chart.draw_series(
            types.iter().enumerate().map(|(i, t)| {
                let count = *type_counts.get(t).unwrap_or(&0) as f32;
                let bar = Rectangle::new(
                    [(i as i32, 0.0), ((i+1) as i32, count)],
                    HSLColor(i as f64 / types.len() as f64, 0.8, 0.5).filled()
                );
                bar
            })
        )
        .map_err(|e| format!("Failed to draw chart: {}", e))?;
        
        root.present()
            .map_err(|e| format!("Failed to save chart: {}", e))?;
            
        Ok(())
    }
}

#[allow(dead_code)]
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation::types::{MutationCandidate, MutationType};
    
    #[test]
    fn test_generate_json_report() {
        let report = create_test_report();
        let generator = ReportGenerator::new();
        
        let result = generator.generate_report(&report, ReportFormat::JSON, None);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("\"total_mutations\":"));
        assert!(json.contains("\"killed_mutations\":"));
        assert!(json.contains("\"mutation_score\":"));
    }
    
    #[test]
    fn test_generate_csv_report() {
        let report = create_test_report();
        let generator = ReportGenerator::new();
        
        let result = generator.generate_report(&report, ReportFormat::CSV, None);
        assert!(result.is_ok());
        
        let csv = result.unwrap();
        assert!(csv.contains("mutation_type,original_code,test_result"));
        assert!(csv.contains("ArithmeticOperator"));
    }
    
    #[test]
    fn test_generate_markdown_report() {
        let report = create_test_report();
        let generator = ReportGenerator::new();
        
        let result = generator.generate_report(&report, ReportFormat::Markdown, None);
        assert!(result.is_ok());
        
        let md = result.unwrap();
        assert!(md.contains("# Mutation Testing Report"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("## Mutation Results"));
    }
    
    fn create_test_report() -> MutationReport {
        let mut report = MutationReport::new();
        
        let candidate = MutationCandidate {
            line: 10,
            column: 5,
            original_code: "+".to_string(),
            mutation_type: MutationType::ArithmeticOperator,
            suggested_mutations: vec!["-".to_string()],
        };
        
        let result = crate::mutation::types::MutationResult {
            candidate: candidate.clone(),
            mutated_code: "a - b".to_string(),
            test_result: TestOutcome::Killed { killing_tests: vec!["test1".to_string()] },
            execution_time_ms: 100,
            error_message: None,
            killing_tests: Some(vec!["test1".to_string()]),
            suggested_improvement: None,
        };
        
        report.add_result(result);
        report
    }
}
