mod consts;
mod data_ui;
mod options_ui;
mod output_ui;
mod template_ui;
mod tokens_ui;
mod values_ui;

use crate::lang;
use crate::piagamdispendik::{DocxTemplate, DataInput, ValuePack};
use crate::ui::{
    options_ui::OptionsUi, output_ui::OutputUi, template_ui::TemplateUi, tokens_ui::TokensUi,
    values_ui::ValuesUi,
};

use nwd::*;
use nwg::stretch::style::FlexDirection;
use nwg::NativeUi;
use std::cell::RefCell;
use std::path::Path;

use self::data_ui::DataUi;

pub fn init_app() {
    nwg::init().unwrap_or_else(|_| panic!("{}", lang::tr("ui-docx-fail-init")));

    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(20)
        .build(&mut font)
        .unwrap_or_else(|_| panic!("{}", lang::tr("ui-docx-fail-build")));
    nwg::Font::set_global_default(Some(font));
    nwg::enable_visual_styles();

    let _app = PiagamDispendikApp::build_ui(Default::default())
        .unwrap_or_else(|_| panic!("{}", lang::tr("ui-docx-fail-build")));
    nwg::dispatch_thread_events();
}

#[derive(NwgUi, Default)]
pub struct PiagamDispendikApp {
    opened_docx: RefCell<Option<DocxTemplate>>,
    opened_data: RefCell<Option<DataInput>>,

    #[nwg_control(title: &lang::tr("ui-docx-app-title"), size: (960, 540), position: (80, 60), accept_files: true)]
    #[nwg_events(OnWindowClose: [PiagamDispendikApp::exit(SELF)], OnFileDrop: [PiagamDispendikApp::load_drop_files(SELF, EVT_DATA)])]
    window: nwg::Window,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Column, auto_spacing: Some(5), padding: consts::WINDOW_PAD)]
    main_layout: nwg::FlexboxLayout,

    // // configuration options controls
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_shrink: 1.0, min_size: consts::MIN_ONELINER_SIZE)]
    options_frame: nwg::Frame,
    #[nwg_partial(parent: options_frame)]
    #[nwg_events((lang_dropdown, OnComboxBoxSelection): [PiagamDispendikApp::set_lang(SELF)])]
    options_partial: OptionsUi,

    // template related controls - open / show currently opened DOCX template file
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_shrink: 1.0, min_size: consts::MIN_TWOLINER_SIZE)]
    template_frame: nwg::Frame,
    #[nwg_partial(parent: template_frame)]
    #[nwg_events((button, OnButtonClick): [PiagamDispendikApp::open_new_file_docx(SELF)])]
    template_partial: TemplateUi,

    // template related controls - open / show currently opened data xlsx/csv file
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_shrink: 1.0, min_size: consts::MIN_TWOLINER_SIZE)]
    data_frame: nwg::Frame,
    #[nwg_partial(parent: data_frame)]
    #[nwg_events((button, OnButtonClick): [PiagamDispendikApp::open_new_file_data(SELF)])]
    data_partial: DataUi,

    // template tokens - list all the tokens found in opened docx, and allow changing order (by rather limited win32 lib UI controls)
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_shrink: 1.0, min_size: consts::MIN_TWOLINER_SIZE)]
    tokens_frame: nwg::Frame,
    #[nwg_partial(parent: tokens_frame)]
    tokens_partial: TokensUi,

    // multi-line field for values to be filled into template(s) - one line per standalone document
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_grow: 1.0)]
    values_frame: nwg::Frame,
    #[nwg_partial(parent: values_frame)]
    values_partial: ValuesUi,

    // final output controls - pattern for names of generate file(s), etc.
    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, flex_shrink: 1.0, min_size: consts::MIN_TWOLINER_SIZE)]
    output_frame: nwg::Frame,
    #[nwg_partial(parent: output_frame)]
    #[nwg_events((button, OnButtonClick): [PiagamDispendikApp::generate_docxs(SELF)])]
    output_partial: OutputUi,
}

impl PiagamDispendikApp {
    /// Proxy event handler for TemplateUi partial.
    fn open_new_file_docx(&self) {
        match self.template_partial.get_browse_file(&self.window) {
            Some(file) => self.load_docx(&file),
            None => eprintln!("file picked from browser file is empty"),
        }
    }

    fn open_new_file_data(&self) {
        match self.data_partial.get_browse_file(&self.window) {
            Some(file) => self.load_csv(&file),
            None => eprintln!("file picked from browser file is empty"),
        }
    }

    /// Drop of files event handler on app window - loads docx template for processing.
    /// Acts as alternative approach to open file (instead of "Load template" button).
    pub fn load_drop_files(&self, data: &nwg::EventData) {
        for file in data.on_file_drop().files().into_iter() {
            // only first file processed - add multiple file handling if/when such feature implemented
            if file.ends_with(".docx") {
                self.load_docx(&file);
            } else if file.ends_with(".csv") {
                self.load_csv(&file);
            } else if file.ends_with(".xlsx") {
                self.load_excel(&file);
            } else {
                continue;
            }
        }
    }

    /// Returns generic error message for opening the template modal messages.
    fn failed_load_str(&self) -> String {
        lang::tr("ui-docx-load-failed")
    }

    fn load_csv(&self, file: &str) {
        match DataInput::open(&file) {
            Ok(inpt) => {
                self.data_partial.set_current_data(file);
                self.values_partial.insert_header(&inpt.header);
                self.values_partial.load_data(&inpt.file_data);
                self.opened_data.replace(Some(inpt));
            }
            Err(error) => {
                nwg::modal_error_message(
                    &self.window,
                    &error.to_string(),
                    "Error on Opening input data file",
                );
            }
        }
    }

    fn load_excel(&self, file: &str) {
        todo!("implements for opening load excel file (xlsx) {:?}", file)
    }

    /// Loads & bind new docx structure from file to the app.
    /// Updates all the app sub-components with new DOCX info as needed.
    fn load_docx<P: AsRef<Path>>(&self, file: P) {
        match DocxTemplate::open(file.as_ref()) {
            Ok(docx) => {
                let tokens = match docx.template_tokens() {
                    Ok(tokens) => tokens,
                    Err(err) => {
                        let err_msg = self.failed_load_str();
                        nwg::modal_error_message(&self.window, &err_msg, &err.to_string());
                        return;
                    }
                };

                self.opened_docx.replace(Some(docx));

                self.template_partial
                    .set_current_docx(file.as_ref().to_str().unwrap());

                let separator = self.options_partial.get_separator();
                self.tokens_partial
                    .set_tokens_of_frame(&self.tokens_frame, &tokens, &separator);

                self.tokens_frame.set_visible(true);
                self.output_frame.set_visible(true);

                let output_pattern = format!("{}.docx", &tokens[0]);
                self.output_partial.set_output_pattern(&output_pattern);
            }
            Err(err) => {
                let err_msg = self.failed_load_str();
                nwg::modal_info_message(&self.window, &err_msg, &err.to_string());
                // return;
            }
        }
    }

    /// Triggers batch generation of DOCX files from input data.
    fn generate_docxs(&self) {
        let docx_ref = self.opened_docx.borrow();
        let generator = match &*docx_ref {
            Some(docx) => docx,
            None => {
                let title = lang::tr("ui-docx-failure");
                let content = lang::tr("ui-docx-no-template");
                nwg::modal_info_message(&self.window, &title, &content);
                return;
            }
        };
        let values: Vec<ValuePack> = match self.opened_data.take() {
            Some(inner) => inner.file_data,
            None => Vec::new(),
        };

        let _separator = &self.options_partial.get_separator();
        match generator.build_docx_batch(
            &self.tokens_partial.get_selected_tokens(),
            &values,
            &self.output_partial.output_pattern(),
        ) {
            Err(err) => {
                nwg::modal_error_message(
                    &self.window,
                    &lang::tr("ui-docx-load-failed"),
                    &err.to_string(),
                );
            }
            Ok(()) => {
                nwg::modal_info_message(
                    &self.window,
                    &lang::tr("ui-docx-success"),
                    &lang::tr("ui-docx-generated"),
                );
            }
        }
    }

    /// Invoke language change from the "options" partial...
    fn set_lang(&self) {
        if let Err(msg) = self.options_partial.set_current_lang() {
            let title = &lang::tr("ui-docx-failure");
            let content = &msg;
            nwg::modal_error_message(&self.window, title, content);
            return;
        }
        self.reset_language();
    }

    /// Update all existing UI items that have bound strings.
    /// Each specific partial is responsible for invoking all required updates.
    fn reset_language(&self) {
        self.template_partial.reset_language();
        self.data_partial.reset_language();
        self.tokens_partial.reset_language();
        self.values_partial.reset_language();
        self.options_partial.reset_language();
        self.output_partial.reset_language();
    }

    /// Main app "destructor", not much to do anyhow...
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}
