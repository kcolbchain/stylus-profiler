use eyre::{eyre, Result};
use std::path::Path;
use wasmparser::{Parser, Payload};

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub index: u32,
    pub name: Option<String>,
    pub body_size: usize,
    pub local_count: u32,
    pub instruction_count: usize,
}

#[derive(Debug)]
pub struct WasmAnalysis {
    pub file_size: usize,
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub memory_pages: Option<u32>,
    pub table_count: u32,
    pub custom_sections: Vec<(String, usize)>,
    pub data_segment_size: usize,
}

impl WasmAnalysis {
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
    })
}
