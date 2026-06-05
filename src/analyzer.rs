use crate::gas;
use eyre::{eyre, Result};
use serde::Serialize;
use std::path::Path;
use wasmparser::{Parser, Payload};

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub index: u32,
    pub name: Option<String>,
    pub body_size: usize,
    pub local_count: u32,
    pub instruction_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WasmAnalysis {
    pub file_size: usize,
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub memory_pages: Option<u32>,
    pub table_count: u32,
    pub custom_sections: Vec<(String, usize)>,
    pub data_segment_size: usize,
    pub schema_version: String,
    pub binary_size_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    pub schema_version: String,
    pub binary_size_bytes: usize,
    pub total_code_size: usize,
    pub total_functions: usize,
    pub functions: Vec<FunctionReport>,
    pub sections: Vec<SectionReport>,
}

#[derive(Debug, Serialize)]
pub struct FunctionReport {
    pub index: u32,
    pub name: String,
    pub body_size: usize,
    pub instruction_count: usize,
    pub estimated_gas: u64,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SectionReport {
    pub name: String,
    pub size_bytes: usize,
    pub kind: String,
}

impl WasmAnalysis {
    pub fn to_report(&self) -> AnalysisReport {
        let mut sorted = self.functions.clone();
        sorted.sort_by(|a, b| b.body_size.cmp(&a.body_size));

        let functions: Vec<FunctionReport> = sorted
            .iter()
            .map(|f| {
                let default_name = format!("func_{}", f.index);
                FunctionReport {
                    index: f.index,
                    name: f.name.clone().unwrap_or(default_name),
                    body_size: f.body_size,
                    instruction_count: f.instruction_count,
                    estimated_gas: gas::estimate_gas(f),
                    suggestion: if f.body_size > 1024 {
                        Some(format!("large function ({} bytes), consider splitting", f.body_size))
                    } else {
                        None
                    },
                }
            })
            .collect();

        let mut sections = Vec::new();
        sections.push(SectionReport {
            name: "code".into(),
            size_bytes: self.total_code_size(),
            kind: "code".into(),
        });
        sections.push(SectionReport {
            name: "data".into(),
            size_bytes: self.data_segment_size,
            kind: "data".into(),
        });
        for (name, sz) in &self.custom_sections {
            sections.push(SectionReport {
                name: name.clone(),
                size_bytes: *sz,
                kind: "custom".into(),
            });
        }

        AnalysisReport {
            schema_version: self.schema_version.clone(),
            binary_size_bytes: self.binary_size_bytes,
            total_code_size: self.total_code_size(),
            total_functions: self.total_functions(),
            functions,
            sections,
        }
    }

    pub fn total_code_size(&self) -> usize {
        self.functions.iter().map(|f| f.body_size).sum()
    }

    pub fn total_functions(&self) -> usize {
        self.functions.len()
    }
}

pub fn analyze(path: &Path) -> Result<WasmAnalysis> {
    let data = std::fs::read(path).map_err(|e| eyre!("Cannot read {}: {}", path.display(), e))?;

    let mut functions = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut memory_pages = None;
    let mut table_count = 0u32;
    let mut custom_sections = Vec::new();
    let mut data_segment_size = 0usize;
    let mut func_index = 0u32;

    let parser = Parser::new(0);
    for payload in parser.parse_all(&data) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    imports.push(format!("{}.{}", import.module, import.name));
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    exports.push(export.name.to_string());
                }
            }
            Payload::MemorySection(reader) => {
                for mem in reader {
                    let mem = mem?;
                    memory_pages = Some(mem.initial as u32);
                }
            }
            Payload::TableSection(reader) => {
                for _ in reader {
                    table_count += 1;
                }
            }
            Payload::CodeSectionEntry(body) => {
                let body_range = body.range();
                let body_size = body_range.end - body_range.start;

                let locals_reader = body.get_locals_reader()?;
                let mut local_count = 0u32;
                for local in locals_reader {
                    let (count, _) = local?;
                    local_count += count;
                }

                // Count instructions
                let mut instruction_count = 0usize;
                let ops_reader = body.get_operators_reader()?;
                for op in ops_reader {
                    let _ = op?;
                    instruction_count += 1;
                }

                functions.push(FunctionInfo {
                    index: func_index,
                    name: None, // populated from name section if available
                    body_size,
                    local_count,
                    instruction_count,
                });
                func_index += 1;
            }
            Payload::CustomSection(section) => {
                custom_sections.push((
                    section.name().to_string(),
                    section.data().len(),
                ));
            }
            Payload::DataSection(reader) => {
                for segment in reader {
                    let segment = segment?;
                    data_segment_size += segment.data.len();
                }
            }
            _ => {}
        }
    }

    Ok(WasmAnalysis {
        file_size: data.len(),
        functions,
        imports,
        exports,
        memory_pages,
        table_count,
        custom_sections,
        data_segment_size,
        schema_version: String::from("1.0.0"),
        binary_size_bytes: data.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    fn minimal_exported_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // header
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type section: () -> ()
            0x03, 0x02, 0x01, 0x00, // function section: one function
            0x07, 0x07, 0x01, 0x03, b'r', b'u', b'n', 0x00, 0x00, // export run()
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b, // code section: empty body
        ]
    }

    #[test]
    fn test_valid_wasm() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        // Minimal valid WASM module header
        file.write_all(b"\x00asm\x01\x00\x00\x00")?;
        
        let analysis = analyze(file.path())?;
        assert_eq!(analysis.binary_size_bytes, 8);
        assert_eq!(analysis.total_functions(), 0);
        assert_eq!(analysis.total_code_size(), 0);
        Ok(())
    }

    #[test]
    fn analyzes_valid_wasm_with_exported_function() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        let bytes = minimal_exported_wasm();
        file.write_all(&bytes)?;

        let analysis = analyze(file.path())?;

        assert_eq!(analysis.file_size, bytes.len());
        assert_eq!(analysis.binary_size_bytes, bytes.len());
        assert_eq!(analysis.total_functions(), 1);
        assert_eq!(analysis.exports, vec!["run"]);
        assert_eq!(analysis.functions[0].index, 0);
        assert!(analysis.functions[0].body_size > 0);
        assert!(analysis.functions[0].instruction_count > 0);
        Ok(())
    }

    #[test]
    fn test_empty_binary_handling() {
        let file = NamedTempFile::new().unwrap();
        // Empty file
        
        let result = analyze(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_wasm_rejection() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not a wasm file").unwrap();
        
        let result = analyze(file.path());
        assert!(result.is_err());
    }
}
