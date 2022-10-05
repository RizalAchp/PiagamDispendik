#![allow(dead_code)]

pub mod tokens;
pub mod validations;
pub mod xlsx_reader;

use crate::lang;
use regex::Regex;
use std::io::{Read, Write};
use std::path::Path;
use std::str::from_utf8;
use std::{collections::HashMap, fs::File, path::PathBuf};

/// Alias for a set of tokens (placeholders).
pub type TokenPack = Vec<String>;
pub type TokenPackArg<'a> = &'a [String];

/// Alias for a set of values to be filled into placeholders.
pub type ValuePack = Vec<String>;
pub type ValuePackArg<'a> = &'a [String];

type DocxResult<T> = Result<T, DocxError>;

/// Error returned on failure of some of the docx-filler methods.
/// String representation should give details on what specifically went wrong.
#[derive(Debug, thiserror::Error)]
pub enum DocxError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Ziping on file docx error")]
    Zip(#[from] zip::result::ZipError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Processing error: {0}")]
    Processing(String),
    #[error("Parsing csv file")]
    CsvError(#[from] csv::Error),
    
    #[error("PiagamDispendikError {0}")]
    ProagramDispendikError(String),

    #[error("Unknown Erorr on PiagamDispendik Application")]
    UnknownError,
}

pub type FileMap = HashMap<String, Vec<u8>>;

trait InputOutputPiagam {}

#[derive(Debug, Default)]
pub struct DataInput {
    pub input_path: PathBuf,
    pub target_xml: Option<String>,
    pub header: Option<Vec<String>>,
    pub file_data: Vec<Vec<String>>,
}

impl DataInput {
    pub fn open<P: AsRef<Path>>(input: P) -> DocxResult<Self> {
        if input.as_ref().ends_with(".csv") {
            let mut rdr = csv::Reader::from_path(input.as_ref())?;
            let header = if rdr.has_headers() {
                Some(rdr.headers()?.deserialize(None)?)
            } else {
                None
            };
            let data = rdr
                .deserialize()
                .map(|f| f.unwrap())
                .collect::<Vec<Vec<String>>>();

            Ok(Self {
                input_path: input.as_ref().to_path_buf(),
                target_xml: None,
                file_data: if header != None { data[1..].to_vec() } else { data },
                header,
            })
        } else {
            todo!("excel document not implemented yet");
            // Ok(Self {
            // input_path: todo!("excel document not implemented yet"),
            // target_xml: (),
            // file_data: (),
            // })
        }
    }
}

/// Main DOCX filler / document generator.
///
/// Loads the contents of DOCX template file into memory (beware huge files).
#[derive(Debug)]
pub struct DocxTemplate {
    /// input path of the DOCX template loaded by this struct.
    pub input_path: PathBuf,

    /// filename/path of the DOCX contents (actual text of the DOCX document).
    pub target_xml: String,

    /// in-memory storage of all the DOCX contents/meta-data.
    pub file_data: FileMap,
}

impl DocxTemplate {
    pub fn open<P: AsRef<Path>>(input: P) -> DocxResult<Self> {
        let mut file_map: FileMap = Default::default();
        let mut zip_content = zip::ZipArchive::new(File::open(input.as_ref())?)?;
        file_map.reserve(zip_content.len());
        for i in 0..zip_content.len() {
            let mut entry = zip_content.by_index(i)?;
            let mut file_buffer = Vec::new();
            if let Ok(_) = entry.read_to_end(&mut file_buffer) {
                let name = &entry.name();
                file_map.insert(name.to_string(), file_buffer);
            };
        }

        Ok(Self {
            input_path: input.as_ref().to_path_buf(),
            target_xml: "word/document.xml".to_owned(),
            file_data: file_map,
        })
    }

    /// Get the tokens identified in the DOCX template.
    ///
    /// # Errors
    ///
    /// Can return errors if no DOCX is loaded when attempting this,
    /// or when parsing of tokens fail.
    pub fn template_tokens(&self) -> DocxResult<TokenPack> {
        let document = self.document_contents()?;

        let re = match Regex::new(r"\{\{.*?\}\}") {
            Ok(re) => re,
            Err(err) => {
                return Err(DocxError::Processing(err.to_string()));
            }
        };

        let caps = re.captures_iter(&document);

        let mut tokens: TokenPack = Default::default();
        for cap in caps {
            if let Some(token) = cap.get(0) {
                let token_str = token.as_str().to_owned();
                if !tokens.contains(&token_str) {
                    tokens.push(token_str);
                }
            }
        }

        validations::validate_tokens(&tokens)?;
        Ok(tokens)
    }

    /// Get the whole textual content of the DOCX template document.
    fn document_contents(&self) -> DocxResult<String> {
        match self.file_data.get(&self.target_xml) {
            Some(document) => match from_utf8(&document) {
                Ok(c) => Ok(c.to_owned()),
                Err(_) => Err(DocxError::Validation(
                    "error on parsing utf8 string".to_owned(),
                )),
            },
            None => Err(DocxError::Processing(
                "file input contents is empty!".to_owned(),
            )),
        }
    }

    /// Generates a single DOCX file from the loaded template.
    /// Replaces all the tokens/placeholders with the corresponding input values.
    /// This method can be used repeatedly to generate multiple output files with various input tokens/values.
    ///
    /// # Arguments
    ///
    /// * `tokens` - vector of tokens to be replaced
    /// * `values` - vector of values to be filled in place of tokens
    /// * `output_pattern` - output file pattern (explicit string or pattern contains tokens)
    ///
    /// # Errors
    ///
    /// Can return errors on inconsistent input data or other internal problems (see error message for details).
    pub fn build_docx(
        &self,
        tokens: TokenPackArg,
        values: ValuePackArg,
        output_pattern: &str,
    ) -> DocxResult<()> {
        validations::validate_single(tokens, values, output_pattern)?;
        self.data_to_docx(tokens, values, output_pattern)?;
        Ok(())
    }

    /// Common executive method for processing one docx file generation form the loaded template.
    ///
    /// # Arguments
    ///
    /// * `tokens` - vector of tokens to be replaced
    /// * `values` - vector of values to be filled in place of tokens
    /// * `output_pattern` - output file pattern (explicit string or pattern contains tokens)
    ///
    /// # Errors
    ///
    /// Can return errors on inconsistent input data or other internal problems (see error message for details).
    fn data_to_docx(
        &self,
        tokens: TokenPackArg,
        values: ValuePackArg,
        output_pattern: &str,
    ) -> DocxResult<()> {
        let out_str = replace_tokens(output_pattern, tokens, values);

        let out_path = PathBuf::from(&out_str);
        if out_path.exists() {
            let args: lang::TrArgVec = vec![("filename".to_string(), out_str)];
            let msg = lang::tr_with_args("docx-filler-fail-overwrite", &args);
            return Err(DocxError::Processing(msg));
        }

        let zip_file = File::create(out_path)?;
        let mut zip = zip::ZipWriter::new(zip_file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        for (name, content) in self.file_data.iter() {
            zip.start_file(name, options)?;
            zip.write_all(content.as_slice())?;
        }

        let orig_document = self.document_contents()?;

        let updated_document = replace_tokens(&orig_document, tokens, values);
        zip.start_file(&self.target_xml, options)?;
        zip.write_all(updated_document.as_bytes())?;
        zip.finish()?;

        Ok(())
    }

    /// Generates batch of DOCX files form  the loaded template, one per each line of values in the input text.
    ///
    /// # Arguments
    ///
    /// * `tokens` - vector of tokens to be replaced
    /// * `text` - one to many lines of text - a set of values per each line for a new document to be generated
    /// * `output_pattern` - output file pattern (explicit string or pattern contains tokens)
    ///
    /// # Errors
    ///
    /// Can return error on failure, with details in the error message.
    pub fn build_docx_batch(
        &self,
        tokens: TokenPackArg,
        text: &Vec<ValuePack>, // TODO change into some line iterator?
        output_pattern: &str,
    ) -> DocxResult<()> {
        validations::validate_batch(tokens, text, output_pattern)?;

        for line in text {
            self.data_to_docx(tokens, &line, output_pattern)?;
        }

        Ok(())
    }
}

/// Fill in the input string with specified set of tokens and values.
fn replace_tokens(input: &str, tokens: TokenPackArg, values: ValuePackArg) -> String {
    assert_eq!(tokens.len(), values.len());
    let mut output: String = input.to_string();
    tokens.iter().enumerate().for_each(|(idx, token)| {
        output = output.replace(token, &values[idx]);
    });
    output
}

/// Parse the input string into set of values.
fn string_to_values(input: &str, separator: &str) -> ValuePack {
    input
        .split(separator)
        .filter(|spr| !spr.is_empty())
        .map(|x| x.trim().to_owned())
        .collect()
}

fn vec_str_to_values(input: &Vec<String>) -> ValuePack {
    input
        .into_iter()
        .filter(|spr| !spr.is_empty())
        .map(|x| x.trim().to_owned())
        .collect()
}
